The beginnings of a Rust kubernetes client

This is highly targeted at my specific usecase right now, and will probably break with every change for now,
but I'm open to discussion, so please file an issue if you have any thoughts.

## Usage

For now, using it feels about like this:

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
