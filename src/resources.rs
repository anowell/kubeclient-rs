use serde::Serialize;
use serde::de::DeserializeOwned;
use std::collections::BTreeMap;
use base64;
use std::fmt;

#[derive(Serialize, Deserialize, Debug)]
pub enum Kind { Secret, ConfigMap }

impl Kind {
    pub fn route(&self) -> &'static str {
        match *self {
            Kind::Secret => "secrets",
            Kind::ConfigMap => "configmaps",
        }
    }
}

// Debug output of Kind is exactly what we want for Display
impl fmt::Display for Kind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self, f)
    }
}

pub trait Resource: Serialize + DeserializeOwned {
    fn kind() -> Kind;
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub kind: String,
    pub api_version: String,
    pub metadata: Metadata,
    pub status: String,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    name: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Secret {
    data: BTreeMap<String, String>,
    metadata: Metadata,
}

impl Secret {
    pub fn new(name: &str) -> Secret {
        let data = BTreeMap::new();
        let metadata = Metadata{ name: Some(name.to_owned()), ..Default::default() };
        Secret { data, metadata }
    }

    pub fn insert<K,V>(&mut self, name: K, secret: V) -> &mut Secret
    where K: Into<String>,
          V: AsRef<[u8]>,
    {
        self.data.insert(name.into(), base64::encode(secret.as_ref()));
        self
    }

    pub fn append<M, K, V>(&mut self, map: M) -> &mut Secret
    where K: Into<String>,
          V: AsRef<[u8]>,
          M: IntoIterator<Item=(K, V)>
    {
        let mut encoded_map = map.into_iter()
            .map(|(k,v)| (k.into(), base64::encode(v.as_ref())))
            .collect();
        self.data.append(&mut encoded_map);
        self
    }

    pub fn get<K>(&self, name: K) -> Option<Vec<u8>>
    where K: AsRef<str>
    {
        self.data.get(name.as_ref())
            .map(|raw| base64::decode(&raw).expect("BUG: secret wasn't base64 encoded"))
    }
}

impl Resource for Secret {
    fn kind() -> Kind { Kind::Secret }
}

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