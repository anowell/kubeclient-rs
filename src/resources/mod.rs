mod secret;
mod config_map;
mod node;

pub use self::secret::*;
pub use self::config_map::*;
pub use self::node::*;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
use std::collections::BTreeMap;

#[derive(Serialize, Deserialize, Debug)]
pub enum Kind { Secret, ConfigMap, Node }

impl Kind {
    pub fn route(&self) -> &'static str {
        match *self {
            Kind::Secret => "secrets",
            Kind::ConfigMap => "configmaps",
            Kind::Node => "nodes",
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
}

pub trait ListableResource: Resource {
    type QueryParams: Serialize;
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
    name: String,
    uid: Option<String>,
    creation_timestamp: Option<DateTime<Utc>>,
    annotations: BTreeMap<String, String>,
    labels: BTreeMap<String, String>,
}
