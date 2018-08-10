use super::*;
use k8s_api::api::core::v1::{ServiceSpec, ServiceStatus};
use k8s_api::apimachinery::pkg::apis::meta::v1::ObjectMeta;

pub(crate) static SERVICE_INFO: KindInfo = KindInfo {
    plural: "services",
    default_namespace: Some("default"),
    api: V1_API,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Service {
    /// Spec defines the behavior of a service. https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    pub spec: ServiceSpec,

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#metadata
    pub metadata: ObjectMeta,

    /// Most recently observed status of the service. Populated by the system. Read-only. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ServiceStatus>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ServiceList {
    items: Vec<Service>,
}

impl Service {
    pub fn new(name: &str) -> Service {
        let metadata = ObjectMeta{ name: Some(name.to_owned()), ..Default::default() };
        Service { metadata, ..Default::default() }
    }
}

impl Resource for Service {
    fn kind() -> Kind { Kind::Service }
}


impl ListableResource for Service {
    type ListResponse = ServiceList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}
