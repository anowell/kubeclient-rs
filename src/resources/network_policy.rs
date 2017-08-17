use super::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct NetworkPolicy {
    pub spec: NetworkPolicySpec,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPolicySpec {
    // pub ingress: Option<Vec<NetworkPolicyIngressRule>>,
    // pub podSelector : Option<LabelSelector>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NetworkPolicyList {
    items: Vec<NetworkPolicy>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct NetworkPolicyListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    field_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_seconds: Option<String>,
}

impl NetworkPolicyListQuery {
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

impl NetworkPolicy {
    pub fn new(name: &str) -> NetworkPolicy {
        let spec = NetworkPolicySpec::default();
        let metadata = Metadata{ name: Some(name.to_owned()), ..Default::default() };
        NetworkPolicy { spec, metadata }
    }
}

impl Resource for NetworkPolicy {
    fn kind() -> Kind { Kind::NetworkPolicy }
    fn api() -> &'static str {
        "/apis/extensions/v1beta1"
    }
}


impl ListableResource for NetworkPolicy {
    type QueryParams = NetworkPolicyListQuery;
    type ListResponse = NetworkPolicyList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}