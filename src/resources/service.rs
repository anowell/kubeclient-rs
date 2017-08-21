use super::*;
use serde_json::Value;

pub(crate) static SERVICE_INFO: KindInfo = KindInfo {
    plural: "services",
    default_namespace: Some("default"),
    api: V1_API,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Service {
    pub spec: ServiceSpec,
    pub metadata: Metadata,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServiceSpec {
    #[serde(rename = "clusterIP")]
    pub cluster_ip: Option<String>,
    #[serde(rename = "externalIPs")]
    pub external_ips: Option<Vec<String>>,
    pub external_name: Option<String>,
    #[serde(rename = "loadBalancerIP")]
    pub load_balancer_ip: String,
    pub load_balancer_source_ranges: Option<Vec<String>>,
    pub ports: Option<Vec<Value>>, // TODO: ServicePort type
    pub selector: Option<BTreeMap<String, String>>,
    pub session_affinity: Option<String>,
    #[serde(rename = "type")]
    pub service_type: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServiceStatus {
    pub load_balancer: Option<Value>, // TODO: LoadBalancerStatus type
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ServiceList {
    items: Vec<Service>,
}

impl Service {
    pub fn new(name: &str) -> Service {
        let spec = ServiceSpec::default();
        let metadata = Metadata{ name: Some(name.to_owned()), ..Default::default() };
        Service { spec, metadata }
    }
}

impl Resource for Service {
    fn kind() -> Kind { Kind::Service }
}


impl ListableResource for Service {
    type ListResponse = ServiceList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}