//! An ergonomic Kubernetes API client to manage kubernetes resources
//!
//! **Disclaimer**: This crate is still super very incomplete in functionality.
//! So expect to file issues and PRs to unblock yourself if you actually
//! take this crate as a dependency.
//!
//! ## Basic Usage
//!
//! The `prelude` contains several the main [`Kubernetes`](clients/struct.Kubernetes.html) type
//!   as well as several traits that expose the resource-specific methods for reading and writing
//!   kubernetes resources.
//!
//! ```no_run
//! use kubeclient::prelude::*;
//!
//! let kube = Kubernetes::load_conf("admin.conf")?;
//!
//! if kube.healthy()? {
//!   if !kube.secrets().exists("my-secret")? {
//!     let output = kube.secrets().get("my-secret")?
//!     // ...
//!   }
//!
//!   for node in kube.nodes().list()? {
//!     println!("Found node: {}", node.metadata.name.unwrap());
//!   }
//! }
//! ```

#[macro_use] extern crate error_chain;
#[macro_use] extern crate serde_derive;

extern crate base64;
extern crate chrono;
extern crate headers_ext;
extern crate openssl;
extern crate k8s_openapi;
extern crate reqwest;
extern crate serde;
extern crate serde_json;
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

use k8s_openapi::v1_9 as k8s_api;
