# fastedge-derive

Procedural macros for the [FastEdge Rust SDK](https://docs.rs/fastedge).

This crate provides the `#[fastedge::http]` attribute macro for creating WebAssembly HTTP handlers for edge computing applications.

## Overview

The `#[fastedge::http]` macro transforms a regular Rust function into a WebAssembly component that can process HTTP requests in the FastEdge runtime.

## Usage

This crate is typically used through the main `fastedge` crate and is not used directly.

```rust
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Hello, FastEdge!"))
        .map_err(Into::into)
}
```

## Requirements

The handler function must:
- Accept a single parameter of type `Request<Body>`
- Return `Result<Response<Body>>` (typically using `anyhow::Result`)
- Be a free function (not a method)

## Documentation

See the [fastedge crate documentation](https://docs.rs/fastedge) for complete usage information and examples.

## License

Licensed under the Apache License, Version 2.0.
