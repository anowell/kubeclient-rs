extern crate kubeclient;
use std::env;
use kubeclient::prelude::*;
use kubeclient::errors::*;

fn get_secret(name: &str) -> Result<bool> {
    let kube = Kubernetes::load_conf("admin.conf")?;

    if kube.healthy()? {
        let __secret = kube.secrets().get(name)?;
        return Ok(true)
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
        Ok(s)  => println!("The secret {} exists {}", secret_name, s),
        Err(e) => println!("Error: {}", e),
    }
}
