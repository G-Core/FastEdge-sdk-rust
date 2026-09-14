# Agent Decision Log

This file tracks agent decisions, architectural changes, and context for future agents working in this repository. It is **not** related to the root `CHANGELOG.md` which tracks release versions.

---

## [2026-09-14] — ProxyWasm cache module

### Overview
Added `fastedge::proxywasm::cache` — the ProxyWasm/CDN counterpart of the `cache-sync` WIT interface already exposed to HTTP apps as `fastedge::cache`.

### Decisions
- Free functions rather than a handle-based `Store` (as in `key_value`): the `cache-sync` WIT interface has no named stores or handles, every operation is scoped to the calling app and addressed by key alone.
- `Error` mirrors the WIT `cache-types.error` variant (`AccessDenied`, `InternalError`, `Other(String)`) instead of reusing `key_value::Error` (which carries `NoSuchStore`).
- `option<u64>` TTL is carried over FFI as a plain `u64` with `0` meaning "no expiry", avoiding an extra flag/pointer parameter for a value that has no meaningful zero case.
- Host status codes follow the real ProxyWasm status enum (`0` ok, `1` not found, `2` bad argument — which also carries access denial, as in `key_value` — `10` internal failure); "not found" is folded into `Ok(None)` / `Ok(false)` / no-op per the WIT contract. Note `reference/ERROR_CODES.md` previously listed `3`/`6` for these, which does not match the host enum; corrected in the same change.

### Changes
- `src/proxywasm/cache.rs` — new module: `get`, `set`, `delete`, `exists`, `incr`, `expire`, `purge`, `purge_prefix`
- `src/proxywasm/mod.rs` — `pub mod cache`, eight `proxy_cache_*` FFI declarations, module docs
- `context/architecture/HOST_SDK_CONTRACT.md` — documented the `proxy_cache_*` FFI functions
- `examples/cdn/cache/` — new CDN example exercising all eight operations via query parameters, modelled on `examples/cdn/key_value/` (no `store` param — the cache has no named stores)
- `examples/README.md` — listed the new CDN example
- `fastedge-plugin-source/manifest.json` — `cdn-cache-blueprint` / `cdn-cache-pattern` source + target entries

### Follow-up
- The `proxy_cache_*` symbol names and signatures were derived from the existing `proxy_kv_store_*` convention — they must be confirmed against the host implementation before release.
- `examples/cdn/cache/Cargo.toml` uses a path dependency (`fastedge = { path = "../../.." }`) because `proxywasm::cache` is not in the published 0.4.2 crate. Every other example uses the registry dep — switch this one to `{ version = "0.4", features = ["proxywasm"] }` once the module is published.
- Per `context/PLUGIN_CONTRACT.md` steps 3-4, the `fastedge-plugin` repo still needs intent files (`cdn/cache-rust.md`, `cdn/examples-cache-rust.md`) and placeholder reference files at the mapped target paths, or the next sync will fail for the two new manifest entries.

---

## [2026-04-07] — Migrated Rust examples from FastEdge-examples

### Overview
Migrated all Rust examples from the FastEdge-examples repository into this SDK repo, completing the pattern already established for JavaScript (FastEdge-sdk-js) and AssemblyScript (proxy-wasm-sdk-as).

### Examples Added

**CDN (proxy-wasm):**
- `jwt` — JWT token validation (signature + expiration)
- `md2html` — Markdown-to-HTML conversion in response body
- `convert_image` — Image-to-AVIF conversion with configurable quality/speed
- `custom_error_pages` — Branded error pages using Handlebars templates (includes build.rs, images, message templates)
- `geoblock` — Country-based request blocking with optional time windows

**HTTP basic (sync):**
- `s3upload` — S3 file upload via signed URLs
- `smart_switch` — SmartThings API device toggling

