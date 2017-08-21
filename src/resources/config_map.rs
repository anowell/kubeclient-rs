use super::*;
use std::collections::BTreeMap;

pub(crate) static CONFIG_MAP_INFO: KindInfo = KindInfo {
    plural: "configmaps",
    default_namespace: Some("default"),
    api: V1_API,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct ConfigMap {
    data: BTreeMap<String, String>,
    metadata: Metadata,
}

impl ConfigMap {
    pub fn new(name: &str) -> ConfigMap {
        let data = BTreeMap::new();
        let metadata = Metadata{ name: Some(name.to_owned()), ..Default::default() };
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