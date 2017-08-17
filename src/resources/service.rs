use super::*;

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
    pub type: Option<String>,
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

#[derive(Clone, Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct ServiceListQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    field_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    label_selector: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    resource_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    timeout_seconds: Option<String>,
}

impl ServiceListQuery {
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
    type QueryParams = ServiceListQuery;
    type ListResponse = ServiceList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}