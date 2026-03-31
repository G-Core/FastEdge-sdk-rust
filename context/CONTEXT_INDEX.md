# Context Discovery Index

## Quick Start

- **Project:** FastEdge Rust SDK
- **Crate:** `fastedge` v0.3.5 on crates.io
- **Purpose:** Build edge computing apps in Rust that compile to WASM for Gcore's FastEdge platform
- **Workspace:** 2 crates — `fastedge` (core SDK) + `fastedge-derive` (proc macro)
- **Target:** `wasm32-wasip1`
- **APIs:** HTTP handler, outbound HTTP, key-value store, secrets, dictionary, utils, WASI-NN
- **Dual Runtime:** Component Model (WIT/wit-bindgen) + ProxyWasm (FFI)
- **Build:** `cargo build --release` (default target is WASM via `.cargo/config.toml`)
- **Test:** `cargo test` (Rust-native tests only)

---

## Documentation Map

### Architecture (read when modifying internal structure)

| Document | Lines | Purpose |
|----------|-------|---------|
| `architecture/SDK_ARCHITECTURE.md` | ~197 | Dual API approach, type conversion pattern, Body type, error handling, module structure, import patterns. Read when working on `src/`. |
| `architecture/RUNTIME_ARCHITECTURE.md` | ~140 | WIT world definition, interface contracts, submodules, ProxyWasm FFI layer, WIT change workflow. Read when working on `wit/` or `src/proxywasm/`. |
| `architecture/HOST_SDK_CONTRACT.md` | ~130 | ABI contract between SDK and host: FFI functions, memory conventions, execution constraints. Read when adding host function wrappers or debugging host interaction. |
| `architecture/REQUEST_LIFECYCLE.md` | ~130 | Request phases (Component Model + ProxyWasm), HTTP callout pause/resume, local response short-circuit. Read when debugging handler behavior or building CDN apps. |

### Development (read when building or deploying)

| Document | Lines | Purpose |
|----------|-------|---------|
| `development/BUILD_AND_CI.md` | ~142 | Workspace config, build commands, CI pipeline, release pipeline, FOSSA, example build pattern, size optimization. Read when changing build or CI. |

### Reference (search on-demand)

| Document | Lines | Purpose |
|----------|-------|---------|
| `PROJECT_OVERVIEW.md` | ~149 | Lightweight overview — crate structure, modules, features, examples, deps, setup. Read when new to the codebase. |
| `reference/PROPERTIES_REFERENCE.md` | ~80 | Available request properties (geo, IP, host, URI, tracing) for ProxyWasm apps. Read when working with `proxy_get_property()`. |
| `reference/ERROR_CODES.md` | ~120 | Host status codes (3100-3120), SDK error enum, module errors, FFI status codes. Read when debugging failures. |
| `CHANGELOG.md` | ~29+ | Agent decision log. Use grep as this file grows. |

### External (not in context/)

| Resource | Location | Purpose |
|----------|----------|---------|
| API docs | https://docs.rs/fastedge | Generated Rust documentation |
| WIT definitions | `wit/` (submodule → G-Core/FastEdge-wit) | Interface contracts |
| Examples | `examples/` (30+ apps) | Real-world usage patterns |
| Release changelog | Root `CHANGELOG.md` | Version history (auto-generated) |

---

## Decision Tree: What Should I Read?

### Adding a New WIT Interface
1. Read `architecture/RUNTIME_ARCHITECTURE.md` (WIT section + change workflow)
2. Read existing `.wit` files in `wit/`
3. Add Rust wrapper in `src/`, following existing module patterns

### Fixing the Proc Macro
1. Read `architecture/SDK_ARCHITECTURE.md` (attribute macro pattern section)
2. Read `derive/src/lib.rs` directly

### Adding a ProxyWasm Feature
1. Read `architecture/RUNTIME_ARCHITECTURE.md` (ProxyWasm FFI section)
2. Read `architecture/SDK_ARCHITECTURE.md` (module structure)
3. Read existing wrapper as template: `src/proxywasm/key_value.rs`

### Modifying HTTP Client
1. Read `architecture/SDK_ARCHITECTURE.md` (HTTP client + type conversion sections)
2. Read `src/http_client.rs` directly

