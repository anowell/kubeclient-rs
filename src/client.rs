use reqwest::{self, StatusCode};
use std::path::Path;
use config::KubeConfig;
use resources::{Kind, Resource, Metadata, ListableResource, ListQuery, Status};
use std::fs::File;
use std::io::Read;
use openssl::pkcs12::Pkcs12;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use serde_json::{self, Value};
use serde_yaml;
use url::Url;
use std::collections::BTreeMap;
use std::borrow::Borrow;
use walkdir::WalkDir;
use errors::*;

pub struct KubeClient {
    kube: KubeClientLowLevel,
    namespace: Option<String>,
}

impl KubeClient {
    pub fn from_conf<P: AsRef<Path>>(path: P) -> Result<KubeClient> {
        Ok(KubeClient{
            kube: KubeClientLowLevel::from_conf(path)?,
            namespace: None,
        })
    }

    pub fn low_level(&self) -> &KubeClientLowLevel {
        &self.kube
    }

    pub fn namespace(&self, namespace: &str) -> KubeClient {
        KubeClient { kube: self.kube.clone(), namespace: Some(namespace.to_owned()) }
    }

    pub fn healthy(&self) -> Result<bool> {
        Ok(self.kube.health()? == "ok")
    }

    pub fn apply_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let _: Vec<Value> = self.kube.apply_path(path)?;
        Ok(())
    }

    pub fn exists<R: Resource>(&self, name: &str) -> Result<bool> {
        let mut route = ResourceRoute::new(R::api(), R::kind().route(), name);
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        self.kube.exists(&route)
    }

    pub fn get<R: Resource>(&self, name: &str) -> Result<R> {
        let mut route = ResourceRoute::new(R::api(), R::kind().route(), name);
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        self.kube.get(&route)
    }

    pub fn list<R: ListableResource>(&self, query: Option<&ListQuery>) -> Result<Vec<R>> {
        let mut route = KindRoute::new(R::api(), R::kind().route());
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        if let Some(query) = query {
            let json = serde_json::to_string(&query)?;
            let map: BTreeMap<String,String> = serde_json::from_str(&json)?;
            route.query(map);
        }
        let response: R::ListResponse = self.kube.list(&route)?;
        Ok(R::list_items(response))
    }

    pub fn create<R: Resource>(&self, resource: &R) -> Result<R> {
        let mut route = KindRoute::new(R::api(), R::kind().route());
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        self.kube.apply(&route, resource)
    }

    pub fn delete<R: Resource>(&self, name: &str) -> Result<()> {
        let mut route = ResourceRoute::new(R::api(), R::kind().route(), name);
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        self.kube.delete(&route)
    }

    fn get_ns<'a, R: Resource>(&'a self) -> Option<&'a str> {
        match self.namespace {
            Some(ref ns) => Some(ns),
            None => R::default_namespace(),
        }
    }
}

#[derive(Clone)]
pub struct KubeClientLowLevel {
    client: reqwest::Client,
    base_url: Url,
}

// This is only used for figuring out the API endpoint to use
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MinimalResource {
    api_version: String,
    kind: Kind,
    metadata: Metadata,
}

impl KubeClientLowLevel {
    pub fn from_conf<P: AsRef<Path>>(path: P) -> Result<KubeClientLowLevel> {
        let kubeconfig = KubeConfig::load(path)?;
        let context = kubeconfig.default_context()?;
        let auth_info = context.user;

        let cluster = context.cluster;

        let ca_cert = cluster.ca_cert()
            .chain_err(|| "kubeconfig missing CA cert")?;
        let client_cert = auth_info.client_certificate()
            .chain_err(|| "kubeconfig missing client cert")?;
        let client_key = auth_info.client_key().chain_err(|| "kubeconfig missing client key")?;
        let pkcs_cert = Pkcs12::builder()
            .build("", "admin", &client_key, &client_cert)
            .chain_err(|| "Failed to build Pkcs12")?;

        let req_ca_cert = reqwest::Certificate::from_der(&ca_cert.to_der().unwrap()).unwrap();
        let req_pkcs_cert = reqwest::Pkcs12::from_der(&pkcs_cert.to_der().unwrap(), "").unwrap();

        let client = reqwest::Client::builder()
            .chain_err(|| "Failed to create reqwest client builder")?
            .add_root_certificate(req_ca_cert)
            .chain_err(|| "Failed to add root cert to reqwest client")?
            .identity(req_pkcs_cert)
            .chain_err(|| "Failed to add PKCS cert and key to reqwest client")?
            .danger_disable_hostname_verification()
            .build()
            .chain_err(|| "Failed to build reqwest client")?;

        Ok(KubeClientLowLevel { client, base_url: cluster.server })
    }

