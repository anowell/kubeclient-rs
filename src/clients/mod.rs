mod low_level;
mod resource_clients;

pub use self::resource_clients::*;
use self::low_level::*;

use std::path::Path;
use resources::*;
use serde_json::Value;
use errors::*;
use std::marker::PhantomData;


/// The main type for instantiating clients for managing kubernetes resources
#[derive(Clone)]
pub struct Kubernetes {
    pub(crate) low_level: KubeLowLevel,
    namespace: Option<String>,
}

impl Kubernetes {
    /// Initialize a Kubernetes client from a Kubernets config file
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// ```
    pub fn load_conf<P: AsRef<Path>>(path: P) -> Result<Kubernetes> {
        Ok(Kubernetes{
            low_level: KubeLowLevel::load_conf(path)?,
            namespace: None,
        })
    }

    /// Get a kubernetes client for managing `ConfigMaps`
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// if kube.config_maps().exists("my-config-map")? {
    ///     println!("Found 'my-config-map'")
    /// }
    /// ```
    pub fn config_maps(&self) -> KubeClient<ConfigMap> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    /// Get a kubernetes client for managing `Deployments`
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// if kube.deployments().exists("web-server")? {
    ///     println!("Found 'web-server' deployment")
    /// }
    /// ```
    pub fn deployments(&self) -> KubeClient<Deployment> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    /// Get a kubernetes client for managing `NetworkPolicies`
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// if kube.network_policies().exists("web-server")? {
    ///     println!("Found 'web-server' network policy")
    /// }
    /// ```
    pub fn network_policies(&self) -> KubeClient<NetworkPolicy> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    /// Get a kubernetes client for managing `Nodes`
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// if kube.nodes().exists("node-123")? {
    ///     println!("Found 'node-123'")
    /// }
    /// ```
    pub fn nodes(&self) -> KubeClient<Node> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    /// Get a kubernetes client for managing `Pods`
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// if kube.pods().exists("web-server-abcdefgh12345678")? {
    ///     println!("Found 'web-server-abcdefgh12345678' pod")
    /// }
    /// ```
    pub fn pods(&self) -> KubeClient<Pod> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    /// Get a kubernetes client for managing `Secrets`
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// if kube.secrets().exists("my-secret")? {
    ///     println!("Found 'my-secret'")
    /// }
    /// ```
    pub fn secrets(&self) -> KubeClient<Secret> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    /// Get a kubernetes client for managing `Services`
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// if kube.services().exists("web-server")? {
    ///     println!("Found 'web-server' service")
    /// }
    /// ```
    pub fn services(&self) -> KubeClient<Service> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    /// Get a kubernetes client that uses a specific namespace
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// let cluster_info = kube.namespace("kube-system")
    ///     .secrets()
    ///     .get("clusterinfo")?;
    /// ```
    pub fn namespace(&self, namespace: &str) -> Kubernetes {
        Kubernetes { low_level: self.low_level.clone(), namespace: Some(namespace.to_owned()) }
    }

    /// Check to see if the Kubernetes API is healthy
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// let is_healthy = kube.healthy()?;
    /// ```
    pub fn healthy(&self) -> Result<bool> {
        Ok(self.low_level.health()? == "ok")
    }

    /// Applies a JSON or YAML resource file
    ///
    /// This is similar to the `kubectl apply` CLI commands.
    ///
    /// This may be a single file or an entire directory.
    /// If the resource(s) specified already exists, this method
    /// will NOT replace the resource, but will simply return
    /// the already existing resource.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// let is_healthy = kube.apply("web-server/deployment.yaml")?;
    /// ```
    pub fn apply<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let _: Vec<Value> = self.low_level.apply_path(path)?;
        Ok(())
    }

    /// Creates a resource from a typed resource defintion
    ///
    /// This is similar to the `kubectl create` CLI commands.
    ///
    /// **Note**: most of the resource type defintions are incomplete
    /// Pull requests to fill missing fields/types are appreciated (especially if documented).
    ///
    /// ## Examples:
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// # use kubeclient::resources::Secret;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// let mut secret = Secret::new("web-server");
    /// secret.insert("db_password", "abc123");
    /// let response = kube.create(&secret)?;
    /// ```
    pub fn create<R: Resource>(&self, resource: &R) -> Result<R> {
        let mut route = KindRoute::new(R::api(), R::kind().plural);
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        self.low_level.apply(&route, resource)
    }

    // Methods below this point are the generic resource read/write methods.
    // They are not exposed publicly, as most of them have no way to infer
    // the generic argument in typical usage, `kube.exists::<Deployment>("web-server")?`
    // is decidedly less ergonomic than `kube.deployments().exists("web-server")?`.

    fn exists<R: Resource>(&self, name: &str) -> Result<bool> {
        let mut route = ResourceRoute::new(R::api(), R::kind().plural, name);
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        self.low_level.exists(&route)
    }

    fn get<R: Resource>(&self, name: &str) -> Result<R> {
        let mut route = ResourceRoute::new(R::api(), R::kind().plural, name);
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        self.low_level.get(&route)
    }

    fn list<R: ListableResource>(&self, query: Option<&ListQuery>) -> Result<Vec<R>> {
        let mut route = KindRoute::new(R::api(), R::kind().plural);
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        if let Some(query) = query {
            route.query(query.as_query_pairs());
        }
        let response: R::ListResponse = self.low_level.list(&route)?;
        Ok(R::list_items(response))
    }

    fn delete<R: Resource>(&self, name: &str) -> Result<()> {
        let mut route = ResourceRoute::new(R::api(), R::kind().plural, name);
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        self.low_level.delete(&route)
    }

    fn get_ns<'a, R: Resource>(&'a self) -> Option<&'a str> {
        match self.namespace {
            Some(ref ns) => Some(ns),
            None => R::default_namespace(),
        }
    }
}