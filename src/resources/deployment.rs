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
    pub available_replicas: u32,
    // pub conditions: DeploymentCondition,
    pub observed_generation: u32,
    pub replicas: u32,
    pub unavailable_replicas: u32,
    pub updated_replicas: u32,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct DeploymentList {
    items: Vec<Deployment>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct DeploymentListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    field_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_seconds: Option<String>,
}

impl DeploymentListQuery {
    pub fn field_selector<S: Into<String>>(&self, field_selector: S) -> Self {
        let mut new = self.clone();
        new.field_selector = Some(field_selector.into());
        new
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
    type QueryParams = DeploymentListQuery;
    type ListResponse = DeploymentList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}