use reqwest::{self, header, StatusCode};
use std::path::Path;
use config::KubeConfig;
use resources::*;
use std::fs::File;
use std::io::Read;
use openssl::pkcs12::Pkcs12;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{self, Value};
use serde_yaml;
use url::Url;
use std::borrow::Borrow;
use walkdir::WalkDir;
use errors::*;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;


#[derive(Clone)]
pub struct KubeLowLevel {
    pub(crate) client: reqwest::Client,
    pub(crate) base_url: Url,
}

// This is only used for figuring out the API endpoint to use
#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct MinimalResource {
    api_version: String,
    kind: Kind,
    metadata: ObjectMeta,
}

impl KubeLowLevel {
    pub fn load_conf<P: AsRef<Path>>(path: P) -> Result<KubeLowLevel> {
        let kubeconfig = KubeConfig::load(path)?;
        let context = kubeconfig.default_context()?;
        let auth_info = context.user;

        let cluster = context.cluster;

        let mut headers = header::Headers::new();
        let mut client = reqwest::Client::builder();

        let mut client = if let Some(ca_cert) = cluster.ca_cert() {
            let req_ca_cert = reqwest::Certificate::from_der(&ca_cert.to_der().unwrap()).unwrap();
            client.add_root_certificate(req_ca_cert)
        } else { &mut client };

        let client = if auth_info.client_certificate().is_some() && auth_info.client_key().is_some() {
            let crt = auth_info.client_certificate().unwrap();
            let key = auth_info.client_key().unwrap();
            let pkcs_cert = Pkcs12::builder().build("", "admin", &key, &crt).chain_err(|| "Failed to build Pkcs12")?;
            let req_pkcs_cert = reqwest::Identity::from_pkcs12_der(&pkcs_cert.to_der().unwrap(), "").unwrap();
            client.identity(req_pkcs_cert)
        } else { &mut client };
        
        if let Some(username) = auth_info.username {
            headers.set(header::Authorization(
                header::Basic { username: username,
                                password: auth_info.password }
            ));
        } else if let Some(token) = auth_info.token {
            headers.set(header::Authorization(
                header::Bearer { token: token }
            ));
        }

        let client = client.default_headers(headers)
                           .danger_disable_hostname_verification()
                           .build()
                           .chain_err(|| "Failed to build reqwest client")?;

        Ok(KubeLowLevel { client, base_url: cluster.server })
    }

    pub fn health(&self) -> Result<String> {
        let mut response = self.http_get(self.base_url.join("healthz")?)?;
        let mut output = String::new();
        let _ = response.read_to_string(&mut output)?;
        Ok(output)
    }


