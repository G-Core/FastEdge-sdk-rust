# AGENTS.md - AI Coding Agent Guide

## Project Overview

**FastEdge Rust SDK** is a library for building edge computing applications using WebAssembly. It provides a dual-API approach supporting both the WebAssembly Component Model and ProxyWasm specifications.

### Quick Facts

- **Language**: Rust (Edition 2021)
- **Target**: `wasm32-wasip1` (WebAssembly System Interface Preview 1)
- **License**: Apache-2.0
- **Current Version**: 0.3.2
- **Primary Maintainer**: G-Core (FastEdge Development Team)
- **Repository**: https://github.com/G-Core/FastEdge-sdk-rust

---

## Project Structure

```
FastEdge-sdk-rust/
├── Cargo.toml              # Workspace manifest
├── src/                    # Core SDK implementation
│   ├── lib.rs             # Main library entry point
│   ├── http_client.rs     # Outbound HTTP client implementation
│   ├── helper.rs          # Internal helper functions
│   └── proxywasm/         # ProxyWasm API implementations
│       ├── mod.rs
│       ├── key_value.rs
│       ├── secret.rs
│       ├── dictionary.rs
│       └── utils.rs
├── derive/                 # Procedural macros
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs         # #[fastedge::http] attribute macro
├── wit/                    # WebAssembly Interface Types definitions
│   ├── world.wit          # Main world definition
│   ├── http-handler.wit   # HTTP handler interface
│   ├── http-client.wit    # HTTP client interface
│   ├── key-value.wit      # Key-value store interface
│   ├── secret.wit         # Secret management interface
│   ├── dictionary.wit     # Dictionary interface
│   └── utils.wit          # Utility functions
├── examples/               # Example applications
│   ├── backend/           # Backend proxy example
│   ├── key-value/         # Key-value store usage
│   ├── secret/            # Secret access example
│   ├── markdown-render/   # Markdown to HTML converter
│   ├── api-wrapper/       # API wrapping example
│   ├── watermark/         # Image watermarking
│   ├── print/             # Simple print example
│   └── dummy/             # Minimal example
└── wasi-nn/               # WASI Neural Network interface (submodule)
```

---

## Architecture & Design Patterns

### 1. Component Model vs ProxyWasm

The SDK supports two runtime models:

**Component Model (Default)**:
- Uses WIT (WebAssembly Interface Types) bindings via `wit-bindgen`
- Modern WebAssembly component model
- Type-safe interfaces
- Generated bindings in `src/lib.rs` via `wit_bindgen::generate!` macro

**ProxyWasm (Feature Flag)**:
- Enabled with `features = ["proxywasm"]`
- Uses FFI (Foreign Function Interface) with `extern "C"` functions
- Compatible with Envoy and other proxy-wasm hosts
- Implementation in `src/proxywasm/` directory

### 2. Core Design Patterns

#### Attribute Macro Pattern

The `#[fastedge::http]` macro transforms a regular Rust function into a WebAssembly component export:

```rust
// User writes:
#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> { ... }

// Macro generates:
struct Component;
impl Guest for Component {
    fn process(req: ::fastedge::http_handler::Request) -> ::fastedge::http_handler::Response {
        // Converts bindgen types to http crate types
        // Calls user function
        // Converts result back to bindgen types
    }
}
```

**Location**: `derive/src/lib.rs`

#### Type Conversion Pattern

The SDK bridges between three type systems:
1. Standard Rust `http` crate types (user-facing)
2. WIT-generated bindgen types (runtime interface)
3. Internal `Body` type with content-type awareness

**Key Conversions** (`src/lib.rs` lines 200-275):
- `impl From<Method> for ::http::Method`
- `impl TryFrom<Request> for ::http::Request<body::Body>`
- `impl From<::http::Response<body::Body>> for Response`
- `impl TryFrom<Response> for ::http::Response<body::Body>`

#### Body Type Pattern

The `Body` type wraps `bytes::Bytes` and tracks content type:

```rust
pub struct Body {
    pub(crate) content_type: String,
    pub(crate) inner: Bytes,
}
```

**Key Features**:
- Implements `Deref` to `Bytes` for transparent access
- Automatic content-type assignment based on input type
- Optional JSON support via feature flag
- Factory methods: `empty()`, `from()` conversions

---

## WIT Interface Definitions

### World Definition

**File**: `wit/world.wit`

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

### Key Interfaces

