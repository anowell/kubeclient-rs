use super::Kubernetes;
use resources::*;
use errors::*;
use std::marker::PhantomData;


pub struct KubeClient<R> {
    pub(super) kube: Kubernetes,
    pub(super) _marker: PhantomData<R>,
}

impl<R> KubeClient<R> {
    pub fn namespace(&self, namespace: &str) -> Self {
        KubeClient { kube: self.kube.namespace(namespace), _marker: PhantomData }
    }
}

pub trait ReadClient  {
    type R;
    fn exists(&self, name: &str) -> Result<bool>;
    fn get(&self, name: &str) -> Result<Self::R>;
}

pub trait WriteClient {
    type R;
    fn create(&self, resource: &Self::R) -> Result<Self::R>;
    fn delete(&self, name: &str) -> Result<()>;
}

pub trait ListClient {
    type R;
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
        self.kube.list(query)
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
