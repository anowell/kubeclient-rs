The beginnings of a Rust kubernetes client

It's still very experimental and rough aruond the edges.
It has basic support for many common operations, but is far from complete.
An documentation is still very lacking.

If there is a specific API you need feel free to open an issue and I might try to bump it up in priority.
Alternatively, consider submitting a PR.

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
