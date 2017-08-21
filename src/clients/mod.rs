mod low_level;
mod resource_clients;

pub use self::resource_clients::*;
use self::low_level::*;

use std::path::Path;
use resources::*;
use serde_json::Value;
use errors::*;
use std::marker::PhantomData;


#[derive(Clone)]
pub struct Kubernetes {
    pub(crate) low_level: KubeLowLevel,
    namespace: Option<String>,
}

impl Kubernetes {
    pub fn load_conf<P: AsRef<Path>>(path: P) -> Result<Kubernetes> {
        Ok(Kubernetes{
            low_level: KubeLowLevel::load_conf(path)?,
            namespace: None,
        })
    }

    pub fn config_maps(&self) -> KubeClient<ConfigMap> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    pub fn deployments(&self) -> KubeClient<Deployment> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    pub fn network_policies(&self) -> KubeClient<NetworkPolicy> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    pub fn nodes(&self) -> KubeClient<Node> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    pub fn pods(&self) -> KubeClient<Pod> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    pub fn secrets(&self) -> KubeClient<Secret> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }

    pub fn services(&self) -> KubeClient<Service> {
        KubeClient { kube: self.clone(), _marker: PhantomData }
    }


    pub fn namespace(&self, namespace: &str) -> Kubernetes {
        Kubernetes { low_level: self.low_level.clone(), namespace: Some(namespace.to_owned()) }
    }

    pub fn healthy(&self) -> Result<bool> {
        Ok(self.low_level.health()? == "ok")
    }

    pub fn apply_path<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let _: Vec<Value> = self.low_level.apply_path(path)?;
        Ok(())
    }

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

    pub fn create<R: Resource>(&self, resource: &R) -> Result<R> {
        let mut route = KindRoute::new(R::api(), R::kind().plural);
        if let Some(ns) = self.get_ns::<R>() {
            route.namespace(ns);
        }
        self.low_level.apply(&route, resource)
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