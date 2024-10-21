# FastEdge Rust SDK

This is the Rust SDK for building applications ready for deploying on FastEdge runtime.
FastEdge Runtime SDK is a simple SDK that helps you to create edge cloud application using [WebAssembly component model](https://github.com/WebAssembly/component-model)
and [Wasmtime](https://wasmtime.dev/) runtime.

## Getting Started

Please read through the documentation provided by Gcore.

- FastEdge Overview: [Getting Started](https://gcore.com/docs/fastedge/getting-started)
- Create a FastEdge App: [Stage 1](https://gcore.com/docs/fastedge/getting-started/create-fastedge-apps#stage-1-create-a-wasm-binary-file)
- Deploying an App:
  [Stage 2](https://gcore.com/docs/fastedge/getting-started/create-fastedge-apps#stage-2-deploy-an-app)

## Language Support

The table below summarizes the feature support for language SDKs.

| Feature       | Rust      | JavaScript |
|---------------|-----------|------------|
| **Handlers**  |           |            |
| HTTP          | Supported | Supported  |
| **APIs**      |           |            |
| Outbound HTTP | Supported | Supported  |
| Env Variables | Supported | Supported  |

## Rust toolchain setup:
- `rustup target add wasm32-wasip1`

# The FastEdge Rust SDK

Example of simple app with http entrypoint:

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

