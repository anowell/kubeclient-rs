extern crate kubeclient;
use kubeclient::prelude::*;
use kubeclient::errors::*;
use std::env;

fn run_list_pods() -> Result<i32> {
    // filename is set to $KUBECONFIG if the env var is available.
    // Otherwise it falls back to "admin.conf".
    let filename = env::var("KUBECONFIG").ok();
    let filename = filename
        .as_ref()
        .map(String::as_str)
        .and_then(|s| if s.is_empty() { None } else { Some(s) })
        .unwrap_or("admin.conf");
    let kube = Kubernetes::load_conf(filename)?;

    if kube.healthy()? {
        for pod in kube.pods().namespace("default").list(None)? {
            println!("found pod: {:?}", pod);
        }
    }

    Ok(0)
}

fn main() {
    match run_list_pods() {
        Ok(n) => println!("Success error code is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}
