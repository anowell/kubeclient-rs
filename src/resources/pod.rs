use super::*;
use std::net::Ipv4Addr;
use serde_json::Value;

pub(crate) static POD_INFO: KindInfo = KindInfo {
    plural: "pods",
    default_namespace: Some("default"),
    api: V1_API,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Pod {
    pub spec: PodSpec,
    pub metadata: Metadata,
    pub status: Option<PodStatus>,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PodSpec {
    pub active_deadline_seconds : Option<u32>,
    pub containers: Option<Vec<Value>>, // TODO: struct Container
    pub dns_policy: Option<String>,
    #[serde(rename = "hostIPC")]
    pub host_ipc: Option<bool>,
    pub host_network: Option<bool>,
    #[serde(rename = "hostPID")]
    pub host_pid: Option<bool>,
    pub hostname: Option<String>,
    pub image_pull_secrets: Option<Vec<Value>>, // TODO: struct LocalObjectReference
    pub node_name: Option<String>,
    pub node_selector: Option<BTreeMap<String, String>>,
    pub restart_policy: Option<String>,
    pub security_context: Option<Value>, // TODO: struct PodSecurityContext
    pub service_account: Option<String>,
    pub service_account_name: Option<String>,
    pub subdomain: Option<String>,
    pub termination_grace_period_seconds: Option<u32>,
    pub volumes: Option<Vec<Value>>, // TODO: struct Volume
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct PodStatus {
    pub conditions: Option<Vec<Value>>, // TODO: PodConditions type
    pub container_statuses: Option<Vec<Value>>, // TODO: ContainerStatus
    #[serde(rename = "hostIP")]
    pub host_ip: Option<String>,
    pub message: Option<String>,
    pub phase: Option<String>,
    #[serde(rename = "podIP")]
    pub pod_id: Option<Ipv4Addr>,
    pub reason: Option<String>,
    // pub start_time: chrono RFC 3339 tolerant date
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct PodList {
    items: Vec<Pod>,
}

#[derive(Serialize, Debug, Default)]
pub struct PodExec {
    stdin: Option<bool>,
    stdout: Option<bool>,
    stderr: Option<bool>,
    tty: Option<bool>,
    container: Option<String>,
    command: Option<Vec<String>>,
}

impl PodExec {
    pub fn tty(mut self) -> PodExec {
        self.tty = Some(true);
        self
    }

    pub fn command(mut self, command: Vec<String>) -> PodExec {
        self.command = Some(command);
        self
    }

    pub fn as_query_pairs(&self) -> BTreeMap<&'static str, String> {
        let mut query = BTreeMap::new();
        if let Some(stdin) = self.stdin {
            query.insert("stdin", stdin.to_string());
        }
        if let Some(stdout) = self.stdout {
            query.insert("stdout", stdout.to_string());
        }
        if let Some(tty) = self.tty {
            query.insert("tty", tty.to_string());
        }
        if let Some(ref container) = self.container {
            query.insert("container", container.to_owned());
        }
        if let Some(ref command) = self.command {
            query.insert("command", command.join(" "));
        }
        query
    }
}

impl Pod {
    pub fn new(name: &str) -> Pod {
        let spec = PodSpec::default();
        let metadata = Metadata{ name: Some(name.to_owned()), ..Default::default() };
        Pod { spec, metadata, status: None }
    }
}

impl Resource for Pod {
    fn kind() -> Kind { Kind::Pod }
}


impl ListableResource for Pod {
    type ListResponse = PodList;
    fn list_items(response: Self::ListResponse) -> Vec<Self> {
        response.items
    }
}