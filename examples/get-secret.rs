extern crate kubeclient;
use std::env;
use kubeclient::prelude::*;
use kubeclient::errors::*;

fn get_secret(name: &str) -> Result<bool> {
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
        let __secret = kube.secrets().get(name)?;
        return Ok(true);
    }

    Ok(false)
}

fn main() {
    let args: Vec<_> = env::args().collect();
    let secret_name = match args.len() > 1 {
        true => &args[1],
        false => "secret1",
    };

    match get_secret(secret_name) {
        Ok(s) => println!("The secret {} exists {}", secret_name, s),
        Err(e) => println!("Error: {}", e),
    }
}
