#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;
#[macro_use] extern crate serde_json;

extern crate base64;
extern crate chrono;
extern crate openssl;
extern crate reqwest;
extern crate serde;
extern crate serde_yaml;
extern crate url;
extern crate url_serde;
extern crate walkdir;

pub mod errors;
pub mod config;
pub mod clients;
pub mod resources;

pub mod prelude {
    pub use clients::{Kubernetes, ReadClient, WriteClient, ListClient};
}

pub use clients::Kubernetes;
pub use config::KubeConfig;
pub use errors::Error;
