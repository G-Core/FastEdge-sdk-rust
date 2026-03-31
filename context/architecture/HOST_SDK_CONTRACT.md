# Host-SDK Contract

This document describes the interface contract between the FastEdge host runtime and WASM modules built with the SDK. It covers what the SDK developer needs to know — not how the host implements it internally.

---

## Overview

SDK apps are WASM modules that run inside a host runtime. Communication happens through two mechanisms:

1. **Component Model (WIT)** — type-safe imports/exports defined in `wit/` (see RUNTIME_ARCHITECTURE.md)
2. **ProxyWasm FFI** — `extern "C"` function calls for CDN/proxy-wasm environments

Both expose the same capabilities (KV, secrets, dictionary, HTTP, diagnostics) through different calling conventions.

---

## Memory Convention

All FFI-based host functions pass data via pointer/size pairs in WASM linear memory:

**Sending data to host:**
```
(key_ptr: *const u8, key_size: usize)  // SDK writes, host reads
```

**Receiving data from host:**
```
(return_data: *mut *mut u8, return_size: *mut usize)  // Host allocates via WASM export, SDK reads
```

The host calls the WASM export `proxy_on_memory_allocate(size) -> ptr` (or `malloc`) to allocate memory in the guest for return values. The SDK's FFI wrappers in `src/proxywasm/` handle all pointer management — application code never deals with raw pointers.

**Return values:** All `proxy_*` FFI functions return `u32` status codes:
- `0` — success
- Non-zero — error (mapped to module-specific error enums by the SDK wrappers)

---

## Host-Provided Functions (ProxyWasm FFI)

These are the `extern "C"` functions the host makes available to WASM modules. The SDK wraps them in safe Rust APIs.

### Key-Value Store

| FFI Function | SDK Wrapper | Purpose |
|-------------|-------------|---------|
| `proxy_kv_store_open(name, len, handle)` | `Store::open(name)` | Open a named KV store, returns handle |
| `proxy_kv_store_get(handle, key, len, ret, ret_len)` | `Store::get(key)` | Retrieve value by key |
| `proxy_kv_store_scan(handle, pattern, len, ret, ret_len)` | `Store::scan(pattern)` | Glob-pattern key scan |
| `proxy_kv_store_zrange_by_score(handle, key, len, min, max, ret, ret_len)` | `Store::zrange_by_score(key, min, max)` | Sorted set range query |
| `proxy_kv_store_zscan(handle, key, len, pattern, plen, ret, ret_len)` | `Store::zscan(key, pattern)` | Sorted set pattern scan |
| `proxy_kv_store_bf_exists(handle, key, len, item, ilen, ret)` | `Store::bf_exists(key, item)` | Bloom filter membership check |

### Secrets

| FFI Function | SDK Wrapper | Purpose |
|-------------|-------------|---------|
| `proxy_secret_get(key, len, ret, ret_len)` | `secret::get(key)` | Retrieve current secret value |
| `proxy_secret_get_effective_at(key, len, ts, ret, ret_len)` | `secret::get_effective_at(key, ts)` | Retrieve secret valid at timestamp |

Timestamp-based retrieval supports secret rotation — fetch the secret that was active at a specific point in time.

### Dictionary

| FFI Function | SDK Wrapper | Purpose |
|-------------|-------------|---------|
| `proxy_dictionary_get(key, len, ret, ret_len)` | `dictionary::get(key)` | Read-only configuration lookup |

Dictionary values are set at deployment time and are immutable during request handling. Fast path for configuration data.

### Diagnostics

| FFI Function | SDK Wrapper | Purpose |
|-------------|-------------|---------|
| `stats_set_user_diag(value, len)` | `utils::set_user_diag(value)` | Set diagnostic string for monitoring |

The diagnostic string is attached to the request and visible in platform monitoring/logs.

---

## Component Model Interface

For the WIT-based Component Model path, the same capabilities are exposed as typed interfaces rather than raw FFI. The `wit_bindgen::generate!` macro produces Rust bindings automatically.

The WIT world (`gcore:fastedge/reactor`) imports:
- `http` + `http-client` — request/response types and outbound HTTP
- `key-value` — persistent storage (same operations as FFI above)
- `secret` — encrypted secrets (same operations as FFI above)
- `dictionary` — read-only config (same as FFI above)
- `utils` — diagnostics (same as FFI above)

And exports:
- `http-handler` — the app's request processing function

See `architecture/RUNTIME_ARCHITECTURE.md` for full WIT definitions.

---

## WASM Module Exports (called by host)

The host expects these exports from the WASM module:

### Component Model Apps (`#[fastedge::http]` / `#[wstd::http_server]`)

| Export | Purpose |
|--------|---------|
| `process(request) -> response` | Main handler — receives HTTP request, returns response |

The `#[fastedge::http]` macro generates the `Guest` trait implementation that bridges to this export.

### ProxyWasm Apps (CDN)

| Export | Purpose |
|--------|---------|
| `proxy_on_memory_allocate(size) -> ptr` | Memory allocation for host-to-guest data transfer |
| `_initialize()` or `_start()` | Module initialization |
| `proxy_on_context_create(context_id, parent_id)` | Create root (parent=0) or request context |
| `proxy_on_request_headers(context_id, num_headers)` | Handle request headers phase |
| `proxy_on_request_body(context_id, body_size, end_of_stream)` | Handle request body phase |
| `proxy_on_response_headers(context_id, num_headers)` | Handle response headers phase |
| `proxy_on_response_body(context_id, body_size, end_of_stream)` | Handle response body phase |
| `proxy_on_log(context_id)` | Final logging callback |
| `proxy_on_http_call_response(ctx, call_id, h_size, b_size, t_size)` | HTTP callout response delivered |

See `architecture/REQUEST_LIFECYCLE.md` for the order these are called.

---

## Execution Constraints

The host enforces limits on WASM execution:

| Constraint | Behavior |
|-----------|----------|
| **Execution timeout** | Host interrupts WASM after a configured duration |
| **Memory limit** | WASM linear memory is capped; exceeding it triggers a trap |
| **Non-public hosts blocked** | Outbound HTTP calls to internal/private IPs are rejected |
| **Single-threaded** | Each request runs in its own WASM instance, no shared state between requests |

---

**Last Updated**: March 2026
