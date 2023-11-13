# The FastEdge Rust SDK

The Rust SDK is used to build FastEdge applications in Rust.

Example of usage:

```rust
// lib.rs
use anyhow::Result;
use fastedge::http::{Request, Response, StatusCode};
use fastedge::body::Body;

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
     Response::builder().status(StatusCode::OK).body(Body::empty())
}
```

The important things to note in the function above:

- the `fastedge::http` macro — this marks the function as the
  entrypoint for the FastEdge application
- the function signature — `fn main(req: Request<Body>) -> Result<Response<Body>>` —
  uses the HTTP objects from the popular Rust crate
  [`http`](https://crates.io/crates/http)
