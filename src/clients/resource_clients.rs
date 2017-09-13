use super::Kubernetes;
use resources::*;
use errors::*;
use std::marker::PhantomData;
use super::ResourceRoute;


pub struct KubeClient<R> {
    pub(super) kube: Kubernetes,
    pub(super) _marker: PhantomData<R>,
}

impl<R> KubeClient<R> {
    /// Get a kubernetes client that uses a specific namespace
    pub fn namespace(&self, namespace: &str) -> Self {
        KubeClient { kube: self.kube.namespace(namespace), _marker: PhantomData }
    }
}

// impl KubeClient<Pod> {
//     // FIXME_FOR_BEER: exec requires SPD upgrade. Here are a few relevant issues
//     // https://stackoverflow.com/questions/37349440/upgrade-request-required-when-running-exec-in-kubernetes#37396806
//     // https://github.com/kubernetes-incubator/client-python/issues/58
//     pub fn exec(&self, pod_name: &str, exec: PodExec) -> Result<String> {
//         let resource = format!("{}/exec", pod_name);
//         let mut route = ResourceRoute::new(Pod::api(), Pod::kind().route(), &resource);
//         if let Some(ns) = self.kube.get_ns::<Pod>() {
//             route.namespace(ns);
//         }
//         route.query(exec.as_query_pairs());

//         let url = route.build(&self.kube.low_level.base_url)?;
//         println!("URL: {}", url);
//         let resp = self.kube.low_level.http_get(url)?;
//         println!("EXEC: {:#?}", resp);
//         Ok("FIXME".to_owned())
//     }
// }

impl KubeClient<Deployment> {
    /// Scale a deployment to a specific number of pods
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// kube.deployments().scale("web-server", 7)?;
    /// ```
    pub fn scale(&self, deployment_name: &str, count: u32) -> Result<Scale> {
        let resource = format!("{}/scale", deployment_name);
        let mut route = ResourceRoute::new(Deployment::api(), Deployment::kind().plural, &resource);
        let ns = self.kube.get_ns::<Deployment>().expect("Namespace necessary for kubernetes scale operation");
        route.namespace(ns);

        let url = route.build(&self.kube.low_level.base_url)?;

        let body = Scale::replicas(&ns, &deployment_name, count);
        let resp = self.kube.low_level.http_put_json(url, &body)?;
        Ok(resp)
    }
}

pub trait ReadClient  {
    type R;
    /// Indicates whether or not the named resource exists in the Kubernetes cluster
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
    fn exists(&self, name: &str) -> Result<bool>;

    /// Gets the named resource
    ///
    /// This is similar to the `kubectl describe` CLI commands.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// let cfg_map = kube.config_maps().get("my-config-map")?;
    /// ```
    fn get(&self, name: &str) -> Result<Self::R>;
}

pub trait WriteClient {
    type R;

    /// Creates the named resource
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
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// let mut cfg_map = ConfigMap::new("stage-config")?;
    /// cfg_map.insert("environment", "production");
    /// // Note: using the `config_maps()` builder here is optional since resource type can be inferred
    /// let response = kube.config_maps().create(&cfg_map)?;
    /// ```
    fn create(&self, resource: &Self::R) -> Result<Self::R>;

    /// Deleteds the named resource
    ///
    /// This is similar to the `kubectl delete` CLI commands.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// kube.config_maps().delete("my-config-map")?;
    /// ```
    fn delete(&self, name: &str) -> Result<()>;
}

pub trait ListClient {
    type R;
    /// Lists resources of a particular type
    ///
    /// This is similar to the `kubectl get` CLI commands.
    ///
    /// The `query` paramater allows for customizing the list request, e.g.,
    ///   setting a timeout or specifying a label selector for filtering results.
    ///
    /// ## Examples
    ///
    /// ```no_run
    /// # use kubeclient::prelude::*;
    /// let kube = Kubernetes::load_conf("admin.conf")?;
    /// let cfg_maps = kube.config_maps().list("my-config-map", None)?;
    /// ```
    fn list(&self, query: Option<&ListQuery>) -> Result<Vec<Self::R>>;
}

impl<R: Resource> ReadClient for KubeClient<R> {
    type R = R;

    fn exists(&self, name: &str) -> Result<bool> {
        self.kube.exists::<Self::R>(name)
    }
    fn get(&self, name: &str) -> Result<Self::R> {
        self.kube.get::<Self::R>(name)
    }
}

impl<R: ListableResource> ListClient for KubeClient<R> {
    type R = R;

    fn list(&self, query: Option<&ListQuery>) -> Result<Vec<Self::R>> {
        self.kube.list::<Self::R>(query)
    }

}

impl<R: Resource> WriteClient for KubeClient<R> {
    type R = R;

    fn create(&self, resource: &Self::R) -> Result<Self::R> {
        self.kube.create(resource)
    }

    fn delete(&self, name: &str) -> Result<()> {
        self.kube.delete::<Self::R>(name)
    }
}
