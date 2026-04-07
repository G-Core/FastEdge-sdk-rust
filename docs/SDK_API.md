# FastEdge Rust SDK — Core API

Reference for the `fastedge` crate. Covers the handler macros, body type, outbound HTTP, errors, and feature flags. For host services (key-value, secrets, dictionary), see [HOST_SERVICES.md](HOST_SERVICES.md).

---

## Quick Start

### Cargo.toml

The current crate version is defined in the repository's `Cargo.toml` under `[workspace.package]`.

```toml
[dependencies]
fastedge = "0.3"
anyhow  = "1.0"
wstd    = "*"

[lib]
crate-type = ["cdylib"]
```

### Minimal Handler

The recommended handler for new applications uses `#[wstd::http_server]` (async, WASI-HTTP):

```rust,no_run
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let response = Response::builder()
        .status(200)
        .body(Body::from("Hello, FastEdge!"))?;
    Ok(response)
}
```

### Build

For `#[wstd::http_server]` (recommended):

```bash
rustup target add wasm32-wasip2
cargo build --target wasm32-wasip2 --release
```

To avoid passing `--target` on every build, add `.cargo/config.toml` to your project:

```toml
[build]
target = "wasm32-wasip2"
```

Then `cargo build --release` is sufficient.

For `#[fastedge::http]` (basic):

```bash
rustup target add wasm32-wasip1
cargo build --target wasm32-wasip1 --release
```

The output `.wasm` file is located at `target/<target-triple>/release/<crate_name>.wasm`.

---

## Handler Macros

### `#[wstd::http_server]` (Recommended)

