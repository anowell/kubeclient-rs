use super::*;
use k8s_api::api::apps::v1::{DeploymentSpec, DeploymentStatus};
use k8s_api::api::apps::v1beta1::{ScaleSpec, ScaleStatus};
use k8s_api::apimachinery::pkg::apis::meta::v1::ObjectMeta;

pub(crate) static DEPLOYMENT_INFO: KindInfo = KindInfo {
    plural: "deployments",
    default_namespace: Some("default"),
    api: V1_BETA_API,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Deployment {
    /// Specification of the desired behavior of the Deployment.
    pub spec: DeploymentSpec,

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#metadata
    pub metadata: ObjectMeta,

    /// Most recently observed status of the Deployment.
    pub status: Option<DeploymentStatus>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Scale {
    /// defines the behavior of the scale. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status.
    pub spec: ScaleSpec,

    /// Standard object metadata; More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#metadata.
    pub metadata: ObjectMeta,

     /// current status of the scale. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status. Read-only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<ScaleStatus>,
}

impl Scale {
    pub(crate) fn replicas(namespace: &str, name: &str, count: u32) -> Scale {
        Scale {
            spec: ScaleSpec { replicas: Some(count as i32) },
            metadata: ObjectMeta {
                name: Some(name.to_owned()),
                namespace: Some(namespace.to_owned()),
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DeploymentList {
    items: Vec<Deployment>,
}

impl Deployment {
    pub fn new(name: &str) -> Deployment {
        let metadata = ObjectMeta{ name: Some(name.to_owned()), ..Default::default() };
        Deployment { metadata, ..Default::default() }
    }
}

impl Resource for Deployment {
    fn kind() -> Kind { Kind::Deployment }
}

impl ListableResource for Deployment {
    type ListResponse = DeploymentList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}