    pub fn health(&self) -> Result<String> {
        let mut response = self.http_get(self.base_url.join("healthz")?)?;
        let mut output = String::new();
        let _ = response.read_to_string(&mut output)?;
        Ok(output)
    }

    pub fn check<D>(&self, route: &str) -> Result<D>
    where D: DeserializeOwned
    {
        self.http_get_json(self.base_url.join(route)?)
    }

    pub fn exists(&self, route: &ResourceRoute) -> Result<bool> {
        let url = route.build(&self.base_url)?;
        let mut response = self.client.get(url)?
            .send()
            .chain_err(|| "Failed to GET URL")?;

        match response.status() {
            StatusCode::NotFound => Ok(false),
            s if s.is_success() => Ok(true),
            _ => {
                let status: Status = response.json()
                    .chain_err(|| "Failed to decode error response as 'Status'")?;
                bail!(status.message);
            }
        }
    }

    pub fn list<D>(&self, route: &KindRoute) -> Result<D>
    where D: DeserializeOwned {
        let url = route.build(&self.base_url)?;
        self.http_get_json(url)
    }

    pub fn get<D>(&self, route: &ResourceRoute) -> Result<D>
    where D: DeserializeOwned
    {
        let url = route.build(&self.base_url)?;
        self.http_get_json(url)
    }

    pub fn create<S, D>(&self, route: &KindRoute, resource: &str, data: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned
    {
        let body = json!({
            "data": data,
            "metadata": { "name": resource }
        });
        self.apply(route, &body)
    }

    pub fn apply<S, D>(&self, route: &KindRoute, body: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned
    {
        let url = route.build(&self.base_url)?;
        self.http_post_json(url, &body)
    }

    pub fn apply_path<D, P: AsRef<Path>>(&self, path: P) -> Result<Vec<D>>
    where D: DeserializeOwned + ::std::fmt::Debug
    {
        WalkDir::new(path).max_depth(1).into_iter()
            .filter_map(|e| e.ok())
            .filter(|entry| entry.file_type().is_file())
            .filter(|entry| {
                if let Some(ext) = entry.path().extension() {
                    let ext = ext.to_string_lossy().to_lowercase();
                    return ext == "json" || ext == "yaml";
                }
                false
            })
            .map(|entry| {
                let file = File::open(entry.path())?;
                let ext = entry.path().extension().unwrap().to_string_lossy();
                self.apply_file(file, &ext)
            })
            .collect()
    }

    // TODO: make format enum of Json/Yaml
    pub fn apply_file<D>(&self, mut file: File, format: &str) -> Result<D>
    where D: DeserializeOwned + ::std::fmt::Debug
    {
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes)?;
        let (mini, body): (MinimalResource, Value) = match &*format.to_lowercase() {
            "json" => (serde_json::from_slice(&bytes)?, serde_json::from_slice(&bytes)?),
            "yaml" => (serde_yaml::from_slice(&bytes)?, serde_yaml::from_slice(&bytes)?),
            _ => unreachable!("kubeclient bug: unexpected and unfiltered file extension"),
        };
        let root = if mini.api_version.starts_with("v") {
            "/api"
        } else {
            "/apis"
        };
        let url = match mini.metadata.namespace {
            Some(ns) => self.base_url.join(
                &format!("{}/{}/namespaces/{}/{}", root, mini.api_version, ns, mini.kind.route())
            )?,
            None => self.base_url.join(
                &format!("{}/{}/{}", root, mini.api_version, mini.kind.route())
            )?,
        };
        let resp = self.http_post_json(url, &body)?;
        Ok(resp)
    }

    pub fn delete(&self, route: &ResourceRoute) -> Result<()> {
        let url = route.build(&self.base_url)?;
        self.http_delete(url).map(|_| ())
    }

    //
    // Low-level
    //

    fn http_get(&self, url: Url) -> Result<reqwest::Response> {
        let mut req = self.client.get(url)?;

        let mut response = req.send().chain_err(|| "Failed to GET URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode kubernetes error response as 'Status'")?;
            bail!(format!("Kubernetes API error: {}", status.message));
        }
        Ok(response)
    }

