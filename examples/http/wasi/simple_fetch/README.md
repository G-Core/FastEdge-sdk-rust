[← Back to examples](../../../README.md)

# Simple Fetch

A minimal example demonstrating outbound HTTP requests using the [WASI-HTTP](https://github.com/WebAssembly/wasi-http) interface via the [`wstd`](https://crates.io/crates/wstd) crate.

Uses the WASI component model with an **async** handler and a proper HTTP client (`wstd::http::Client`). The same async pattern is used by all examples in `examples/http/wasi/`.

## How it works

The app receives an incoming request, reads the target URL from the `x-fetch-url` header, makes an outbound GET request to that URL, and streams the response back to the caller.

If the `x-fetch-url` header is absent, it defaults to `https://httpbin.org/get`.

## Request headers

| Header | Required | Description |
|--------|----------|-------------|
| `x-fetch-url` | No | URL to fetch. Defaults to `https://httpbin.org/get` |

## Example

```bash
curl -H "x-fetch-url: https://httpbin.org/uuid" https://<your-app-domain>/
```

## Build

```bash
cargo build --release
# Output: target/wasm32-wasip2/release/simple_fetch.wasm
```

## Key differences from basic HTTP examples

| | Basic HTTP (`fastedge` crate) | WASI HTTP (`wstd` crate) |
|---|---|---|
| Handler | `fn main(req)` — sync | `async fn main(req)` — async |
| Macro | `#[fastedge::http]` | `#[wstd::http_server]` |
| Outbound HTTP | `fastedge::send_request(req)` | `Client::new().send(req).await` |
| Build target | `wasm32-wasip1` | `wasm32-wasip2` |
