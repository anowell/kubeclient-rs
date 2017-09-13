An ergonomic Kubernetes API client to manage Kubernetes resources

[Documentation](http://docs.rs/kubeclient)

[![Crates.io](https://img.shields.io/crates/v/kubeclient.svg?maxAge=2592000)](https://crates.io/crates/kubeclient)


## Usage

Basic usage looks like this:

```rust
use kubeclient::prelude::*;

let kube = Kubernetes::load_conf("admin.conf")?;

if kube.healthy()? {
  if !kube.secrets().exists("my-secret")? {
    let output = kube.secrets().get("my-secret")?
    // ...
  }

  for node in kube.nodes().list()? {
    println!("Found node: {}", node.metadata.name);
  }
}
```

## Status

This client is still very experimental and rough aruond the edges.
It has basic support for many common operations, namely the ones I've personally
needed up to this point, but is far from complete, and documentation is still lacking.

If there is a specific API you need feel free to open an issue or submit a PR.

