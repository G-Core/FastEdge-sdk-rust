# Runtime Architecture

## WIT World Definition

The SDK implements the `gcore:fastedge/reactor` world, defined in the `wit/` submodule (source: `G-Core/FastEdge-wit`).

```wit
world reactor {
    import http;           // HTTP types and utilities
    import http-client;    // Outbound HTTP requests
    import dictionary;     // Fast read-only config
    import secret;         // Encrypted secret access
    import key-value;      // Persistent storage
    import utils;          // Diagnostics and stats

    export http-handler;   // Main application entry point
}
```

---

## WIT Interfaces

### http-handler (exported by app)

```wit
interface http-handler {
    use http.{request, response};
    process: func(req: request) -> response;
}
```

This is what the `#[fastedge::http]` macro implements via the `Guest` trait.

### http-client (imported from platform)

Provides `send-request` for outbound HTTP calls. Wrapped by `src/http_client.rs`.

### key-value (imported from platform)

Resource-based API:
- `store::open(name)` — open a named store
- `store::get(key)` — retrieve value
- `store::scan(pattern)` — glob-pattern scan
- `store::zrange-by-score(key, min, max)` — sorted set range query
- `store::zscan(key, pattern, cursor, count)` — sorted set scan
- `store::bf-exists(key, item)` — bloom filter membership check

Errors: `no-such-store`, `access-denied`, `internal-error`

### secret (imported from platform)

- `get(key)` — retrieve current secret value
- `get-effective-at(key, timestamp)` — retrieve secret valid at a specific time

Errors: `access-denied`, `decrypt-error`, `other(string)`

### dictionary (imported from platform)

- `get(key)` — fast read-only configuration lookup

### utils (imported from platform)

- `set-user-diag(value)` — set diagnostic string for monitoring

---

## WIT Binding Generation

In `src/lib.rs`:

```rust
wit_bindgen::generate!({
    world: "reactor",
    path: "wit",
    pub_export_macro: true,
});
```

This generates Rust types and traits matching all WIT interfaces. The generated code lives in `target/` (not in source tree) under namespaces like `gcore::fastedge::*` and `exports::gcore::fastedge::*`.

---

## Submodules

| Submodule | Git Source | Purpose |
|-----------|-----------|---------|
| `wit/` | `G-Core/FastEdge-wit` | WIT interface definitions — the contract between SDK and platform |
| `wasi-nn/` | `WebAssembly/wasi-nn` | ML inference interface for neural network support |

Both are checked out via `.gitmodules`. The `wit/` submodule is required for building; `wasi-nn/` is optional.

---

## Host-SDK Contract

For the full ABI contract between the SDK and the host runtime — including all FFI functions, memory conventions, execution constraints, and WASM exports the host expects — see `architecture/HOST_SDK_CONTRACT.md`.

For how requests flow through the handler phases (including the HTTP callout pause/resume mechanism), see `architecture/REQUEST_LIFECYCLE.md`.

---

## ProxyWasm FFI Layer

When the `proxywasm` feature is enabled (default), `src/proxywasm/mod.rs` declares FFI bindings:

```rust
extern "C" {
    fn proxy_secret_get(...) -> u32;
    fn proxy_dictionary_get(...) -> u32;
    fn proxy_kv_store_open(...) -> u32;
    fn proxy_kv_store_get(...) -> u32;
    // ... etc
}
```

Each wrapper module (`secret.rs`, `key_value.rs`, `dictionary.rs`, `utils.rs`) wraps these unsafe FFI calls in safe Rust APIs with proper error handling, pointer management, and type conversion.

### FFI Wrapper Pattern

```rust
pub fn get(key: &str) -> Result<Option<Vec<u8>>, Error> {
    let mut return_data: *mut u8 = std::ptr::null_mut();
    let mut return_size: usize = 0;
    let status = unsafe { proxy_*_get(key.as_ptr(), key.len(), &mut return_data, &mut return_size) };
    // Convert status code → Result, copy data from host memory
}
```

---

## How WIT Changes Flow Through the Codebase

1. Update `.wit` files in `wit/` submodule
2. Run `cargo build` — `wit-bindgen` regenerates Rust types
3. Update Rust wrapper module in `src/` to expose the new API
4. Add ProxyWasm FFI equivalent in `src/proxywasm/` if applicable
5. Create example demonstrating the new capability

---

**Last Updated**: March 2026
