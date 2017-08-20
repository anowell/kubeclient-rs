use super::*;
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
pub struct DaemonSet {
    pub spec: DaemonSetSpec,
    pub metadata: Metadata,
    pub status: Option<DaemonSetStatus>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DaemonSetSpec {
    pub selector: Option<Value>,
    pub template: Option<Value>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DaemonSetStatus {
    pub current_number_scheduled : u32,
    pub desired_number_scheduled : u32,
    pub number_misscheduled : u32,
    pub number_ready : u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DaemonSetList {
    items: Vec<DaemonSet>,
}

impl DaemonSet {
    pub fn new(name: &str) -> DaemonSet {
        let spec = DaemonSetSpec::default();
        let metadata = Metadata{ name: Some(name.to_owned()), ..Default::default() };
        DaemonSet { spec, metadata, status: None }
    }
}

impl Resource for DaemonSet {
    fn kind() -> Kind { Kind::DaemonSet }
    fn api() -> &'static str {
        "/apis/extensions/v1beta1"
    }
}

impl ListableResource for DaemonSet {
    type ListResponse = DaemonSetList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}