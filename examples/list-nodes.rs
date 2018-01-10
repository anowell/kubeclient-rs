extern crate kubeclient;
use kubeclient::prelude::*;
use kubeclient::errors::*;

fn run_main() -> Result<i32> {
    let kube = Kubernetes::load_conf("admin.conf")?;

    if kube.healthy()? {
        for node in kube.nodes().list(None)? {
            println!("found node: {:?}", node);
        }
    }

    Ok(0)
}

fn main() {
    match run_main() {
        Ok(n)  => println!("Success error code is {}", n),
        Err(e) => println!("Error: {}", e),
    }
}