#### HTTP Handler (`wit/http-handler.wit`)
```wit
interface http-handler {
    use http.{request, response};
    process: func(req: request) -> response;
}
```

#### Key-Value Store (`wit/key-value.wit`)
- Resource-based API (`resource store`)
- Operations: `open`, `get`, `scan`, `zrange-by-score`, `zscan`, `bf-exists`
- Errors: `no-such-store`, `access-denied`, `internal-error`

#### Secret (`wit/secret.wit`)
- `get(key: string) -> result<option<value>, error>`
- `get-effective-at(key: string, at: u32) -> result<option<value>, error>`
- Supports versioned secrets with time-based retrieval

---

## Dependencies

### Core Dependencies

```toml
fastedge-derive = { path = "derive", version = "0.3" }  # Procedural macros
http = "1.3"                                            # HTTP types
bytes = "1.10"                                          # Byte buffer
wit-bindgen = "0.46"                                    # WIT bindings generator
thiserror = "2.0"                                       # Error derive macros
mime = "^0.3"                                           # MIME type constants
serde_json = { version = "^1.0", optional = true }     # JSON support
```

### Important Considerations

1. **http crate**: Version 1.3+ required for modern HTTP types
2. **wit-bindgen**: Version 0.46 - must match Wasmtime runtime version
3. **bytes**: Used for zero-copy buffer management
4. **serde_json**: Only included with `json` feature flag

---

## Feature Flags

### Available Features

```toml
[features]
default = ["proxywasm"]
proxywasm = []              # Enable ProxyWasm compatibility layer
json = ["serde_json"]       # Enable JSON body support
```

### Usage Patterns

**ProxyWasm Feature**:
```rust
#[cfg(feature = "proxywasm")]
pub mod proxywasm;  // Conditionally compiled
```

**JSON Feature**:
```rust
#[cfg(feature = "json")]
impl TryFrom<serde_json::Value> for Body {
    type Error = serde_json::Error;
    fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
        Ok(Body {
            content_type: mime::APPLICATION_JSON.to_string(),
            inner: Bytes::from(serde_json::to_vec(&value)?),
        })
    }
}
```

---

## Common Development Patterns

### 1. Creating a New Example

```bash
# Create example directory
cd examples
mkdir my-example
cd my-example

# Create Cargo.toml
cat > Cargo.toml << 'EOF'
[package]
name = "my-example"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
fastedge = { path = "../.." }
anyhow = "1.0"
EOF

# Create src/lib.rs
mkdir src
cat > src/lib.rs << 'EOF'
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Hello from example"))
        .map_err(Into::into)
}
EOF
```

### 2. Building Examples

```bash
# From workspace root
cargo build --target wasm32-wasip1 --release --package my-example

# Output location
ls target/wasm32-wasip1/release/my_example.wasm
```

### 3. Adding New WIT Interfaces

When adding new WIT interfaces:

1. Create `.wit` file in `wit/` directory
2. Update `wit/world.wit` to import/export the interface
3. Regenerate bindings by running `cargo build`
4. Add Rust wrapper module in `src/` to expose nice API
5. Add documentation in module-level docs

Example pattern:
```rust
// src/lib.rs
pub mod my_interface {
    #[doc(inline)]
    pub use crate::gcore::fastedge::my_interface::MyFunction;
}
```

### 4. Error Handling Pattern

Always use `Result` types with `anyhow` for application code:

```rust
use anyhow::{Result, anyhow, Context};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let value = some_operation()
        .context("Failed to perform operation")?;
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(value))?)
}
```

### 5. Testing Pattern

Unit tests should focus on business logic, not the handler:

```rust
// lib.rs
fn process_data(input: &str) -> Result<String> {
    // Business logic here
}

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let input = String::from_utf8_lossy(&req.body());
    let output = process_data(&input)?;
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(output))?)
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_process_data() {
        let result = process_data("input").unwrap();
        assert_eq!(result, "expected");
    }
}
```

---

## Code Generation

### WIT Binding Generation

The SDK uses `wit-bindgen` to generate Rust code from WIT files:

**In `src/lib.rs`**:
```rust
wit_bindgen::generate!({
    world: "reactor",
    path: "wit",
    pub_export_macro: true,
});
```

This generates:
- Type definitions matching WIT interfaces
- Import function signatures
- Export trait definitions
- Conversion utilities

**Generated modules** (not in source tree):
- `gcore::fastedge::http`
- `gcore::fastedge::http_client`
- `gcore::fastedge::key_value`
- `gcore::fastedge::secret`
- `gcore::fastedge::dictionary`
- `gcore::fastedge::utils`
- `exports::gcore::fastedge::http_handler`

