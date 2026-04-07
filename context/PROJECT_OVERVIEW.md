# Project Overview

## What Is This?

**FastEdge Rust SDK** is a library for building edge computing applications that compile to WebAssembly and run on Gcore's FastEdge platform. It provides HTTP request handling, backend communication, key-value storage, secret management, and ML inference capabilities.

- **Crate**: `fastedge` on crates.io
- **Docs**: https://docs.rs/fastedge
- **Repository**: https://github.com/G-Core/FastEdge-sdk-rust
- **License**: Apache-2.0

---

## Crate Structure (Workspace)

This is a Rust workspace with 2 member crates:

| Crate | Type | Purpose |
|-------|------|---------|
| `fastedge` | Library (`cdylib`) | Core SDK — HTTP handler, client, KV, secrets, dictionary, utils |
| `fastedge-derive` | Proc-macro | `#[fastedge::http]` attribute macro for marking HTTP handler functions |

Both share version `0.3.5`, edition 2021.

---

## Key Modules

### Core SDK (`src/`)

| File | Lines | Purpose |
|------|-------|---------|
| `lib.rs` | ~667 | Entry point, type conversions, module re-exports, `wit_bindgen::generate!` |
| `http_client.rs` | ~141 | `send_request()` — outbound HTTP to backend services |
| `helper.rs` | ~75 | Internal binary serialization/deserialization utilities |

### ProxyWasm Layer (`src/proxywasm/`)

Used by **CDN apps** — proxy-wasm filters running inside Gcore's CDN proxy layer. CDN apps are a separate application type from HTTP apps, not a feature-flag variant. The `proxywasm` Cargo feature enables this module.

| File | Lines | Purpose |
|------|-------|---------|
| `mod.rs` | ~109 | `extern "C"` FFI declarations for all proxy_* functions |
| `key_value.rs` | ~292 | `Store` resource — open, get, scan, zrange, bloom filter |
| `secret.rs` | ~142 | `secret::get()`, `secret::get_effective_at()` |
| `dictionary.rs` | ~64 | `dictionary::get()` — read-only config lookups |
| `utils.rs` | ~50 | `set_user_diag()` — diagnostic reporting |

### Derive Macro (`derive/src/lib.rs`)

~186 lines. Transforms `#[fastedge::http] fn main(req) -> Result<Response>` into a full Component Model export with Guest trait implementation and error-to-500 conversion.

---

## Feature Flags

| Feature | Default | Purpose |
|---------|---------|---------|
| `proxywasm` | Yes | ProxyWasm compatibility layer (FFI bindings) |
| `json` | No | JSON body support via `serde_json` |

---

## WIT World

The SDK implements the `gcore:fastedge/reactor` world:

**Imports** (platform provides): `http`, `http-client`, `dictionary`, `secret`, `key-value`, `utils`
**Exports** (app implements): `http-handler`

WIT definitions live in the `wit/` submodule (points to `G-Core/FastEdge-wit`).

---

## Examples

30+ examples organized in three categories:

| Category | Path | Handler | Description |
|----------|------|---------|-------------|
| HTTP WASI | `examples/http/wasi/` | `#[wstd::http_server]` (async, **recommended**) | hello_world, headers, key_value, outbound_fetch, secret_rollover, geo_redirect, variables_and_secrets, simple_fetch |
| HTTP Basic | `examples/http/basic/` | `#[fastedge::http]` (sync, **original**) | hello_world, headers, key_value, secret, backend, geo_redirect, api_wrapper, etc. |
| CDN | `examples/cdn/` | ProxyWasm | headers, body, http_call, key_value, geo_redirect, properties, variables_and_secrets, etc. |

- `#[wstd::http_server]` is the forward path — new apps should use this pattern
- `#[fastedge::http]` is the original basic pattern and remains fully supported
- Each example has its own `Cargo.toml`, `src/lib.rs`, and `README.md`

---

## Submodules

| Submodule | Source | Purpose |
|-----------|--------|---------|
| `wit/` | `G-Core/FastEdge-wit` | WIT interface definitions for the reactor world |
| `wasi-nn/` | `WebAssembly/wasi-nn` | ML inference interface definitions |

---

## Development Setup

```bash
# Prerequisites
rustup target add wasm32-wasip1

# Build
cargo build --target wasm32-wasip1 --release

# Check (no build artifacts)
cargo check --target wasm32-wasip1

# Lint
cargo clippy --target wasm32-wasip1 --all-targets --all-features

# Format
cargo fmt

# Test (Rust-native tests only, no WASM)
cargo test

# Build a specific example
cargo build --target wasm32-wasip1 --release --package hello-world
```

Default build target is `wasm32-wasip1` (set in `.cargo/config.toml`).

**Build targets by app type:**

| App Type | Target | Handler |
|----------|--------|---------|
| HTTP (async, recommended) | `wasm32-wasip2` | `#[wstd::http_server]` |
| HTTP (basic, legacy) | `wasm32-wasip1` | `#[fastedge::http]` |
| CDN (proxy-wasm) | `wasm32-wasip1` | `proxy_wasm::main!` + traits |

---

## Platform Constraints

- **Logging:** Only stdout is captured. `eprint!` / `eprintln!` output is silently lost and will NOT appear in the platform's log viewer. Use `print!` / `println!` or `log::info!` (with proxy-wasm log level configured for CDN apps).
- **Dictionary:** `fastedge::dictionary::get()` reads platform-managed environment configuration (set at deployment time, same management as secrets but not encrypted).

---

## Testing

- Inline unit tests in modules (e.g., `helper.rs` has serialize/deserialize round-trip tests)
- No dedicated test directory — examples serve as integration-level validation
- CI runs: `cargo-audit`, `cargo build --release --all-features`, `cargo doc`, `cargo clippy`
- All warnings are errors in CI (`RUSTFLAGS="-Dwarnings"`)

---

## Key Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| `http` | 1.3 | Standard HTTP types (user-facing API) |
| `bytes` | 1.10 | Zero-copy byte buffers |
| `wit-bindgen` | 0.46 | WIT code generation (must match Wasmtime version) |
| `thiserror` | 2.0 | Error derive macros |
| `mime` | 0.3 | MIME type handling |
| `serde_json` | 1.0 | JSON support (optional, `json` feature) |

---

## Consumer Documentation (`docs/`)

Generated from source code by `fastedge-plugin-source/generate-docs.sh`. These docs are consumed by the fastedge-plugin sync pipeline and serve as the source of truth for SDK reference material in the Claude plugin and (future) MCP server.

| File | Covers |
|------|--------|
| `SDK_API.md` | Handler macros, Body type, send_request, errors, feature flags |
| `HOST_SERVICES.md` | HTTP app host services (Component Model): key_value, secret, dictionary, utils |
| `CDN_APPS.md` | CDN app guide: proxy-wasm lifecycle, `fastedge::proxywasm::*` API, examples |
| `quickstart.md` | Getting started for both HTTP and CDN apps |
| `INDEX.md` | Doc index with reading order by app type |

**Do not hand-edit** — update `.generation-config.md` and regenerate.

---

**Last Updated**: April 2026
