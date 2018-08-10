mod secret;
mod config_map;
mod node;
mod daemon_set;
mod deployment;
mod network_policy;
mod pod;
mod service;

pub use self::secret::*;
pub use self::config_map::*;
pub use self::node::*;
pub use self::daemon_set::*;
pub use self::deployment::*;
pub use self::network_policy::*;
pub use self::pod::*;
pub use self::service::*;

use k8s_api::apimachinery::pkg::apis::meta::v1::ObjectMeta;
use serde::Serialize;
use serde::de::DeserializeOwned;
use std::fmt;
use std::collections::BTreeMap;
use std::ops::Deref;

pub(crate) const V1_API: &str = "/api/v1";
pub(crate) const V1_BETA_API: &str = "/apis/extensions/v1beta1";

#[derive(Serialize, Deserialize, Debug)]
pub enum Kind { DaemonSet, Deployment, ConfigMap, NetworkPolicy, Node, Pod, Secret, Service }

impl Deref for Kind {
    type Target = KindInfo;
    fn deref(&self) -> &KindInfo {
        match *self {
            Kind::ConfigMap => &CONFIG_MAP_INFO,
            Kind::DaemonSet => &DAEMON_SET_INFO,
            Kind::Deployment => &DEPLOYMENT_INFO,
            Kind::NetworkPolicy => &NETWORK_POLICY_INFO,
            Kind::Node => &NODE_INFO,
            Kind::Pod => &POD_INFO,
            Kind::Secret => &SECRET_INFO,
            Kind::Service => &SERVICE_INFO,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct KindInfo {
    pub plural: &'static str,
    pub default_namespace: Option<&'static str>,
    pub api: &'static str,
}

// Debug output of Kind is exactly what we want for Display
impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub trait Resource: Serialize + DeserializeOwned {
    fn kind() -> Kind;
    fn api() -> &'static str {
        Self::kind().api
    }
    fn default_namespace() -> Option<&'static str> {
        Self::kind().default_namespace
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
    pub metadata: ObjectMeta,
    pub status: String,
    pub message: String,
}

#[derive(Clone, Debug, Default)]
pub struct ListQuery {
    field_selector: Option<String>,
    label_selector: Option<String>,
    resource_version: Option<String>,
    timeout_seconds: Option<String>,
}

impl ListQuery {
    pub fn as_query_pairs(&self) -> BTreeMap<&str, String> {
        let mut map = BTreeMap::new();
        if let Some(ref fs) = self.field_selector {
            map.insert("fieldSelector", fs.to_owned());
        }
        if let Some(ref ls) = self.label_selector {
            map.insert("labelSelector", ls.to_owned());
        }
        if let Some(ref rv) = self.resource_version {
            map.insert("resourceVersion", rv.to_owned());
        }
        if let Some(ref ts) = self.timeout_seconds {
            map.insert("timeoutSeconds", ts.to_owned());
        }
        map
    }

    /// Be aware of: https://github.com/kubernetes/kubernetes/issues/1362
    pub fn field_selector<S: Into<String>>(mut self, field_selector: S) -> Self {
        self.field_selector = Some(field_selector.into());
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
