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
    type ListResponse = NetworkPolicyList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}