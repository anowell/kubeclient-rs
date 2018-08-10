use super::*;
use k8s_api::api::networking::v1::NetworkPolicySpec;
use k8s_api::apimachinery::pkg::apis::meta::v1::ObjectMeta;

pub(crate) static NETWORK_POLICY_INFO: KindInfo = KindInfo {
    plural: "networkpolicies",
    default_namespace: Some("default"),
    api: V1_BETA_API,
};

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct NetworkPolicy {
    /// Specification of the desired behavior for this NetworkPolicy.
    pub spec: NetworkPolicySpec,

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#metadata
    pub metadata: ObjectMeta,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct NetworkPolicyList {
    items: Vec<NetworkPolicy>,
}


impl NetworkPolicy {
    pub fn new(name: &str) -> NetworkPolicy {
        let metadata = ObjectMeta{ name: Some(name.to_owned()), ..Default::default() };
        NetworkPolicy { metadata, ..Default::default() }
    }
}

impl Resource for NetworkPolicy {
    fn kind() -> Kind { Kind::NetworkPolicy }
}


impl ListableResource for NetworkPolicy {
    type ListResponse = NetworkPolicyList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}
