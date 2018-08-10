use super::*;
use std::collections::BTreeMap;
use base64;
use k8s_api::apimachinery::pkg::apis::meta::v1::ObjectMeta;

pub(crate) static SECRET_INFO: KindInfo = KindInfo {
    plural: "secrets",
    default_namespace: Some("default"),
    api: V1_API,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct Secret {
    data: BTreeMap<String, String>,
    metadata: ObjectMeta,
}

impl Secret {
    pub fn new(name: &str) -> Secret {
        let data = BTreeMap::new();
        let metadata = ObjectMeta{ name: Some(name.to_owned()), ..Default::default() };
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
