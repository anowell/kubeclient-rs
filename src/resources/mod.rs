mod secret;
mod config_map;
mod node;
mod deployment;
mod network_policy;
mod pod;
mod service;

pub use self::secret::*;
pub use self::config_map::*;
pub use self::node::*;
pub use self::deployment::*;
pub use self::network_policy::*;
pub use self::pod::*;
pub use self::service::*;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
use std::collections::BTreeMap;
use std::borrow::Borrow;

#[derive(Serialize, Deserialize, Debug)]
pub enum Kind { Deployment, ConfigMap, NetworkPolicy, Node, Pod, Secret, Service }

impl Kind {
    pub fn route(&self) -> &'static str {
        match *self {
            Kind::ConfigMap => "configmaps",
            Kind::Deployment => "deployments",
            Kind::NetworkPolicy => "networkpolicies",
            Kind::Node => "nodes",
            Kind::Pod => "pods",
            Kind::Secret => "secrets",
            Kind::Service => "services",
        }
    }
}

// Debug output of Kind is exactly what we want for Display
impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub trait Resource: Serialize + DeserializeOwned {
    fn kind() -> Kind;
    fn default_namespace() -> Option<&'static str> {
        Some("default")
    }
    fn api() -> &'static str {
        "/api/v1"
    }

}

pub trait ListableResource: Resource {
    type ListResponse: DeserializeOwned;
    fn list_items(response: Self::ListResponse) -> Vec<Self>;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub kind: String,
    pub api_version: String,
    pub metadata: Metadata,
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub uid: Option<String>,
    pub creation_timestamp: Option<DateTime<Utc>>,
    pub annotations: Option<BTreeMap<String, String>>,
    pub labels: Option<BTreeMap<String, String>>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    field_selector: Option<BTreeMap<String, String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_seconds: Option<String>,
}

impl ListQuery {
    pub fn field_selector<B, K, V>(mut self, field_selector: B) -> Self
    where
        B: Borrow<(K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        let f = field_selector.borrow();
        let (k, v) = (f.0.as_ref(), f.1.as_ref());
        match self.field_selector {
            Some(ref mut m) => {
                m.insert(k.to_owned(), v.to_owned());
            }
            None => {
                let mut m = BTreeMap::new();
                m.insert(k.to_owned(), v.to_owned());
                self.field_selector = Some(m);
            }
        }
        self
    }
    pub fn label_selector<S: Into<String>>(&self, label_selector: S) -> Self {
        let mut new = self.clone();
        new.label_selector = Some(label_selector.into());
        new
    }
    pub fn resource_version<S: Into<String>>(&self, resource_version: S) -> Self {
        let mut new = self.clone();
        new.resource_version = Some(resource_version.into());
        new
    }
    pub fn timeout_seconds(&self, timeout_seconds: u32) -> Self {
        let mut new = self.clone();
        new.timeout_seconds = Some(timeout_seconds.to_string());
        new
    }
}