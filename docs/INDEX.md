# FastEdge Rust SDK Documentation

Documentation for the `fastedge` crate (v0.3.5) — a Rust SDK for building edge computing applications that compile to WebAssembly and run on the FastEdge platform.

## Documents

| File                                   | Description                                                                  |
| -------------------------------------- | ---------------------------------------------------------------------------- |
| [quickstart.md](quickstart.md)         | Getting started: adding the dependency, writing a handler, building to WASM  |
| [SDK_API.md](SDK_API.md)               | Public API reference: types, traits, macros, error variants, feature flags   |
| [HOST_SERVICES.md](HOST_SERVICES.md)   | Host-provided services: KV store, secrets, outbound HTTP, request properties |

## Suggested Reading Order

1. **[quickstart.md](quickstart.md)** — Start here. Covers dependency setup, a minimal handler, and building to `wasm32-wasip1`.
2. **[SDK_API.md](SDK_API.md)** — Reference for all public types, the `#[fastedge::http]` and `#[wstd::http_server]` handler macros, the `Body` type, HTTP request/response types, and error handling.
3. **[HOST_SERVICES.md](HOST_SERVICES.md)** — Reference for runtime services your handler can call: outbound HTTP, key-value store, secrets, and request properties.
