⏮️ Back to Rust [README.md](../../README.md)

# Fetch

A minimal example demonstrating outbound HTTP requests using the [WASI-HTTP](https://github.com/WebAssembly/wasi-http) interface via the [`wstd`](https://crates.io/crates/wstd) crate.

Unlike the other HTTP examples that use the synchronous FastEdge SDK, this example uses the WASI component model with an **async** handler and a proper HTTP client (`wstd::http::Client`).

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

### Prerequisites

- Rust toolchain
- [`cargo-component`](https://github.com/bytecodealliance/cargo-component)

```bash
cargo install cargo-component
```

### Compile

```bash
cargo component build --release
```

The compiled component will be at:
```
target/wasm32-wasip1/release/fetch.wasm
```

## Key differences from FastEdge SDK examples

| | FastEdge SDK | This example (WASI-HTTP) |
|---|---|---|
| Handler | `fn main(req)` — sync | `async fn main(req)` — async |
| Macro | `#[fastedge::http]` | `#[wstd::http_server]` |
| Outbound HTTP | `fastedge::send_request(req)` | `Client::new().send(req).await` |
| Build tool | `cargo build` | `cargo component build` |
