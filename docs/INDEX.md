# FastEdge Rust SDK Documentation

Documentation for the `fastedge` crate (v0.3.5) — a Rust SDK for building edge computing applications that compile to WebAssembly and run on the FastEdge platform.

## Documents

| File                                 | Description                                                                                                                                                      |
| ------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [quickstart.md](quickstart.md)       | Getting started: project setup, writing a handler, building to WASM (`wasm32-wasip2` for async, `wasm32-wasip1` for basic/CDN)                                   |
| [SDK_API.md](SDK_API.md)             | Core API: handler macros (`#[wstd::http_server]`, `#[fastedge::http]`), Body type, outbound HTTP (`send_request`), errors, feature flags                         |
| [HOST_SERVICES.md](HOST_SERVICES.md) | Host services for HTTP apps: key-value store, secrets, dictionary, diagnostics                                                                                   |
| [CDN_APPS.md](CDN_APPS.md)           | CDN apps: proxy-wasm lifecycle, `fastedge::proxywasm::*` API surface, request/response manipulation                                                              |

## Suggested Reading Order

### HTTP Apps

1. **[quickstart.md](quickstart.md)** — project setup, first handler, build.
2. **[SDK_API.md](SDK_API.md)** — handler macros, Body type, outbound HTTP, errors.
3. **[HOST_SERVICES.md](HOST_SERVICES.md)** — key-value store, secrets, dictionary, diagnostics.

### CDN Apps

1. **[quickstart.md](quickstart.md)** — project setup, CDN section links to CDN_APPS.md.
2. **[CDN_APPS.md](CDN_APPS.md)** — proxy-wasm lifecycle, host services via `fastedge::proxywasm::*`, request/response manipulation.
