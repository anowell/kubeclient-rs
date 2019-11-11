use super::*;
use k8s_openapi::api::core::v1::{PodSpec, PodStatus};
use k8s_openapi::apimachinery::pkg::apis::meta::v1::ObjectMeta;

pub(crate) static POD_INFO: KindInfo = KindInfo {
    plural: "pods",
    default_namespace: Some("default"),
    api: V1_API,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Pod {
    /// Specification of the desired behavior of the pod. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    pub spec: PodSpec,

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#metadata
    pub metadata: ObjectMeta,

    /// Most recently observed status of the pod. This data may not be up to date. Populated by the system. Read-only.
    /// More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#spec-and-status
    pub status: Option<PodStatus>,
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
        let metadata = ObjectMeta{ name: Some(name.to_owned()), ..Default::default() };
        Pod { metadata, ..Default::default() }
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
