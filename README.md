An ergonomic Kubernetes API client to manage Kubernetes resources

[![Crates.io](https://img.shields.io/crates/v/kubeclient.svg?maxAge=2592000)](https://crates.io/crates/kubeclient)

## Documentation

[docs.rs/kubeclient](http://docs.rs/kubeclient)

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

This client is still very incomplete, so expect to file issues and PRs to
unblock yourself if you actually take this crate as a dependency.

It has basic support for many common operations, namely the ones I've personally needed,
but I'm not yet using this library in production, so it's not very high priority for me.
That said, I will commit to discussing issues and reviewing PRs in a timely manner.