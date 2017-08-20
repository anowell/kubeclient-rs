use super::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Deployment {
    pub spec: DeploymentSpec,
    pub metadata: Metadata,
    pub status: Option<DeploymentStatus>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentSpec {
    pub min_ready_seconds: Option<u32>,
    pub paused: Option<bool>,
    pub progress_deadline_seconds: Option<u32>,
    pub replicas: Option<u32>,
    pub revision_history_limit: Option<u32>,
    // pub rollback_to: Option<RollbackConfig>,
    // pub selector: Option<LabelSelector>,
    // pub strategy: Option<DeploymentStrategy>,
    // pub template: Option<PodTemplateSpec>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentStatus {
    // pub conditions: Option<Vec<DeploymentCondition>>,
    pub observed_generation: u32,
    pub replicas: u32,
    pub unavailable_replicas: u32,
    pub updated_replicas: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Scale {
    pub spec: ScaleSpec,
    pub metadata: Metadata,
    // pub status: Option<ScaleStatus>,
}

impl Scale {
    pub(crate) fn replicas(namespace: &str, name: &str, count: u32) -> Scale {
        Scale {
            spec: ScaleSpec { replicas: count },
            metadata: Metadata {
                name: Some(name.to_owned()),
                namespace: Some(namespace.to_owned()),
                ..Default::default() }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ScaleSpec {
    pub replicas: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DeploymentList {
    items: Vec<Deployment>,
}

impl Deployment {
    pub fn new(name: &str) -> Deployment {
        let spec = DeploymentSpec::default();
        let metadata = Metadata{ name: Some(name.to_owned()), ..Default::default() };
        Deployment { spec, metadata, status: None }
    }
}

impl Resource for Deployment {
    fn kind() -> Kind { Kind::Deployment }
    fn api() -> &'static str {
        "/apis/extensions/v1beta1"
    }
}

impl ListableResource for Deployment {
    type ListResponse = DeploymentList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}