### Working with KV / Secrets / Dictionary
1. Read `architecture/SDK_ARCHITECTURE.md` (module structure section)
2. Read the specific module in `src/proxywasm/`

### Writing a New WASI-HTTP App (async)
1. Read `architecture/SDK_ARCHITECTURE.md` (two handler patterns — `#[wstd::http_server]` section)
2. Browse `examples/http/wasi/hello_world/` as the simplest template
3. Use `wstd = "0.6"` — this is the **recommended** approach for new apps

### Working with Basic Sync Handler
1. Read `architecture/SDK_ARCHITECTURE.md` (two handler patterns — `#[fastedge::http]` section)
2. Browse `examples/http/basic/` for sync examples
3. Note: `#[wstd::http_server]` is preferred for new apps, but `#[fastedge::http]` is fully supported

### Adding an Example
1. Browse `examples/` for a similar existing example
2. **Prefer `#[wstd::http_server]` (async) for new examples** over `#[fastedge::http]` (basic sync)
3. Read `PROJECT_OVERVIEW.md` (examples section)
4. Read `development/BUILD_AND_CI.md` (example build pattern)

### Understanding the System (New to Codebase)
1. Read `PROJECT_OVERVIEW.md` (~149 lines)
2. Skim `architecture/SDK_ARCHITECTURE.md` (two handler patterns + module structure)
3. Browse `examples/http/wasi/hello_world/` for the recommended pattern

### Changing Build or CI
1. Read `development/BUILD_AND_CI.md`
2. Check `.github/workflows/` for specific pipeline

### Modifying Type Conversions
1. Read `architecture/SDK_ARCHITECTURE.md` (type conversion + body type sections)
2. Read `src/lib.rs` (conversion implementations)

### Adding Error Handling
1. Read `reference/ERROR_CODES.md` (full error catalog)
2. Read `architecture/SDK_ARCHITECTURE.md` (error handling section)
3. Check existing module error types in `src/proxywasm/`

### Debugging Host Interaction / Status Codes
1. Read `reference/ERROR_CODES.md` (host codes 3100-3120)
2. Read `architecture/HOST_SDK_CONTRACT.md` (execution constraints)

### Working with Request Properties (ProxyWasm)
1. Read `reference/PROPERTIES_REFERENCE.md` (available properties)
2. Browse `examples/cdn/properties/` for usage example

### Understanding HTTP Callout Mechanism
1. Read `architecture/REQUEST_LIFECYCLE.md` (pause/resume section)
2. Read `architecture/HOST_SDK_CONTRACT.md` (host-provided functions)
3. Browse `examples/cdn/http_call/` for working example

### Adding a Host Function Wrapper
1. Read `architecture/HOST_SDK_CONTRACT.md` (FFI functions, memory convention)
2. Read `architecture/RUNTIME_ARCHITECTURE.md` (ProxyWasm FFI + WIT change workflow)
3. Use `src/proxywasm/key_value.rs` as template

### Working with WASI-NN / ML
1. Read `architecture/RUNTIME_ARCHITECTURE.md` (submodules section)
2. Check `wasi-nn/` submodule for interface definitions

### Updating Dependencies
1. Read `PROJECT_OVERVIEW.md` (key dependencies table)
2. Read `development/BUILD_AND_CI.md` (workspace config)
3. Note: `wit-bindgen` version must match Wasmtime runtime version

---

## Search Tips

- **Don't** read `CHANGELOG.md` linearly — grep for keywords as it grows
- **Grep patterns:**
  - `grep -r "wit_bindgen" src/` — find WIT binding usage
  - `grep -r "extern \"C\"" src/` — find FFI declarations
  - `grep -r "fastedge::http" examples/` — find handler examples
  - `grep -r "#\[cfg(feature" src/` — find feature-gated code
  - `grep -r "pub fn\|pub struct\|pub enum" src/` — find public API surface

---

## Documentation Size Reference

| Category | Documents | Total Lines |
|----------|-----------|-------------|
| Architecture | 4 docs | ~597 |
| Development | 1 doc | ~142 |
| Reference | 4 docs | ~378 |
| **Total** | **9 docs** | **~1,117** |

All documents are designed for single-sitting reads. No doc exceeds ~200 lines.

---

**Last Updated**: March 2026
