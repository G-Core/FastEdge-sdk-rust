# Build System and CI

## Workspace Configuration

Root `Cargo.toml` defines a workspace:

```toml
[workspace]
members = ["derive"]

[workspace.package]
version = "0.3.5"
edition = "2021"
license = "Apache-2.0"
```

Both `fastedge` and `fastedge-derive` share the workspace version. Examples are standalone crates with `path = "../.."` dependencies.

---

## Build Target

**Default target**: `wasm32-wasip1` (WebAssembly System Interface Preview 1)

Set in `.cargo/config.toml`:
```toml
[build]
target = "wasm32-wasip1"
```

This means `cargo build` defaults to WASM output without needing `--target`.

---

## Common Commands

| Command | Purpose |
|---------|---------|
| `rustup target add wasm32-wasip1` | One-time setup |
| `cargo build --release` | Release build (WASM) |
| `cargo check` | Type-check without building |
| `cargo clippy --all-targets --all-features` | Lint |
| `cargo fmt` | Format code |
| `cargo test` | Run Rust-native tests (not WASM) |
| `cargo build --release --package <name>` | Build a specific example |
| `cargo doc` | Generate documentation |
| `cargo clean` | Clear build artifacts (useful after WIT changes) |

---

## Build Outputs

- Debug: `target/wasm32-wasip1/debug/*.wasm`
- Release: `target/wasm32-wasip1/release/*.wasm`

Examples build as `cdylib` (dynamic library), producing `.wasm` files.

---

## CI Pipeline (`.github/workflows/ci.yaml`)

Triggered on every push. Steps:

1. **Checkout** with submodules (recursive)
2. **Setup Rust** with `wasm32-wasip1` target
3. **Security audit**: `cargo-audit` on binary crate
4. **Build**: `cargo build --release --all-features`
5. **Documentation**: `cargo doc`
6. **Lint**: `cargo clippy --all-targets --all-features`

Environment: `RUSTFLAGS="-Dwarnings"` — all warnings treated as errors.

---

## Release Pipeline (`.github/workflows/release.yml`)

Triggered on push to `main` or manual `workflow_dispatch`. Two jobs:

### 1. Prepare

- Analyzes conventional commits since last release
- Determines version bump (major/minor/patch)
- Creates a version bump PR

### 2. Publish

- Tags release as `v{version}`
- Creates GitHub release
- Publishes `fastedge-derive` to crates.io first (dependency)
- Publishes `fastedge` to crates.io second

Version management uses `release-plz` (config in `release-plz.toml`).

---

## FOSSA Compliance (`.github/workflows/fossa.yml`)

License compliance checking via FOSSA. Runs on push to ensure all dependencies meet licensing requirements.

---

## Example Build Pattern

Each example is a standalone crate:

```toml
# examples/http/basic/hello_world/Cargo.toml
[package]
name = "hello-world"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
fastedge = { path = "../../../.." }
anyhow = "1.0"
```

Build: `cargo build --release --package hello-world`
Output: `target/wasm32-wasip1/release/hello_world.wasm`

---

## Size Optimization

For production WASM binaries:

```toml
[profile.release]
opt-level = "z"    # Optimize for size
lto = true         # Link-time optimization
codegen-units = 1  # Single codegen unit
strip = true       # Remove debug info
```

Optional post-build: `wasm-opt -Oz -o output.wasm input.wasm` (requires binaryen).

---

**Last Updated**: March 2026
