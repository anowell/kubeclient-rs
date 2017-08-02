The beginnings of a Rust kubernetes client

This is highly targeted at my specific usecase right now, and will probably break with every change for now,
but I'm open to discussion, so please file an issue if you have any thoughts.

## Usage

For now...

```rust
let kube = KubeClient::new("admin.conf");

let health = kube.health()?;
if health == "ok" {
  if !kube.exists("pods", "my-pod-name")? {
    let output: Value = kube.apply("pods", get_my_pod_spec())?
    // ...
  }
} else {
  println!("Health: {}", health);
}
```
