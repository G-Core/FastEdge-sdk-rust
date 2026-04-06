# SDK Architecture

## Dual API Approach

The SDK supports two runtime models:

### Component Model (Primary)

- Uses WIT (WebAssembly Interface Types) via `wit-bindgen` 0.46
- Type-safe generated bindings from `wit/` definitions
- Generated in `src/lib.rs` via `wit_bindgen::generate!` macro
- Modern WebAssembly Component Model standard

### ProxyWasm (Secondary, feature-gated)

- Enabled by default via `features = ["proxywasm"]`
- FFI-based using `extern "C"` declarations in `src/proxywasm/mod.rs`
- Compatible with Envoy and other proxy-wasm hosts
- Wraps unsafe FFI in safe Rust APIs

---

## Two Handler Patterns

### `#[wstd::http_server]` — Async / WASI-HTTP (Future)

The forward path. Uses the `wstd` crate (v0.6) for async WASI-HTTP handlers. **This is the recommended approach for new apps.**

```rust
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    Ok(Response::builder()
        .status(200)
        .body(Body::from("Hello"))?)
}
```

- Uses standard WASI-HTTP interfaces (not FastEdge-specific)
- Async handler with proper HTTP client (`wstd::http::Client`)
- Examples: `examples/http/wasi/` (hello_world, headers, key_value, outbound_fetch, etc.)
- Dependency: `wstd = "0.6"` (external crate, not part of this SDK)

### `#[fastedge::http]` — Sync / Original

The original pattern provided by this SDK's derive macro (`derive/src/lib.rs`). Transforms sync functions into Component Model exports.

```rust
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    Ok(Response::builder().status(StatusCode::OK).body(Body::empty())?)
}
```

- Sync handler — no async support
- Macro generates Guest trait implementation with type conversions and error-to-500 handling
- Examples: `examples/http/basic/` (hello_world, headers, key_value, secret, backend, etc.)
- Dependency: `fastedge` crate (this SDK)

**Note:** `#[wstd::http_server]` is the preferred choice for new apps, but `#[fastedge::http]` remains fully supported.

---

## Type Conversion Pattern

Three type systems are bridged:

1. **Standard `http` crate types** — user-facing API (`http::Request`, `http::Response`)
2. **WIT-generated bindgen types** — runtime interface (`http_handler::Request`, `http_handler::Response`)
3. **Internal `Body` type** — wraps `bytes::Bytes` with content-type awareness

Key conversions in `src/lib.rs`:

| Conversion | Direction |
|------------|-----------|
| `From<Method> for http::Method` | bindgen → http crate |
| `TryFrom<Request> for http::Request<Body>` | bindgen → http crate |
| `From<http::Response<Body>> for Response` | http crate → bindgen |
| `TryFrom<Response> for http::Response<Body>` | bindgen → http crate |

---

## Body Type

`body::Body` wraps `bytes::Bytes` with content-type metadata:

```rust
pub struct Body {
    pub(crate) content_type: String,
    pub(crate) inner: Bytes,
}
```

- Implements `Deref` to `Bytes` for transparent access
- Auto-assigns content-type: `text/plain` for strings, `application/octet-stream` for bytes
- `Body::empty()` factory for empty responses
- Optional JSON support (`json` feature): `TryFrom<serde_json::Value>` → `application/json`

---

## Error Handling

### SDK Error Type (`src/lib.rs`)

```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    UnsupportedMethod(http::Method),    // Unknown HTTP method
    BindgenHttpError(HttpError),        // WIT-generated error
    HttpError(http::Error),             // http crate error
    InvalidBody,                        // Body conversion failure
    InvalidStatusCode(u16),             // Bad status code
}
```

### Module-Specific Errors

Each ProxyWasm module defines its own error enum:
- **key_value**: `NoSuchStore`, `AccessDenied`, `InternalError`
- **secret**: `AccessDenied`, `DecryptError`, `Other(String)`

### Handler Error Flow

When a handler returns `Err(...)`, the `#[fastedge::http]` macro catches it and returns an HTTP 500 response with the error message as body.

---

## Module Structure

### Public API Surface

```rust
// Re-exported from lib.rs
pub mod body;           // Body type
pub mod dictionary;     // dictionary::get()
pub mod secret;         // secret::get(), secret::get_effective_at()
pub mod key_value;      // Store resource
pub mod utils;          // set_user_diag()
pub use http;           // Re-export http crate
pub use send_request;   // Outbound HTTP function
```

### Generated Namespaces (from WIT)

```
gcore::fastedge::http           // HTTP types
gcore::fastedge::http_client    // Outbound requests
gcore::fastedge::key_value      // KV store
gcore::fastedge::secret         // Secret access
gcore::fastedge::dictionary     // Config lookups
gcore::fastedge::utils          // Diagnostics
exports::gcore::fastedge::http_handler  // Handler export
```

---

## HTTP Client (`src/http_client.rs`)

`send_request(req: http::Request<Body>) -> Result<Response<Body>, Error>`

1. Converts `http::Request<Body>` → bindgen request types
2. Calls `http_client::send_request` (WIT import)
3. Converts bindgen response → `http::Response<Body>`
4. Supports all methods: GET, POST, PUT, DELETE, HEAD, PATCH, OPTIONS

**Note:** Outbound HTTP calls to non-public (internal/private) IP addresses are blocked by the host. See `architecture/HOST_SDK_CONTRACT.md` for execution constraints.

### HTTP Callouts in ProxyWasm (CDN Mode)

CDN-mode apps use a different mechanism: `proxy_http_call()` which is asynchronous with a pause/resume pattern. The app pauses request processing, the host makes the outbound call, then delivers the response via `proxy_on_http_call_response()`. See `architecture/REQUEST_LIFECYCLE.md` for the full flow.

---

## Import Patterns

```rust
// Standard handler
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

// HTTP client
use fastedge::send_request;

// Key-value store
use fastedge::key_value::Store;

// Secrets & dictionary
use fastedge::secret;
use fastedge::dictionary;

// Utilities
use fastedge::utils::set_user_diag;
```

---

**Last Updated**: March 2026