    fn http_get_json<D: DeserializeOwned>(&self, url: Url) -> Result<D> {
        let mut response = self.http_get(url)?;
        Ok(response.json().chain_err(|| "Failed to decode JSON response")?)
    }

    fn http_post_json<S, D>(&self, url: Url, body: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned,
    {
        let mut response = self.client.post(url)?
            .json(&body).expect("JSON serialization failed")
            .send()
            .chain_err(|| "Failed to POST URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode kubernetes error response as 'Status'")?;
            bail!(format!("Kubernetes API error: {}", status.message));
        }

        Ok(response.json().chain_err(|| "Failed to decode JSON response")?)
    }

    fn http_delete(&self, url: Url) -> Result<reqwest::Response> {
        let mut response = self.client.delete(url)?
            .send()
            .chain_err(|| "Failed to DELETE URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode kubernetes error response as 'Status'")?;
            bail!(format!("Kubernetes API error: {}", status.message));
        }

        Ok(response)
    }

}


pub struct KindRoute<'a> {
    api: &'a str,
    namespace: Option<&'a str>,
    kind: &'a str,
    query: Option<Vec<(String, String)>>,
}

pub struct ResourceRoute<'a> {
    api: &'a str,
    namespace: Option<&'a str>,
    kind: &'a str,
    resource: &'a str,
    query: Option<Vec<(String, String)>>,
}


impl<'a> KindRoute<'a> {
    pub fn new(api: &'a str, kind: &'a str) -> KindRoute<'a> {
        KindRoute {
            api, kind,
            namespace: None,
            query: None,
        }
    }

    pub fn namespace(&mut self, namespace: &'a str) -> &mut KindRoute<'a> {
        self.namespace = Some(namespace);
        self
    }

    pub fn query<I, K, V>(&mut self, query: I) -> &mut KindRoute<'a>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        // This is ugly, but today the borrow checker beat me
        let pairs = query.into_iter()
            .map(|i| {
                let (ref k, ref v) = *i.borrow();
                (k.as_ref().to_owned(), v.as_ref().to_owned())
            })
            .collect();
        self.query = Some(pairs);
        self
    }

    pub fn build(&self, base_url: &Url) -> Result<Url> {
        let path = match self.namespace {
            Some(ns) => format!("{}/namespaces/{}/{}", self.api, ns, self.kind),
            None => format!("{}/{}", self.api, self.kind),
        };
        let mut url = base_url.join(&path)?;
        if let Some(ref query) = self.query {
            url.query_pairs_mut().extend_pairs(query);
        }
        Ok(url)
    }
}

impl<'a> ResourceRoute<'a> {
    pub fn new(api: &'a str, kind: &'a str, resource: &'a str) -> ResourceRoute<'a> {
        ResourceRoute {
            api, kind, resource,
            namespace: None,
            query: None,
        }
    }

    pub fn namespace(&mut self, namespace: &'a str) -> &mut ResourceRoute<'a> {
        self.namespace = Some(namespace);
        self
    }


    pub fn query<I, K, V>(&mut self, query: I) -> &mut ResourceRoute<'a>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        // This is ugly, but today the borrow checker beat me
        let pairs = query.into_iter()
            .map(|i| {
                let (ref k, ref v) = *i.borrow();
                (k.as_ref().to_owned(), v.as_ref().to_owned())
            })
            .collect();
        self.query = Some(pairs);
        self
    }

    fn build(&self, base_url: &Url) -> Result<Url> {
        let path = match self.namespace {
            Some(ns) => format!("{}/namespaces/{}/{}/{}", self.api, ns, self.kind, self.resource),
            None => format!("{}/{}/{}", self.api, self.kind, self.resource),
        };
        let mut url = base_url.join(&path)?;
        if let Some(ref query) = self.query {
            url.query_pairs_mut().extend_pairs(query);
        }
        Ok(url)
    }
}