### Procedural Macro Implementation

**File**: `derive/src/lib.rs`

The `#[fastedge::http]` macro uses `syn` for parsing and `quote` for code generation:

```rust
#[proc_macro_attribute]
pub fn http(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;
    
    quote!(
        // Generated code structure:
        // 1. Define internal_error helper
        // 2. Preserve original function with #[no_mangle]
        // 3. Implement Guest trait
        // 4. Export component
    ).into()
}
```

**Key aspects**:
- Preserves original function for potential testing
- Wraps in error handling
- Converts types between user API and bindgen API
- Exports via `fastedge::export!` macro

---

## ProxyWasm Implementation

When `proxywasm` feature is enabled, the SDK provides FFI bindings to ProxyWasm host functions.

### FFI Functions (`src/proxywasm/mod.rs`)

```rust
extern "C" {
    fn proxy_secret_get(...) -> u32;
    fn proxy_secret_get_effective_at(...) -> u32;
    fn proxy_dictionary_get(...) -> u32;
    fn proxy_kv_store_open(...) -> u32;
    fn proxy_kv_store_get(...) -> u32;
    fn proxy_kv_store_zrange_by_score(...) -> u32;
    fn proxy_kv_store_scan(...) -> u32;
    // etc.
}
```

### Wrapper Pattern

Each ProxyWasm module wraps FFI calls in safe Rust:

```rust
// src/proxywasm/secret.rs
pub fn get(key: &str) -> Result<Option<Vec<u8>>, Error> {
    let mut return_data: *mut u8 = std::ptr::null_mut();
    let mut return_size: usize = 0;
    
    let status = unsafe {
        proxy_secret_get(
            key.as_ptr(),
            key.len(),
            &mut return_data,
            &mut return_size,
        )
    };
    
    // Convert status code to Result
    // Copy data from host memory
    // Return safe Rust types
}
```

---

## Build System

### Workspace Configuration

**Root `Cargo.toml`**:
```toml
[workspace]
members = ["derive", ".", "examples/*"]

[workspace.package]
version = "0.3.2"
edition = "2021"
# ... shared metadata
```

**Benefits**:
- Single version number across all crates
- Shared dependency resolution
- Unified build commands

### Build Targets

#### Development Build
```bash
cargo build --target wasm32-wasip1
```

#### Release Build
```bash
cargo build --target wasm32-wasip1 --release
```

#### Build Specific Example
```bash
cargo build --target wasm32-wasip1 --release --package backend
```

#### Check Without Building
```bash
cargo check --target wasm32-wasip1
```

### Output Locations

- Debug: `target/wasm32-wasip1/debug/*.wasm`
- Release: `target/wasm32-wasip1/release/*.wasm`

---

## Error Types

### SDK Error Type (`src/lib.rs`)

```rust
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("method `{0}` is not supported")]
    UnsupportedMethod(::http::Method),
    
    #[error("http error: {0}")]
    BindgenHttpError(#[from] HttpError),
    
    #[error("http error: {0}")]
    HttpError(#[from] ::http::Error),
    
    #[error("invalid http body")]
    InvalidBody,
    
    #[error("invalid status code {0}")]
    InvalidStatusCode(u16),
}
```

### Module-Specific Errors

**Key-Value Store**:
```rust
variant error {
    no-such-store,
    access-denied,
    internal-error,
}
```

**Secret**:
```rust
variant error {
    access-denied,
    decrypt-error,
    other(string),
}
```

---

## Testing Strategy

### Unit Tests

Test business logic separately from handlers:

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_logic() {
        // Test pure functions
    }
}
```

### Integration Tests

Integration tests would require a FastEdge runtime environment. Consider:
- Using `wasmtime` CLI for testing
- Mocking WIT interfaces
- Testing individual functions, not the full handler

---

## Documentation Standards

### Module-Level Documentation

```rust
//! # Module Name
//!
//! Brief description.
//!
//! ## Example
//!
//! ```rust
//! use fastedge::module::function;
//! // Example code
//! ```
```

### Function Documentation

```rust
/// Brief description of function.
///
/// # Arguments
///
/// * `param` - Description of parameter
///
/// # Returns
///
/// Description of return value
///
/// # Errors
///
/// When this function errors and why
///
/// # Example
///
/// ```
/// let result = function(param);
/// ```
pub fn function(param: Type) -> Result<ReturnType> {
    // Implementation
}
```

### Re-exports

Use `#[doc(inline)]` for re-exported items:

