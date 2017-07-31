use reqwest;
use std::path::Path;
use super::config::KubeConfig;
use std::io::Read;
use openssl::pkcs12::Pkcs12;
use serde::de::DeserializeOwned;
use url::Url;
use errors::*;

pub struct KubeClient {
    client: reqwest::Client,
    base_url: Url,
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

        KubeClient { client, base_url: cluster.server }
    }


    pub fn health(&self) -> Result<String> {
        let mut response = self.http_get("healthz")?;
        let mut output = String::new();
        let _ = response.read_to_string(&mut output)?;
        Ok(output)

    }

    pub fn check<T>(&self, route: &str) -> Result<T>
    where T: DeserializeOwned
    {
        self.http_get_json(&route)
    }

    pub fn list<T>(&self, namespace: &str, kind: &str) -> Result<T>
    where T: DeserializeOwned
    {
        let route = format!("api/v1/namespaces/{}/{}", namespace, kind);
        self.http_get_json(&route)
    }

    pub fn get<T>(&self, namespace: &str, kind: &str, resource: &str) -> Result<T>
    where T: DeserializeOwned
    {
        let route = format!("api/v1/namespaces/{}/{}/{}", namespace, kind, resource);
        self.http_get_json(&route)
    }

    fn http_get(&self, route: &str) -> Result<reqwest::Response> {
        let response = self.client.get(self.base_url.join(route)?)
            .expect("URL failed to be built")
            .send()
            .expect("ERROR - Request failed...")
            .error_for_status()
            .chain_err(|| "Failed to GET URL")?;
        Ok(response)
    }

    fn http_get_json<T>(&self, route: &str) -> Result<T>
    where T: DeserializeOwned
    {
        let mut response = self.http_get(route)?;
        Ok(response.json().chain_err(|| "Failed to decode response as JSON")?)
    }
}
