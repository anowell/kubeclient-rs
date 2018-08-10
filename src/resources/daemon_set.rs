use super::*;
use ::k8s_api::api::apps::v1beta2::{DaemonSetSpec, DaemonSetStatus};
use k8s_api::apimachinery::pkg::apis::meta::v1::ObjectMeta;

pub(crate) static DAEMON_SET_INFO: KindInfo = KindInfo {
    plural: "daemonsets",
    default_namespace: Some("default"),
    api: V1_BETA_API,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct DaemonSet {
    /// The desired behavior of this daemon set. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    pub spec: DaemonSetSpec,

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#metadata
    pub metadata: ObjectMeta,

    /// The current status of this daemon set. This data may be out of date by some window of time. Populated by the system. Read-only. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<DaemonSetStatus>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DaemonSetList {
    items: Vec<DaemonSet>,
}

impl DaemonSet {
    pub fn new(name: &str) -> DaemonSet {
        let spec = DaemonSetSpec::default();
        let metadata = ObjectMeta{ name: Some(name.to_owned()), ..Default::default() };
        DaemonSet { spec, metadata, status: None }
    }
}

impl Resource for DaemonSet {
    fn kind() -> Kind { Kind::DaemonSet }
}

impl ListableResource for DaemonSet {
    type ListResponse = DaemonSetList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}
