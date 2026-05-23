[← Back to examples](../../../README.md)

# Hello World (WASI)

The simplest possible async FastEdge application — echoes the full request URI in the response body.

Demonstrates the `#[wstd::http_server]` entry-point macro and the async handler signature used by all WASI HTTP examples.

## What it returns

```
HTTP/1.1 200 OK
content-type: text/plain;charset=UTF-8

Hello, you made a wasi request to http://<host>/<path>?<query>
```

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip2/release/hello_world.wasm
```

## APIs used

- `#[wstd::http_server]` — WASI HTTP entry-point macro
- `wstd::http::{Request, Response}` — request/response types
- `wstd::http::body::Body` — response body construction
- `request.uri().to_string()` — full absolute request URI