```rust
pub mod dictionary {
    #[doc(inline)]
    pub use crate::gcore::fastedge::dictionary::get;
}
```

---

## Common Issues & Solutions

### Issue: WIT Binding Conflicts

**Problem**: Changes to `.wit` files not reflected in build

**Solution**:
```bash
cargo clean
cargo build --target wasm32-wasip1
```

### Issue: Type Conversion Errors

**Problem**: `InvalidBody` or `InvalidStatusCode` errors

**Solution**: Ensure proper error handling when converting between types:
```rust
let response = Response::builder()
    .status(StatusCode::OK)
    .body(body)?;  // Use ? to propagate http::Error
```

### Issue: ProxyWasm FFI Crashes

**Problem**: Segfaults when using ProxyWasm features

**Solution**:
- Check pointer validity before dereferencing
- Ensure proper memory management (host vs guest memory)
- Validate return codes from FFI calls

### Issue: Example Won't Build

**Problem**: `unresolved import` errors in examples

**Solution**: Ensure example's `Cargo.toml` has correct dependencies:
```toml
[dependencies]
fastedge = { path = "../.." }
```

---

## Contribution Guidelines

### Adding New Features

1. **Update WIT interfaces** if adding host capabilities
2. **Implement in `src/`** following existing patterns
3. **Add ProxyWasm support** if applicable (`src/proxywasm/`)
4. **Create example** demonstrating the feature
5. **Update documentation** in module docs and README
6. **Add tests** where possible

### Code Style

- Follow Rust standard style (`rustfmt`)
- Use meaningful variable names
- Add inline comments for complex logic
- Keep functions focused and small
- Prefer explicit error handling over `unwrap()`

### Commit Messages

Follow conventional commits:
- `feat: Add new capability`
- `fix: Resolve issue with...`
- `docs: Update documentation for...`
- `refactor: Improve structure of...`
- `test: Add tests for...`

---

## Deployment

### Building for Production

```bash
# Full release build
cargo build --target wasm32-wasip1 --release --workspace

# Optimize WASM size (requires wasm-opt from binaryen)
wasm-opt -Oz -o output.wasm input.wasm
```

### Size Optimization Tips

1. Use `opt-level = "z"` or `"s"` in `Cargo.toml`:
```toml
[profile.release]
opt-level = "z"
lto = true
codegen-units = 1
```

2. Remove debug info:
```toml
[profile.release]
strip = true
```

3. Consider `wee_alloc` for smaller allocator (if needed)

---

## Version History

- **0.3.2** (Current): Latest stable release
- **0.3.x**: Refinements and bug fixes
- **0.2.x**: Added ProxyWasm support
- **0.1.x**: Initial release with Component Model

---

## Key Contacts & Resources

- **Repository**: https://github.com/G-Core/FastEdge-sdk-rust
- **Documentation**: https://docs.rs/fastedge
- **Platform Docs**: https://gcore.com/docs/fastedge
- **Maintainer**: FastEdge Development Team <fastedge@gcore.com>

---

## Quick Reference

### Essential Commands

```bash
# Setup
rustup target add wasm32-wasip1

# Build
cargo build --target wasm32-wasip1 --release

# Check
cargo check --target wasm32-wasip1

# Test (Rust tests only, no WASM)
cargo test

# Build specific example
cargo build --target wasm32-wasip1 --release --package example-name

# Format code
cargo fmt

# Lint
cargo clippy --target wasm32-wasip1
```

### Key Files for AI Agents

When making changes, these files are most commonly edited:

1. **`src/lib.rs`** - Core SDK implementation
2. **`wit/*.wit`** - Interface definitions
3. **`derive/src/lib.rs`** - Attribute macro
4. **`examples/*/src/lib.rs`** - Example applications
5. **`Cargo.toml`** - Dependencies and metadata

### Import Patterns

```rust
// Standard handler
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

// HTTP client
use fastedge::send_request;
use fastedge::http::Method;

// Key-value store
use fastedge::key_value::{Store, Error as StoreError};

// Secrets
use fastedge::secret;

// Dictionary
use fastedge::dictionary;

// Utilities
use fastedge::utils::set_user_diag;
```

---

**This document is intended for AI coding agents to understand the FastEdge Rust SDK architecture, patterns, and development practices. It should be updated whenever significant architectural changes are made.**
