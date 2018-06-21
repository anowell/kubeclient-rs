use super::*;
use std::collections::BTreeMap;
use k8s_api::apimachinery::pkg::apis::meta::v1::ObjectMeta;

pub(crate) static CONFIG_MAP_INFO: KindInfo = KindInfo {
    plural: "configmaps",
    default_namespace: Some("default"),
    api: V1_API,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigMap {
    /// Data contains the configuration data. Each key must consist of alphanumeric characters, '-', '_' or '.'.
    data: BTreeMap<String, String>,

    /// Standard object's metadata. More info: https://git.k8s.io/community/contributors/devel/api-conventions.md#metadata
    metadata: ObjectMeta,
}

impl ConfigMap {
    pub fn new(name: &str) -> ConfigMap {
        let data = BTreeMap::new();
        let metadata = ObjectMeta{ name: Some(name.to_owned()), ..Default::default() };
        ConfigMap { data, metadata }
    }

    pub fn insert<K, V>(&mut self, name: K, data: V) -> &mut ConfigMap
    where K: Into<String>,
          V: Into<String>,
    {
        self.data.insert(name.into(), data.into());
        self
    }

    pub fn append<M, K, V>(&mut self, map: M) -> &mut ConfigMap
    where K: Into<String>,
          V: Into<String>,
          M: IntoIterator<Item=(K, V)>
    {
        let mut encoded_map = map.into_iter()
            .map(|(k,v)| (k.into(), v.into()))
            .collect();
        self.data.append(&mut encoded_map);
        self
    }
}

impl Resource for ConfigMap {
    fn kind() -> Kind { Kind::ConfigMap }
}
