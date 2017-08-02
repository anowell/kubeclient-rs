use reqwest::{self, StatusCode};
use std::path::Path;
use config::KubeConfig;
use data::Status;
use std::fs::File;
use std::io::Read;
use openssl::pkcs12::Pkcs12;
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{self, Value};
use url::Url;
use errors::*;

pub struct KubeClient {
    client: reqwest::Client,
    base_url: Url,
    namespace: String,
}

impl KubeClient {
    pub fn new<P: AsRef<Path>>(path: P) -> KubeClient {
        let kubeconfig = KubeConfig::load(path).expect("TODO: explain why load failed");
        let context = kubeconfig.default_context().expect("Failed to get default context from kubeconfig");
        let auth_info = context.user;

        let cluster = context.cluster;

        let ca_cert = cluster.ca_cert().expect("kubeconfig missing CA cert");
        let client_cert = auth_info.client_certificate().expect("kubeconfig missing client cert");
        let client_key = auth_info.client_key().expect("kubeconfig missing client key");
        let pkcs_cert = Pkcs12::builder().build("", "admin", &client_key, &client_cert).expect("Failed to build Pkcs12");

        let req_ca_cert = reqwest::Certificate::from_der(&ca_cert.to_der().unwrap()).unwrap();
        let req_pkcs_cert = reqwest::Pkcs12::from_der(&pkcs_cert.to_der().unwrap(), "").unwrap();

        let client = reqwest::Client::builder().expect("failed to create reqwest client builder")
            .add_root_certificate(req_ca_cert)
            .expect("Failed to add root cert to reqwest client")
            .identity(req_pkcs_cert)
            .expect("Failed to add PKCS cert and key to reqwest client")
            .danger_disable_hostname_verification()
            .build()
            .expect("Failed to build reqwest client");

        KubeClient { client, base_url: cluster.server, namespace: "default".to_owned() }
    }

    pub fn namespace(&self, namespace: &str) -> KubeClient {
        KubeClient {
            client: self.client.clone(),
            base_url: self.base_url.clone(),
            namespace: namespace.to_owned(),
        }
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
        self.http_get_json(&route)
    }

    pub fn exists(&self, kind: &str, resource: &str) -> Result<bool> {
        let route = format!("api/v1/namespaces/{}/{}/{}", self.namespace, kind, resource);
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

    pub fn list<D>(&self, kind: &str) -> Result<D>
    where D: DeserializeOwned
    {
        let route = format!("api/v1/namespaces/{}/{}", self.namespace, kind);
        self.http_get_json(&route)
    }

    pub fn get<D>(&self, kind: &str, resource: &str) -> Result<D>
    where D: DeserializeOwned
    {
        let route = format!("api/v1/namespaces/{}/{}/{}", self.namespace, kind, resource);
        self.http_get_json(&route)
    }

    pub fn create<S, D>(&self, kind: &str, resource: &str, data: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned
    {
        let body = json!({
            "data": data,
            "metadata": { "name": resource }
        });
        self.apply(kind, &body)
    }

    pub fn apply<S, D>(&self, kind: &str, body: &S) -> Result<D>
    where S: Serialize,
          D: DeserializeOwned
    {
        let route = format!("api/v1/namespaces/{}/{}", self.namespace, kind);
        self.http_post_json(&route, &body)
    }

    pub fn apply_file<D, P: AsRef<Path>>(&self, kind: &str, path: P) -> Result<D>
    where D: DeserializeOwned
    {
        let file = File::open(path)?;
        let body: Value = serde_json::from_reader(file)?;
        self.apply(&kind, &body)
    }

    pub fn delete(&self, kind: &str, resource: &str) -> Result<()> {
        let route = format!("api/v1/namespaces/{}/{}/{}", self.namespace, kind, resource);
        self.http_delete(&route).map(|_| ())
    }

    //
    // Low-level
    //

    fn http_get(&self, route: &str) -> Result<reqwest::Response> {
        let mut response = self.client.get(self.base_url.join(route)?)
            .expect("URL failed to be built")
            .send()
            .chain_err(|| "Failed to GET URL")?;

        if !response.status().is_success() {
            let status: Status = response.json()
                .chain_err(|| "Failed to decode error response as 'Status'")?;
            bail!(status.message);
        }
        Ok(response)
    }

    fn http_get_json<D>(&self, route: &str) -> Result<D>
    where D: DeserializeOwned
    {
        let mut response = self.http_get(route)?;
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
