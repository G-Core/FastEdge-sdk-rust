# Getting Started with FastEdge Rust SDK

Build and deploy edge computing applications that compile to WebAssembly using the `fastedge` crate.

## Prerequisites

- Rust toolchain (stable)
- `wasm32-wasip1` target — required for the sync handler path
- `wasm32-wasip2` target — required for the async WASI handler path

Install both targets:

```bash
rustup target add wasm32-wasip1
rustup target add wasm32-wasip2
```

## Create a New Project

Create a new library crate:

```bash
cargo new --lib my-edge-app
cd my-edge-app
```

The crate must be compiled as a `cdylib`. Add to `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]
```

## Option A: Async Handler (Recommended)

The async handler uses the standard WASI-HTTP interface via the [`wstd`](https://crates.io/crates/wstd) crate. This path supports `async`/`await` and a full HTTP client.

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
wstd   = "0.6"
anyhow = "1.0"

[lib]
crate-type = ["cdylib"]
```

Write the handler in `src/lib.rs`:

```rust,no_run
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let url = request.uri().to_string();

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain;charset=UTF-8")
        .body(Body::from(format!("Hello, you made a request to {url}")))?)
}
```

Build targeting `wasm32-wasip2`:

```bash
cargo build --target wasm32-wasip2 --release
```

To avoid passing `--target` on every build, add a `.cargo/config.toml` to your project:

```toml
[build]
target = "wasm32-wasip2"
```

Then `cargo build --release` is sufficient.

The compiled `.wasm` file is written to `target/wasm32-wasip2/release/`.

## Option B: Basic Sync Handler

The sync handler uses the `fastedge` crate directly. It is synchronous and suited for simple request/response processing where `async` is not required.

Add dependencies to `Cargo.toml`:

```toml
[dependencies]
fastedge = "0.3"
anyhow   = "1.0"

[lib]
crate-type = ["cdylib"]
```

Write the handler in `src/lib.rs`:

```rust,no_run
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let url = req.uri().to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain;charset=UTF-8")
        .body(Body::from(format!("Hello, you made a request to {url}")))
        .map_err(Into::into)
}
```

Build targeting `wasm32-wasip1`:

```bash
cargo build --target wasm32-wasip1 --release
```

The compiled `.wasm` file is written to `target/wasm32-wasip1/release/`.

## Build

| Handler path            | Build command                                  |
| ----------------------- | ---------------------------------------------- |
| Async (`wstd`)          | `cargo build --target wasm32-wasip2 --release` |
| Sync (`fastedge::http`) | `cargo build --target wasm32-wasip1 --release` |

Both commands produce a `.wasm` binary in the respective `target/<target>/release/` directory.

## Feature Flags

| Feature      | Default | Description                               |
| ------------ | ------- | ----------------------------------------- |
| `proxywasm`  | yes     | Enable ProxyWasm compatibility layer      |
| `json`       | no      | Enable JSON body support via `serde_json` |

Enable the `json` feature in `Cargo.toml`:

```toml
[dependencies]
fastedge = { version = "0.3", features = ["json"] }
```

## Next Steps

Once your handler compiles, you can extend it with outbound HTTP and platform host services:

- **Outbound HTTP** — call backend services using `fastedge::send_request` (sync) or `wstd::http::Client` (async) — see [SDK_API.md](SDK_API.md)
- **Key-Value Storage** — read and write persistent data via `fastedge::key_value::Store`
- **Secrets** — retrieve encrypted credentials via `fastedge::secret::get`
- **Dictionary** — read static configuration via `fastedge::dictionary::get`

See [HOST_SERVICES.md](HOST_SERVICES.md) for key-value, secrets, dictionary, and utilities.

## See Also

- [SDK_API.md](SDK_API.md) — Core API: handler macros, Body type, outbound HTTP, error handling
- [HOST_SERVICES.md](HOST_SERVICES.md) — Key-value storage, secrets, dictionary, utilities
- [INDEX.md](INDEX.md) — Documentation index