Provided by the [`wstd`](https://crates.io/crates/wstd) crate. Registers an async function as the HTTP request handler using the standard WASI-HTTP interface. This is the recommended handler for all new FastEdge applications.

The decorated function must match the following signature:

```rust,ignore
async fn <name>(request: Request<Body>) -> anyhow::Result<Response<Body>>
```

**Requirements:**

- Must be declared `async`.
- Accepts exactly one parameter of type `wstd::http::Request<wstd::http::body::Body>`.
- Returns `anyhow::Result<wstd::http::Response<wstd::http::body::Body>>`.
- Build target: `wasm32-wasip2`.

**Example — echo handler:**

```rust,no_run
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let method = request.method().to_string();
    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!("Method: {}", method)))?)
}
```

**Example — outbound HTTP with `wstd::http::Client`:**

```rust,no_run
use wstd::http::body::Body;
use wstd::http::{Client, Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let upstream = Request::get("https://api.example.com/data")
        .header("accept", "application/json")
        .body(Body::empty())?;

    let response = Client::new().send(upstream).await?;
    Ok(response)
}
```

### `#[fastedge::http]` (Basic)

```rust,ignore
#[proc_macro_attribute]
pub fn http(attr: TokenStream, item: TokenStream) -> TokenStream
```

Provided by the `fastedge` crate. Registers a synchronous function as the HTTP request handler using the FastEdge-specific WIT interface. Use this for applications that require synchronous execution or the `fastedge::send_request` client. New projects should prefer `#[wstd::http_server]`.

The decorated function must match the following signature:

```rust,ignore
fn <name>(req: fastedge::http::Request<fastedge::body::Body>) -> anyhow::Result<fastedge::http::Response<fastedge::body::Body>>
```

**Requirements:**

- Accepts exactly one parameter of type `Request<Body>`.
- Returns `Result<Response<Body>>`. Any `Result` whose error implements `Into<Box<dyn std::error::Error>>` (such as `anyhow::Result`) is accepted.
- The function name is not significant; `main` is conventional.
- Build target: `wasm32-wasip1`.

**Error handling:**

If the function returns `Err(e)`, the macro converts it to an HTTP `500 Internal Server Error` response with the error message as the body. No panic occurs.

**Example — minimal handler:**

```rust,no_run
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

**Example — method dispatch:**

```rust,no_run
use anyhow::{anyhow, Result};
use fastedge::body::Body;
use fastedge::http::{Method, Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    match req.method() {
        &Method::GET => Response::builder()
            .status(StatusCode::OK)
            .body(Body::from("GET OK"))
            .map_err(Into::into),
        _ => Err(anyhow!("method not allowed")),
    }
}
```

### Comparison

| Aspect              | `#[wstd::http_server]`   | `#[fastedge::http]`      |
| ------------------- | ------------------------ | ------------------------ |
| Execution model     | Async (`async fn`)       | Synchronous              |
| HTTP client         | `wstd::http::Client`     | `fastedge::send_request` |
| Body type           | `wstd::http::body::Body` | `fastedge::body::Body`   |
| Build target        | `wasm32-wasip2`          | `wasm32-wasip1`          |
| Interface standard  | WASI-HTTP (standard)     | FastEdge-specific WIT    |
| Recommendation      | New applications         | Legacy / sync required   |

---

## Body Type

```rust,ignore
pub struct Body { /* private fields */ }
```

`fastedge::body::Body` wraps [`bytes::Bytes`](https://docs.rs/bytes) and carries a MIME content-type. The content-type is set at construction time based on the input data and cannot be changed after creation.

`Body` implements `Deref<Target = bytes::Bytes>`, so all `Bytes` methods (`.len()`, `.is_empty()`, slicing, iteration) are available directly.

### Constructors

| Constructor                                  | Content-Type                | Notes                                                              |
| -------------------------------------------- | --------------------------- | ------------------------------------------------------------------ |
| `Body::from(value: String)`                  | `text/plain; charset=utf-8` |                                                                    |
| `Body::from(value: &'static str)`            | `text/plain; charset=utf-8` |                                                                    |
| `Body::from(value: Vec<u8>)`                 | `application/octet-stream`  |                                                                    |
| `Body::from(value: &'static [u8])`           | `application/octet-stream`  |                                                                    |
| `Body::empty()`                              | `text/plain; charset=utf-8` | Zero-length body                                                   |
| `Body::try_from(value: serde_json::Value)`   | `application/json`          | Requires `json` feature; returns `Result<Body, serde_json::Error>` |

```rust
use fastedge::body::Body;

let text  = Body::from("hello");
let owned = Body::from(String::from("hello"));
let bytes = Body::from(vec![0x48u8, 0x69]);
let empty = Body::empty();
```

```rust
// json feature required
use fastedge::body::Body;
use serde_json::json;

# fn main() -> Result<(), serde_json::Error> {
let body = Body::try_from(json!({"status": "ok"}))?;
assert_eq!(body.content_type(), "application/json");
# Ok(())
# }
```

### Methods

| Method                            | Return Type | Description                                         |
| --------------------------------- | ----------- | --------------------------------------------------- |
| `content_type(&self) -> String`   | `String`    | Returns the MIME type set when the body was created |
| `empty() -> Self`                 | `Body`      | Constructs a zero-length body                       |

All methods from `bytes::Bytes` are available via `Deref`:

```rust
use fastedge::body::Body;

let body = Body::from("hello");
assert_eq!(body.len(), 5);
assert!(!body.is_empty());
let slice: &[u8] = &body[..];
```

### Content-Type Detection

Content-type is determined at construction time and cannot be changed after creation.

| Input type           | Resulting content-type      |
| -------------------- | --------------------------- |
| `String` / `&str`    | `text/plain; charset=utf-8` |
| `Vec<u8>` / `&[u8]`  | `application/octet-stream`  |
| `serde_json::Value`  | `application/json`          |
| `Body::empty()`      | `text/plain; charset=utf-8` |

To send a response with a content-type that does not match automatic detection, set the `Content-Type` header explicitly on the response builder:

```rust
use fastedge::body::Body;
use fastedge::http::{Response, StatusCode};

let html = "<h1>Hello</h1>";
let response = Response::builder()
    .status(StatusCode::OK)
    .header("content-type", "text/html; charset=utf-8")
    .body(Body::from(html))
    .unwrap();
```

---

## Outbound HTTP

### `send_request`

```rust,ignore
pub fn send_request(req: http::Request<Body>) -> Result<http::Response<Body>, Error>
```

Sends a synchronous outbound HTTP request to a backend service and returns the response. Available when using `#[fastedge::http]`. For async outbound requests with `#[wstd::http_server]`, use `wstd::http::Client` instead.

**Supported methods:** `GET`, `POST`, `PUT`, `DELETE`, `HEAD`, `PATCH`, `OPTIONS`. Any other method returns `Err(Error::UnsupportedMethod)`.

**Errors:**

- `Error::UnsupportedMethod` — the request method is not in the supported set.
- `Error::BindgenHttpError` — the host runtime rejected or failed the request.
- `Error::InvalidBody` — the response body could not be decoded.

```rust,no_run
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Method, Request, Response, StatusCode};

#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    let upstream = Request::builder()
        .method(Method::GET)
        .uri("https://api.example.com/data")
        .header("accept", "application/json")
        .body(Body::empty())?;

    let upstream_resp = fastedge::send_request(upstream)?;

    Response::builder()
        .status(StatusCode::OK)
        .body(upstream_resp.into_body())
        .map_err(Into::into)
}
```

```rust,no_run
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Method, Request, Response, StatusCode};

#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    let payload = Body::from(r#"{"event":"click"}"#);
    let upstream = Request::builder()
        .method(Method::POST)
        .uri("https://ingest.example.com/events")
        .header("content-type", "application/json")
        .body(payload)?;

    let _resp = fastedge::send_request(upstream)?;

    Response::builder()
        .status(StatusCode::ACCEPTED)
        .body(Body::empty())
        .map_err(Into::into)
}
```

---

## Error Handling

### Error Enum

```rust,ignore
#[derive(thiserror::Error, Debug)]
pub enum Error {
    UnsupportedMethod(http::Method),
    BindgenHttpError(/* host HTTP error */),
    HttpError(http::Error),
    InvalidBody,
    InvalidStatusCode(u16),
}
```

| Variant                             | When it occurs                                                                                      |
| ----------------------------------- | --------------------------------------------------------------------------------------------------- |
| `UnsupportedMethod(http::Method)`   | `send_request` was called with a method other than GET, POST, PUT, DELETE, HEAD, PATCH, or OPTIONS  |
| `BindgenHttpError`                  | The host runtime returned an error during request execution                                         |
| `HttpError(http::Error)`            | An error occurred constructing or parsing an HTTP message                                           |
| `InvalidBody`                       | The request or response body could not be encoded or decoded                                        |
| `InvalidStatusCode(u16)`            | A status code outside the range 100–599 was encountered                                             |

`Error` implements `std::error::Error` and `std::fmt::Display`. It is compatible with `anyhow` and `?` propagation.

```rust
use fastedge::{Error, send_request};
use fastedge::body::Body;
use fastedge::http::{Method, Request};

fn fetch(uri: &str) -> Result<String, Error> {
    let req = Request::builder()
        .method(Method::GET)
        .uri(uri)
        .body(Body::empty())
        .map_err(Error::HttpError)?;

    let resp = send_request(req)?;
    Ok(format!("status: {}", resp.status()))
}
```

---

## Feature Flags

| Flag          | Default   | Effect                                                                              |
| ------------- | --------- | ----------------------------------------------------------------------------------- |
| `proxywasm`   | enabled   | Enables the `fastedge::proxywasm` module for ProxyWasm ABI compatibility            |
| `json`        | disabled  | Enables `Body::try_from(serde_json::Value)` and adds `serde_json` as a dependency  |

Enable non-default features in `Cargo.toml`:

```toml
[dependencies]
fastedge = { version = "0.3", features = ["json"] }
```

Disable the default `proxywasm` feature if you do not need it:

```toml
[dependencies]
fastedge = { version = "0.3", default-features = false }
```

---

## Re-exports

`fastedge` re-exports the [`http`](https://crates.io/crates/http) crate as `fastedge::http`. All standard HTTP types are available through this path without adding `http` as a direct dependency.

```rust,ignore
use fastedge::http::{Method, Request, Response, StatusCode, HeaderMap, Uri};
```

**Supported HTTP methods** (the complete set accepted by `send_request`):

| Constant            | Method    |
| ------------------- | --------- |
| `Method::GET`       | `GET`     |
| `Method::POST`      | `POST`    |
| `Method::PUT`       | `PUT`     |
| `Method::DELETE`    | `DELETE`  |
| `Method::HEAD`      | `HEAD`    |
| `Method::PATCH`     | `PATCH`   |
| `Method::OPTIONS`   | `OPTIONS` |

---

## Logging

The FastEdge platform captures **stdout only**. Output written to `stderr` is silently discarded and will not appear in the platform's log viewer. Use `print!` / `println!` for all diagnostic output. Do not use `eprint!` / `eprintln!` — those produce no visible output on the platform.

```rust,no_run
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    println!("Received request: {} {}", req.method(), req.uri());

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())
        .map_err(Into::into)
}
```

---

## See Also

- [HOST_SERVICES.md](HOST_SERVICES.md) — Key-value store, secrets, and dictionary APIs
- [quickstart.md](quickstart.md) — Getting started guide
- [INDEX.md](INDEX.md) — Documentation index
