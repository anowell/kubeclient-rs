use super::*;

pub(crate) static NODE_INFO: KindInfo = KindInfo {
    plural: "nodes",
    default_namespace: None,
    api: V1_API,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Node {
    pub spec: NodeSpec,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NodeSpec {
    #[serde(rename = "podCIDR")]
    pub pod_cidr: Option<String>,
    #[serde(rename = "providerID")]
    pub provider_id: Option<String>,
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
}

impl ListableResource for Node {
    type ListResponse = NodeList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}