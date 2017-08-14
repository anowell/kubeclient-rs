use reqwest::{self, StatusCode};
use std::path::Path;
use config::KubeConfig;
use resources::{Resource, ListableResource, Status};
use std::fs::File;
use std::io::Read;
use openssl::pkcs12::Pkcs12;
use serde::{Serialize};
use serde::de::DeserializeOwned;
use serde_json::{self, Value};
use url::Url;
use std::collections::BTreeMap;
use std::borrow::Borrow;
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

    pub fn exists<R: Resource>(&self, name: &str) -> Result<bool> {
        self.kube.exists(R::kind().route(), &name, self.get_ns::<R>())
    }

    pub fn get<R: Resource>(&self, name: &str) -> Result<R> {
        self.kube.get(R::kind().route(), &name, self.get_ns::<R>())
    }

    pub fn list<R: ListableResource>(&self) -> Result<Vec<R>> {
        let response: R::ListResponse =
            self.kube.list(R::kind().route(), self.get_ns::<R>())?;
        Ok(R::list_items(response))
    }

    pub fn list_with_query<R: ListableResource>(&self, query: &R::QueryParams) -> Result<Vec<R>> {
        let json = serde_json::to_string(&query)?;
        let map: BTreeMap<String,String> = serde_json::from_str(&json)?;
        let response: R::ListResponse =
            self.kube.list_with_query(R::kind().route(), self.get_ns::<R>(), map)?;
        Ok(R::list_items(response))
    }

    pub fn create<R: Resource>(&self, resource: &R) -> Result<R> {
        self.kube.apply(R::kind().route(), self.get_ns::<R>(), resource)
    }

    pub fn delete<R: Resource>(&self, name: &str) -> Result<()> {
        self.kube.delete(R::kind().route(), &name, self.get_ns::<R>())
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
        let mut response = self.http_get("healthz")?;
        let mut output = String::new();
        let _ = response.read_to_string(&mut output)?;
        Ok(output)
    }

    pub fn check<D>(&self, route: &str) -> Result<D>
    where D: DeserializeOwned
    {
        self.http_get_json::<_,_,String,String>(&route, &[])
    }

    pub fn exists(&self, kind: &str, resource: &str, namespace: Option<&str>) -> Result<bool> {
        let route = match namespace {
            Some(ns) => format!("api/v1/namespaces/{}/{}/{}", ns, kind, resource),
            None => format!("api/v1/{}/{}", kind, resource),
        };
        let mut response = self.client.get(self.base_url.join(&route)?)
            .expect("URL failed to be built")
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

    pub fn list<D>(&self, kind: &str, namespace: Option<&str>) -> Result<D>
    where D: DeserializeOwned {
        self.list_with_query::<_,_,String,String>(kind, namespace, &[])
    }


    pub fn list_with_query<D, I, K, V>(&self, kind: &str, namespace: Option<&str>, query: I) -> Result<D>
    where D: DeserializeOwned,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>
    {
        let route = match namespace {
            Some(ns) => format!("api/v1/namespaces/{}/{}", ns, kind),
            None => format!("api/v1/{}", kind),
        };
        self.http_get_json(&route, query)
    }

    pub fn get<D>(&self, kind: &str, resource: &str, namespace: Option<&str>) -> Result<D>
    where D: DeserializeOwned
    {
        let route = match namespace {
            Some(ns) => format!("api/v1/namespaces/{}/{}/{}", ns, kind, resource),
            None => format!("api/v1/{}/{}", kind, resource),
        };
        self.http_get_json::<_,_,String,String>(&route, &[])
    }

    pub fn create<S, D>(&self, kind: &str, resource: &str, namespace: Option<&str>, data: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned
    {
        let body = json!({
            "data": data,
            "metadata": { "name": resource }
        });
        self.apply(kind, namespace, &body)
    }

    pub fn apply<S, D>(&self, kind: &str, namespace: Option<&str>, body: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned
    {
        let route = match namespace {
            Some(ns) => format!("api/v1/namespaces/{}/{}", ns, kind),
            None => format!("api/v1/{}", kind),
        };
        self.http_post_json(&route, &body)
    }

    pub fn apply_file<D, P: AsRef<Path>>(&self, kind: &str, namespace: Option<&str>, path: P) -> Result<D>
    where D: DeserializeOwned
    {
        let file = File::open(path)?;
        let body: Value = serde_json::from_reader(file)?;
        self.apply(&kind, namespace, &body)
    }

    pub fn delete(&self, kind: &str, resource: &str, namespace: Option<&str>) -> Result<()> {
        let route = match namespace {
            Some(ns) => format!("api/v1/namespaces/{}/{}/{}", ns, kind, resource),
            None => format!("api/v1/{}/{}", kind, resource),
        };

        self.http_delete(&route).map(|_| ())
    }

    //
    // Low-level
    //
    fn http_get(&self, route: &str) -> Result<reqwest::Response> {
        self.http_get_query::<_,String,String>(route, &[])
    }

    fn http_get_query<I, K, V>(&self, route: &str, query: I) -> Result<reqwest::Response>
    where
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>
     {
        let mut url = self.base_url.join(route)?;
        url.query_pairs_mut().extend_pairs(query);

        let mut req = self.client.get(url)
            .expect("URL failed to be built");

        let mut response = req.send().chain_err(|| "Failed to GET URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode error response as 'Status'")?;
            bail!(status.message);
        }
        Ok(response)
    }

    fn http_get_json<D, I, K, V>(&self, route: &str, query: I) -> Result<D>
    where
        D: DeserializeOwned,
        I: IntoIterator,
        I::Item: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>
    {
        let mut response = self.http_get_query(route, query)?;
        Ok(response.json().chain_err(|| "Failed to decode response as JSON")?)
    }

    fn http_post_json<S, D>(&self, route: &str, body: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned,
    {
        let mut response = self.client.post(self.base_url.join(route)?)
            .expect("URL failed to be built")
            .json(&body).expect("JSON serialization failed")
            .send()
            .chain_err(|| "Failed to POST URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode error response as 'Status'")?;
            bail!(status.message);
        }

        Ok(response.json().chain_err(|| "Failed to decode response as JSON")?)
    }

    fn http_delete(&self, route: &str) -> Result<reqwest::Response> {
        let mut response = self.client.delete(self.base_url.join(route)?)
            .expect("URL failed to be built")
            .send()
            .chain_err(|| "Failed to DELETE URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode error response as 'Status'")?;
            bail!(status.message);
        }

        Ok(response)
    }

}
