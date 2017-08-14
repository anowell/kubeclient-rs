use super::*;

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    spec: NodeSpec,
    metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NodeSpec {
    #[serde(rename = "podCIDR")]
    pod_cidr: Option<String>,
    #[serde(rename = "providerID")]
    provider_id: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NodeList {
    items: Vec<Node>,
}

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct NodeListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    field_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_seconds: Option<String>,
}

impl NodeListQuery {
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

impl Node {
    pub fn new(name: &str) -> Node {
        let spec = NodeSpec::default();
        let metadata = Metadata{ name: name.to_owned(), ..Default::default() };
        Node { spec, metadata }
    }
}

impl Resource for Node {
    fn kind() -> Kind { Kind::Node }
    fn default_namespace() -> Option<&'static str> {
        None
    }
}

impl ListableResource for Node {
    type QueryParams = NodeListQuery;
    type ListResponse = NodeList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}