### Decisions
- Each example follows the existing standalone pattern: empty `[workspace]`, crates.io dependencies (not path deps), `src/lib.rs`, own README
- `markdown` and `kv-store` from FastEdge-examples were **not** migrated — they duplicate `markdown_render` and `key_value` already in this repo
- `custom_error_pages` had its Tailwind CSS dependency removed — replaced with hand-written CSS (~60 lines) covering the exact utility classes used. Removed `package.json`, `tailwind.css`, `.prettierrc`
- `custom_error_pages` had a bug fix: `on_http_response_body` previously defaulted `status_code` to 200 when `response.status` was missing/invalid, which could replace successful responses with the error template. Now returns `Action::Continue` early if the property is missing, not 2 bytes, or outside 4xx/5xx
- `custom_error_pages` includes 6 fastedge-test fixtures for testing different error states via the built-in responder's `x-debugger-status` header
- Renamed `tailwind_styles` to `styles` in both Rust source and Handlebars template

### Files Updated
- `examples/README.md` — Added all 7 new examples to the tables
- `examples/cdn/` — 5 new example directories
- `examples/http/basic/` — 2 new example directories

---

## [2026-03-31] — Added Host-Side Context (Contract, Lifecycle, Properties, Errors)

### Overview
Added 4 new context files documenting the host-SDK relationship from the SDK developer's perspective. Based on research of the host runtime source code (rust_host/proxywasm), distilled to high-level concepts without exposing proprietary implementation details.

### Decisions
- Host internals are proprietary — context docs describe the **contract** (what the SDK developer needs to know), not the implementation
- Properties, error codes, and lifecycle phases documented as reference material for debugging and development
- HTTP callout pause/resume mechanism documented as a conceptual flow — this was a gap that made CDN-mode development harder to understand

### Files Created
- `context/architecture/HOST_SDK_CONTRACT.md` — ABI contract, FFI functions, memory conventions, execution constraints
- `context/architecture/REQUEST_LIFECYCLE.md` — Handler phases, HTTP callout pause/resume, local response short-circuit
- `context/reference/PROPERTIES_REFERENCE.md` — Available request properties (geo, IP, host, tracing)
- `context/reference/ERROR_CODES.md` — Host status codes (3100-3120), SDK errors, module errors

### Files Updated
- `context/CONTEXT_INDEX.md` — Added entries for all 4 new files + 5 new decision tree scenarios
- `context/architecture/RUNTIME_ARCHITECTURE.md` — Cross-references to HOST_SDK_CONTRACT and REQUEST_LIFECYCLE
- `context/architecture/SDK_ARCHITECTURE.md` — Added HTTP callout pause/resume mention + host constraint note
- `CLAUDE.md` — Updated context organization tree + decision tree table

---

## [2026-03-31] — Documented Two Handler Patterns

### Overview
Added documentation for `#[wstd::http_server]` (async, WASI-HTTP) alongside `#[fastedge::http]` (sync, original).

### Decisions
- `#[wstd::http_server]` is the recommended approach for new apps — documented as the forward path
- `#[fastedge::http]` is the original basic pattern — `#[wstd::http_server]` is preferred for new apps
- New examples should use the wstd async pattern, not fastedge sync
- Updated all context docs (SDK_ARCHITECTURE, PROJECT_OVERVIEW, CONTEXT_INDEX, CLAUDE.md) to reflect both patterns

---

## [2026-03-31] — Context System Created

### Overview

Established discovery-based context system (CLAUDE.md + context/) for AI agent discoverability, following the pattern used by FastEdge-sdk-js and fastedge-test.

### Decisions

- Followed the **FastEdge-sdk-js pattern** (lean SDK-style) rather than fastedge-test (full-stack app with more surface area)
- Content distilled from existing `AGENTS.md` (827 lines) and `DOCUMENTATION.md` (844 lines) into structured context/ files
- `AGENTS.md` converted to a pointer file directing agents to `CLAUDE.md`
- `DOCUMENTATION.md` removed — all content absorbed into context/
- Context docs kept under 170 lines each for single-sitting reads

### Files Created

- `CLAUDE.md` — entry point for AI agents
- `context/CONTEXT_INDEX.md` — discovery hub
- `context/PROJECT_OVERVIEW.md` — lightweight overview
- `context/architecture/SDK_ARCHITECTURE.md` — core architecture
- `context/architecture/RUNTIME_ARCHITECTURE.md` — WIT + runtime
- `context/development/BUILD_AND_CI.md` — build system + CI
- `context/CHANGELOG.md` — this file