    pub fn exists(&self, route: &ResourceRoute) -> Result<bool> {
        let url = route.build(&self.base_url)?;
        let mut response = self.client.get(url)
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

    // pub fn create<S, D>(&self, route: &KindRoute, resource: &str, data: &S) -> Result<D>
    // where S: Serialize,
    //       D: DeserializeOwned
    // {
    //     let body = json!({
    //         "data": data,
    //         "metadata": { "name": resource }
    //     });
    //     self.apply(route, &body)
    // }

    pub fn apply<S, D>(&self, route: &KindRoute, body: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned
    {
        let url = route.build(&self.base_url)?;
        self.http_post_json(url, &body)
    }

    pub(crate) fn each_resource_path<D, F, P: AsRef<Path>>(&self, path: P, handler: F) -> Result<Vec<D>>
    where
        D: DeserializeOwned + ::std::fmt::Debug,
        F: Fn(&Path) -> Result<D>,
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
            .map(|entry| handler(entry.path()))
            .collect()
    }

    // TODO: This function could use a serious refactoring
    pub(crate) fn apply_file<D>(&self, path: &Path) -> Result<D>
    where D: DeserializeOwned + ::std::fmt::Debug
    {
        let mut bytes = Vec::new();
        let ext = path.extension().unwrap().to_string_lossy().to_lowercase();
        let mut file = File::open(path)?;
        file.read_to_end(&mut bytes)?;
        let body: Value = match &*ext {
            "json" => serde_json::from_slice(&bytes)?,
            "yaml" => serde_yaml::from_slice(&bytes)?,
            _ => unreachable!("kubeclient bug: unexpected and unfiltered file extension"),
        };
        let mini: MinimalResource = serde_json::from_value(body.clone())?;

        let root = if mini.api_version.starts_with("v") {
            "/api"
        } else {
            "/apis"
        };
        let name = mini.metadata.name.expect("must set metadata.name to apply kubernetes resource");
        let kind_path = match mini.metadata.namespace.as_ref().map(|x| &**x).borrow().or(mini.kind.default_namespace) {
            Some(ns) => format!("{}/{}/namespaces/{}/{}", root, mini.api_version, ns, mini.kind.plural),
            None =>format!("{}/{}/{}", root, mini.api_version, mini.kind.plural),
        };
        let kind_url = self.base_url.join(&kind_path)?;
        let resource_url = self.base_url.join(&format!("{}/{}", kind_path, name))?;

        // First check if resource already exists
        let mut response = self.client.get(resource_url).send()
            .chain_err(|| "Failed to GET URL")?;
        match response.status() {
            // Apply if resource doesn't exist
            StatusCode::NotFound => {
                let resp = self.http_post_json(kind_url, &body)?;
                Ok(resp)
            }
            // Return it if it already exists
            s if s.is_success() => {
                let resp = response.json().chain_err(|| "Failed to decode JSON response")?;
                Ok(resp)
            }
            // Propogate any other error
            _ => {
                let status: Status = response.json()
                    .chain_err(|| "Failed to decode error response as 'Status'")?;
                bail!(status.message);
            }
        }
    }

    pub(crate) fn replace_file<D>(&self, path: &Path) -> Result<D>
    where D: DeserializeOwned + ::std::fmt::Debug
    {
        let mut bytes = Vec::new();
        let ext = path.extension().unwrap().to_string_lossy().to_lowercase();
        let mut file = File::open(path)?;
        file.read_to_end(&mut bytes)?;
        let body: Value = match &*ext {
            "json" => serde_json::from_slice(&bytes)?,
            "yaml" => serde_yaml::from_slice(&bytes)?,
            _ => unreachable!("kubeclient bug: unexpected and unfiltered file extension"),
        };
        let mini: MinimalResource = serde_json::from_value(body.clone())?;

        let root = if mini.api_version.starts_with("v") {
            "/api"
        } else {
            "/apis"
        };
        let name = mini.metadata.name.expect("must set metadata.name to apply kubernetes resource");
        let url = match mini.metadata.namespace {
            Some(ns) => self.base_url.join(
                &format!("{}/{}/namespaces/{}/{}/{}", root, mini.api_version, ns, mini.kind.plural, name)
                )?,
            None => self.base_url.join(
                &format!("{}/{}/{}/{}", root, mini.api_version, mini.kind.plural, name)
                )?,
        };
        let resp = self.http_put_json(url, &body)?;
        Ok(resp)
    }

    pub fn delete(&self, route: &ResourceRoute) -> Result<()> {
        let url = route.build(&self.base_url)?;
        self.http_delete(url).map(|_| ())
    }

    //
    // Low-level
    //

    pub(crate) fn http_get(&self, url: Url) -> Result<reqwest::Response> {
        let mut req = self.client.get(url);

        let mut response = req.send().chain_err(|| "Failed to GET URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode kubernetes error response as 'Status'")?;
            bail!(format!("Kubernetes API error: {}", status.message));
        }
        Ok(response)
    }

    pub(crate) fn http_get_json<D: DeserializeOwned>(&self, url: Url) -> Result<D> {
        let mut response = self.http_get(url)?;
        Ok(response.json().chain_err(|| "Failed to decode JSON response")?)
    }

    pub(crate) fn http_post_json<S, D>(&self, url: Url, body: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned,
    {
        let mut response = self.client.post(url)
            .json(&body)
            .send()
            .chain_err(|| "Failed to POST URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode kubernetes error response as 'Status'")?;
            bail!(format!("Kubernetes API error: {}", status.message));
        }

        Ok(response.json().chain_err(|| "Failed to decode JSON response")?)
    }

    pub(crate) fn http_put_json<S, D>(&self, url: Url, body: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned,
    {
        let mut response = self.client.put(url)
            .json(&body)
            .send()
            .chain_err(|| "Failed to PUT URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode kubernetes error response as 'Status'")?;
            bail!(format!("Kubernetes API error: {}", status.message));
        }

        Ok(response.json().chain_err(|| "Failed to decode JSON response")?)
    }

    pub(crate) fn http_delete(&self, url: Url) -> Result<reqwest::Response> {
        let mut response = self.client.delete(url)
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


    // pub fn query<I, K, V>(&mut self, query: I) -> &mut ResourceRoute<'a>
    // where
    //     I: IntoIterator,
    //     I::Item: Borrow<(K, V)>,
    //     K: AsRef<str>,
    //     V: AsRef<str>,
    // {
    //     // This is ugly, but today the borrow checker beat me
    //     let pairs = query.into_iter()
    //         .map(|i| {
    //             let (ref k, ref v) = *i.borrow();
    //             (k.as_ref().to_owned(), v.as_ref().to_owned())
    //         })
    //         .collect();
    //     self.query = Some(pairs);
    //     self
    // }

    pub(crate) fn build(&self, base_url: &Url) -> Result<Url> {
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