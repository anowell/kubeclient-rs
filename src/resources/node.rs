use super::*;
use k8s_api::api::core::v1::{NodeSpec, NodeStatus};
use k8s_api::apimachinery::pkg::apis::meta::v1::ObjectMeta;

pub(crate) static NODE_INFO: KindInfo = KindInfo {
    plural: "nodes",
    default_namespace: None,
    api: V1_API,
};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Node {
    /// Spec defines the behavior of a node. https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    pub spec: NodeSpec,

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#metadata
    pub metadata: ObjectMeta,

    /// Most recently observed status of the node. Populated by the system. Read-only.
    /// More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<NodeStatus>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NodeList {
    items: Vec<Node>,
}

impl Node {
    pub fn new(name: &str) -> Node {
        let metadata = ObjectMeta{ name: Some(name.to_owned()), ..Default::default() };
        Node { metadata, ..Default::default() }
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
