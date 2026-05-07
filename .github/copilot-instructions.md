# Copilot PR Review Instructions — FastEdge-sdk-rust

## Constitution

This repository is `fastedge` (crate) — the Rust SDK for Gcore FastEdge. It provides the `#[fastedge::http]` and `#[wstd::http_server]` handler macros, type conversions, an outbound HTTP client, and ProxyWasm FFI wrappers for CDN apps.

### Principles (enforce during review)

1. **Handler preference** — `#[wstd::http_server]` (async, wasm32-wasip2) is the recommended handler for new HTTP apps. `#[fastedge::http]` is legacy. New examples must use `wstd`.
2. **No over-engineering** — Simple solutions over complex abstractions. Three similar lines > premature abstraction.
3. **Platform constraints** — Only stdout is captured; `eprintln!` output is silently lost. Flag any use of stderr in code or examples.
4. **CDN/HTTP separation** — CDN apps (proxy-wasm filters) and HTTP apps (standalone handlers) are independent application types with different architectures and lifecycles. Never mix their APIs.
5. **WIT submodule integrity** — `wit/` files come from `G-Core/FastEdge-wit` submodule. Never modify them directly.

### Public API contract

The public API surface is defined by:
- `src/lib.rs` — Core types (`Body`, `Error`), type conversions, `send_request`
- `derive/src/lib.rs` — `#[fastedge::http]` proc macro
- `src/proxywasm/` — ProxyWasm FFI wrappers (KV store, secrets, dictionary, utils)
- `src/http_client.rs` — Outbound HTTP client

Changes to these surfaces require updated `docs/`, updated tests, and a semver-appropriate version bump.

## Generated Content — `docs/`

Files in `docs/` are **machine-generated** from source code by `./fastedge-plugin-source/generate-docs.sh`. They must not be edited by hand — manual changes will be silently overwritten on the next generation run.

### When reviewing PRs that touch `docs/`:

- **Never** suggest manual edits to any file in `docs/`
- If docs are stale or incorrect, suggest: **Run `./fastedge-plugin-source/generate-docs.sh`**
- If the generated output itself is wrong (e.g., wrong structure, missing section), the fix belongs in `fastedge-plugin-source/.generation-config.md`, not in `docs/` directly
- If a PR modifies `docs/` files without a corresponding source code change, flag it — the change should come from the generation script, not a hand-edit

### When reviewing PRs that change source code covered by `docs/`:

- Check whether the change affects the public API or user-facing behavior
- If yes, and `docs/` was not regenerated in the same PR, **request changes** with:
  > Source code affecting public API was changed but docs/ was not regenerated.
  > Run: `./fastedge-plugin-source/generate-docs.sh`

## Documentation Freshness

### Public API changes (must regenerate docs/)
- New, modified, or removed public types/functions in `src/lib.rs`
- Changes to `#[fastedge::http]` macro behavior in `derive/src/lib.rs`
- Changes to ProxyWasm wrapper APIs in `src/proxywasm/`
- Changes to outbound HTTP client in `src/http_client.rs`
- New or modified WIT interfaces in `wit/`
- Changes to `Cargo.toml` (version, features, dependencies)

### Mapping: code location → doc file

| Code path                                     | Doc file              |
| --------------------------------------------- | --------------------- |
| `src/lib.rs` (Body, Error, send_request)      | `docs/SDK_API.md`     |
| `derive/src/lib.rs` (handler macros)          | `docs/SDK_API.md`     |
| `src/http_client.rs` (outbound HTTP)          | `docs/SDK_API.md`     |
| `src/proxywasm/key_value.rs`                  | `docs/HOST_SERVICES.md` |
| `src/proxywasm/secret.rs`                     | `docs/HOST_SERVICES.md` |
| `src/proxywasm/dictionary.rs`                 | `docs/HOST_SERVICES.md` |
| `src/proxywasm/utils.rs`                      | `docs/HOST_SERVICES.md` |
| `src/proxywasm/` (CDN lifecycle, FFI)         | `docs/CDN_APPS.md`    |
| `Cargo.toml` (version, features)              | `docs/INDEX.md`       |
| `fastedge-plugin-source/manifest.json`        | `.github/copilot-instructions.md` |

### Violation example

> PR changes `send_request` signature in `src/lib.rs` but `docs/SDK_API.md` still shows the old signature → **request changes**. Run `./fastedge-plugin-source/generate-docs.sh` before merge.

### Quickstart protection

If any public API signature or behavior changes, check whether `docs/quickstart.md` examples are still accurate. Request regeneration if examples would no longer work against the updated code.

## Pipeline source contract

If `fastedge-plugin-source/manifest.json` lists source files that overlap with files changed in this PR, request that `docs/` is regenerated (run `./fastedge-plugin-source/generate-docs.sh`) to keep the plugin pipeline's source material current.

## Quality Rules

- All public function signatures in docs must match actual source declarations
- No `eprintln!` or `eprint!` in any code or examples — output is lost on the platform
- New HTTP examples must use `#[wstd::http_server]`, not `#[fastedge::http]`
- No marketing language in documentation — precise, technical prose only
