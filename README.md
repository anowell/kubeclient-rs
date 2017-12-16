An ergonomic Kubernetes API client to manage Kubernetes resources

[![Crates.io](https://img.shields.io/crates/v/kubeclient.svg?maxAge=2592000)](https://crates.io/crates/kubeclient)

## Documentation

[docs.rs/kubeclient](http://docs.rs/kubeclient)

## Usage

You can find out about the basic usage in [examples](/examples).

```
# Ensure you have a valid kubeconfig in admin.conf

## Get secret
cargo run --example get-secret secret123
[...]

## List nodes
cargo run --example list-nodes
[...]

```

## Status

This client is still very incomplete, so expect to file issues and PRs to
unblock yourself if you actually take this crate as a dependency.

It has basic support for many common operations, namely the ones I've personally needed,
but I'm not yet using this library in production, so it's not very high priority for me.
That said, I will commit to discussing issues and reviewing PRs in a timely manner.