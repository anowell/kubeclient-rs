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

impl Node {
    pub fn new(name: &str) -> Node {
        let spec = NodeSpec::default();
        let metadata = Metadata{ name: Some(name.to_owned()), ..Default::default() };
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
    type ListResponse = NodeList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}