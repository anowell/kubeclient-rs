#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;

extern crate base64;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
extern crate serde_yaml;
extern crate openssl;
extern crate url;
extern crate url_serde;

pub mod errors;
pub mod config;
pub mod client;

pub use client::KubeClient;
pub use config::KubeConfig;
pub use errors::Error;
