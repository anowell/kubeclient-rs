mod secret;
mod config_map;
mod node;
mod deployment;

pub use self::secret::*;
pub use self::config_map::*;
pub use self::node::*;
pub use self::deployment::*;

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
use std::collections::BTreeMap;
// use std::str::FromStr;

#[derive(Serialize, Deserialize, Debug)]
pub enum Kind { Secret, ConfigMap, Node, Deployment }

impl Kind {
    pub fn route(&self) -> &'static str {
        match *self {
            Kind::Secret => "secrets",
            Kind::ConfigMap => "configmaps",
            Kind::Node => "nodes",
            Kind::Deployment => "deployments",
        }
    }
}

// impl FromStr for Kind {
//     type Err = String;
//     fn from_str(s: &str) -> Result<Self, Self::Err> {
//         match s {
//             "Secret" => Ok(Kind::Secret),
//             "ConfigMap" => Ok(Kind::ConfigMap),
//             "Node" => Ok(Kind::Node),
//             "Deployment" => Ok(Kind::Deployment),
//             _ => Err(format!("Unsupported kind '{}'", s)),
//         }
//     }
// }

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
    pub name: Option<String>,
    pub namespace: Option<String>,
    pub uid: Option<String>,
    pub creation_timestamp: Option<DateTime<Utc>>,
    pub annotations: Option<BTreeMap<String, String>>,
    pub labels: Option<BTreeMap<String, String>>,
}
