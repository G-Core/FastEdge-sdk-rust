# FastEdge Rust SDK - Complete Documentation

## Table of Contents

1. [Overview](#overview)
2. [Architecture](#architecture)
3. [Installation & Setup](#installation--setup)
4. [Core Concepts](#core-concepts)
5. [API Reference](#api-reference)
6. [Examples & Usage Patterns](#examples--usage-patterns)
7. [Advanced Topics](#advanced-topics)
8. [Troubleshooting](#troubleshooting)

---

## Overview

The FastEdge Rust SDK is a comprehensive toolkit for building high-performance edge computing applications using WebAssembly (WASM). It supports both the WebAssembly Component Model and the ProxyWasm API, providing flexibility for different deployment scenarios.

### Key Features

- **HTTP Request Handling**: Process incoming HTTP requests at the edge
- **Outbound HTTP Client**: Make HTTP requests to backend services
- **Key-Value Storage**: Persistent storage interface for application data
- **Secret Management**: Secure access to encrypted secrets and credentials
- **Dictionary Interface**: Fast key-value lookups for configuration
- **Dual Runtime Support**: Works with both Component Model and ProxyWasm
- **Machine Learning**: WASI-NN integration for ML inference at the edge

### Version Information

- **Current Version**: 0.3.2
- **License**: Apache-2.0
- **Documentation**: https://docs.rs/fastedge
- **Repository**: https://github.com/G-Core/FastEdge-sdk-rust

---

## Architecture

### Component Model Architecture

The FastEdge SDK is built on the WebAssembly Component Model, which provides:

```
┌─────────────────────────────────────┐
│   FastEdge Application              │
│   (Your Rust Code)                  │
└──────────────┬──────────────────────┘
               │ #[fastedge::http]
               ▼
┌─────────────────────────────────────┐
│   FastEdge SDK (fastedge crate)     │
│   - HTTP Handler                    │
│   - HTTP Client                     │
│   - Key-Value Store                 │
│   - Secret Management               │
│   - Dictionary                      │
└──────────────┬──────────────────────┘
               │ WIT Bindings
               ▼
┌─────────────────────────────────────┐
│   Wasmtime Runtime                  │
│   (FastEdge Platform)               │
└─────────────────────────────────────┘
```

### WIT Interfaces

The SDK uses WebAssembly Interface Types (WIT) to define the contract between your application and the runtime:

**World Definition** (`wit/world.wit`):
```wit
package gcore:fastedge;

world reactor {
    import http;
    import http-client;
    import dictionary;
    import secret;
    import key-value;
    import utils;

    export http-handler;
}
```

### ProxyWasm Architecture

For environments using ProxyWasm, the SDK provides a compatibility layer:

```rust
#[cfg(feature = "proxywasm")]
pub mod proxywasm;
```

This allows the same application code to work in both environments.

---

## Installation & Setup

### Prerequisites

1. **Rust Toolchain**: Install from https://rustup.rs
2. **WASM Target**: Add the wasm32-wasip1 target

```bash
rustup target add wasm32-wasip1
```

### Adding to Your Project

Add to your `Cargo.toml`:

```toml
[dependencies]
fastedge = "0.3"
anyhow = "1.0"  # For error handling

# Optional features
# fastedge = { version = "0.3", features = ["json"] }
```

### Project Structure

Create a new library project:

```bash
cargo new --lib my-fastedge-app
cd my-fastedge-app
```

Configure `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
fastedge = "0.3"
anyhow = "1.0"
```

### Building

Build for the WASM target:

```bash
cargo build --target wasm32-wasip1 --release
```

The compiled WASM binary will be at:
```
target/wasm32-wasip1/release/my_fastedge_app.wasm
```

---

## Core Concepts

### The HTTP Handler

Every FastEdge application starts with an HTTP handler function decorated with `#[fastedge::http]`:

```rust
use fastedge::http::{Request, Response, StatusCode};
use fastedge::body::Body;
use anyhow::Result;

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    // Your application logic here
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Hello, World!"))
        .map_err(Into::into)
}
```

### Request and Response Types

The SDK uses the standard Rust `http` crate types:

- `http::Request<Body>`: Incoming HTTP request
- `http::Response<Body>`: Outgoing HTTP response
- `fastedge::body::Body`: Request/response body with content-type handling

### Body Type

The `Body` type is a wrapper around `bytes::Bytes` with content-type awareness:

```rust
// Create bodies from different types
let text_body = Body::from("Hello");
let bytes_body = Body::from(vec![1, 2, 3]);
let empty_body = Body::empty();

// With JSON feature enabled
#[cfg(feature = "json")]
let json_body = Body::try_from(serde_json::json!({"key": "value"}))?;

// Access body data
let bytes: &Bytes = &body;  // Deref to Bytes
let content_type = body.content_type();
```

### Error Handling

The SDK uses the `anyhow` crate for error handling:

```rust
use anyhow::{Result, anyhow};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let query = req.uri().query()
        .ok_or(anyhow!("Missing query parameter"))?;
    
    // Your logic here
    
    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Success"))?)
}
```

---

## API Reference

### HTTP Module

#### Sending HTTP Requests

```rust
use fastedge::send_request;
use fastedge::http::{Method, Request};

// Create a request
let request = Request::builder()
    .method(Method::GET)
    .uri("https://api.example.com/data")
    .header("User-Agent", "FastEdge/1.0")
    .body(Body::empty())?;

// Send the request
let response = send_request(request)?;

// Process the response
let status = response.status();
let body_bytes = response.body();
```

### Key-Value Storage

The key-value store provides persistent storage with advanced features:

```rust
use fastedge::key_value::{Store, Error};

// Open a store
let store = Store::open("my-store")?;

// Basic operations
let value = store.get("key")?;
if let Some(data) = value {
    // Process data
}

// Scan with pattern matching
let keys = store.scan("user:*")?;

// Sorted set operations
let results = store.zrange_by_score("leaderboard", 0.0, 100.0)?;
for (value, score) in results {
    println!("Value: {:?}, Score: {}", value, score);
}

// Bloom filter check
let exists = store.bf_exists("filter", "item")?;
```

#### Error Handling

```rust
match Store::open("restricted-store") {
    Ok(store) => { /* use store */ },
    Err(Error::AccessDenied) => {
        return Response::builder()
            .status(StatusCode::FORBIDDEN)
            .body(Body::from("Access denied"))
            .map_err(Into::into);
    },
    Err(Error::NoSuchStore) => {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::from("Store not found"))
            .map_err(Into::into);
    },
    Err(Error::InternalError) => {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::empty())
            .map_err(Into::into);
    }
}
```

### Secret Management

Securely access encrypted secrets:

```rust
use fastedge::secret;

// Get current secret value
match secret::get("API_KEY") {
    Ok(Some(value)) => {
        let api_key = String::from_utf8_lossy(&value);
        // Use the API key
    },
    Ok(None) => {
        // Secret not found
    },
    Err(secret::Error::AccessDenied) => {
        // Access denied
    },
    Err(secret::Error::DecryptError) => {
        // Decryption failed
    },
    Err(secret::Error::Other(msg)) => {
        // Other error
    }
}

// Get secret effective at a specific time
use std::time::{SystemTime, UNIX_EPOCH};

let timestamp = SystemTime::now()
    .duration_since(UNIX_EPOCH)?
    .as_secs() as u32;

let historical_value = secret::get_effective_at("API_KEY", timestamp)?;
```

### Dictionary

Fast, read-only key-value lookups:

```rust
use fastedge::dictionary;

// Get a value from the dictionary
if let Some(config_value) = dictionary::get("config-key")? {
    let config = String::from_utf8_lossy(&config_value);
    // Use the configuration value
}
```

### Utilities

Diagnostic and statistics functions:

```rust
use fastedge::utils;

// Set custom diagnostic information
utils::set_user_diag("Processing completed successfully");
```

### WASI-NN (Machine Learning)

Integrate machine learning models:

```rust
use fastedge::wasi_nn;

// Load and use ML models
// (Requires WASI-NN compatible runtime)
```

---

## Examples & Usage Patterns

### 1. Simple HTTP Handler

```rust
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from("Hello, FastEdge!"))
        .map_err(Into::into)
}
```

### 2. Backend Proxy

Forward requests to a backend service:

```rust
use anyhow::{anyhow, Result};
use fastedge::body::Body;
use fastedge::http::{Method, Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let query = req.uri().query()
        .ok_or(anyhow!("Missing query parameter"))?;
    
    let params = querystring::querify(query);
    let url = params.iter()
        .find(|(k, _)| k == &"url")
        .ok_or(anyhow!("Missing url parameter"))?
        .1;
    
    let backend_request = Request::builder()
        .method(Method::GET)
        .uri(urlencoding::decode(url)?.to_string())
        .body(req.into_body())?;
    
    let backend_response = fastedge::send_request(backend_request)?;
    
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(
            "Response length: {}, Content-Type: {:?}",
            backend_response.body().len(),
            backend_response.headers().get("Content-Type")
        )))
        .map_err(Into::into)
}
```

### 3. Key-Value Store Operations

```rust
use anyhow::{anyhow, Result};
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};
use fastedge::key_value::Store;

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let query = req.uri().query()
        .ok_or(anyhow!("No query parameters"))?;
    
    let params = querystring::querify(query);
    
    let store_name = params.iter()
        .find(|(k, _)| k == &"store")
        .map(|(_, v)| v)
        .ok_or(anyhow!("Missing 'store' parameter"))?;
    
    let key = params.iter()
        .find(|(k, _)| k == &"key")
        .map(|(_, v)| v)
        .ok_or(anyhow!("Missing 'key' parameter"))?;
    
    let store = Store::open(store_name)?;
    let value = store.get(key)?;
    
    match value {
        Some(data) => {
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from(data))
                .map_err(Into::into)
        },
        None => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Key not found"))
                .map_err(Into::into)
        }
    }
}
```

### 4. Secret Access

```rust
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};
use fastedge::secret;

#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    match secret::get("DATABASE_URL") {
        Ok(Some(secret_value)) => {
            let db_url = String::from_utf8_lossy(&secret_value);
            // Use the database URL to connect
            
            Response::builder()
                .status(StatusCode::OK)
                .body(Body::from("Connected successfully"))
                .map_err(Into::into)
        },
        Ok(None) => {
            Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::from("Secret not configured"))
                .map_err(Into::into)
        },
        Err(secret::Error::AccessDenied) => {
            Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from("Access denied"))
                .map_err(Into::into)
        },
        Err(_) => {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Secret retrieval failed"))
                .map_err(Into::into)
        }
    }
}
```

### 5. Environment Variables

```rust
use std::env;
use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    let base_url = env::var("BASE_URL")
        .unwrap_or_else(|_| "https://default.example.com".to_string());
    
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("Base URL: {}", base_url)))
        .map_err(Into::into)
}
```

### 6. Markdown Renderer

Transform Markdown content to HTML:

```rust
use fastedge::body::Body;
use fastedge::http::{header, Method, Request, Response, StatusCode};
use pulldown_cmark::{Options, Parser};
use std::env;

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>, http::Error> {
    if !matches!(req.method(), &Method::GET | &Method::HEAD) {
        return Response::builder()
            .status(StatusCode::METHOD_NOT_ALLOWED)
            .header(header::ALLOW, "GET, HEAD")
            .body(Body::from("Method not allowed"));
    }
    
    let base = env::var("BASE").unwrap_or_default();
    let path = req.uri().path();
    
    // Fetch Markdown from backend
    let backend_req = Request::builder()
        .method(Method::GET)
        .uri(format!("{}{}", base, path))
        .body(Body::empty())?;
    
    let response = fastedge::send_request(backend_req)
        .map_err(|_| http::Error::from(()))?;
    
    let markdown = String::from_utf8_lossy(response.body()).to_string();
    
    // Convert to HTML
    let parser = Parser::new_ext(
        &markdown,
        Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES
    );
    
    let mut html = String::from("<!DOCTYPE html><html><body>");
    pulldown_cmark::html::push_html(&mut html, parser);
    html.push_str("</body></html>");
    
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime::TEXT_HTML.to_string())
        .body(Body::from(html))
}
```

---

## Advanced Topics

### ProxyWasm Compatibility

When using the `proxywasm` feature, you can access ProxyWasm-specific APIs:

```rust
#[cfg(feature = "proxywasm")]
use fastedge::proxywasm;

// ProxyWasm-specific implementations available
```

The ProxyWasm module provides:
- Key-Value Store operations
- Secret management
- Dictionary access
- Utility functions

All through native ProxyWasm FFI calls.

### Custom Body Types

Implement custom conversions for your types:

```rust
use fastedge::body::Body;
use bytes::Bytes;

struct CustomData {
    data: Vec<u8>,
}

impl From<CustomData> for Body {
    fn from(custom: CustomData) -> Self {
        Body::from(custom.data)
    }
}
```

### JSON Feature

Enable JSON support in your `Cargo.toml`:

```toml
[dependencies]
fastedge = { version = "0.3", features = ["json"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
```

Use JSON bodies:

```rust
use serde_json::json;

#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    let json_response = json!({
        "status": "ok",
        "message": "Success",
        "data": {
            "items": [1, 2, 3]
        }
    });
    
    Response::builder()
        .status(StatusCode::OK)
        .body(Body::try_from(json_response)?)
        .map_err(Into::into)
}
```

### Header Manipulation

```rust
use fastedge::http::header;

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    // Read request headers
    let user_agent = req.headers()
        .get(header::USER_AGENT)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("unknown");
    
    // Build response with custom headers
    Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::CACHE_CONTROL, "max-age=3600")
        .header("X-Custom-Header", "custom-value")
        .body(Body::from(format!("User-Agent: {}", user_agent)))
        .map_err(Into::into)
}
```

### Error Type Conversions

The SDK provides comprehensive error types:

```rust
use fastedge::Error;

// Error variants:
// - UnsupportedMethod(http::Method)
// - BindgenHttpError(HttpError)
// - HttpError(http::Error)
// - InvalidBody
// - InvalidStatusCode(u16)

fn handle_error(err: Error) -> Response<Body> {
    match err {
        Error::UnsupportedMethod(method) => {
            Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .body(Body::from(format!("Method {} not supported", method)))
                .unwrap()
        },
        Error::InvalidStatusCode(code) => {
            Response::builder()
                .status(StatusCode::BAD_GATEWAY)
                .body(Body::from(format!("Invalid status code: {}", code)))
                .unwrap()
        },
        _ => {
            Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("Internal error"))
                .unwrap()
        }
    }
}
```

---

## Troubleshooting

### Common Build Issues

#### Missing WASM Target

**Error**: `error[E0463]: can't find crate for 'std'`

**Solution**:
```bash
rustup target add wasm32-wasip1
```

#### Compilation Errors

**Error**: `linking with 'rust-lld' failed`

**Solution**: Ensure you're building with the correct target:
```bash
cargo build --target wasm32-wasip1 --release
```

### Runtime Issues

#### Store Access Denied

When getting `Error::AccessDenied` from key-value store:

1. Verify the store name is correct
2. Check that your application has been granted access to the store
3. Ensure the store exists in your FastEdge environment

#### Secret Decryption Errors

When getting `Error::DecryptError`:

1. Verify the secret name is correct
2. Check that the secret is properly encrypted in the platform
3. Ensure your application has the correct permissions

### Performance Optimization

#### Minimize Memory Allocations

```rust
// Instead of creating new strings
let response_text = format!("Value: {}", value);

// Consider using static strings when possible
const RESPONSE: &str = "Fixed response";
```

#### Reuse Connections

The HTTP client handles connection pooling internally, but ensure you're not creating unnecessary requests:

```rust
// Good: Single request
let response = fastedge::send_request(request)?;

// Avoid: Multiple redundant requests
```

#### Optimize Body Sizes

```rust
// For large responses, consider streaming or pagination
// instead of loading everything into memory
```

### Debugging

Enable debug output:

```rust
use fastedge::utils::set_user_diag;

set_user_diag(&format!("Processing request: {:?}", req.uri()));
```

Check the FastEdge platform logs for diagnostic messages.

---

## Best Practices

1. **Error Handling**: Always use `Result` types and handle all error cases
2. **Security**: Never log or expose sensitive data from secrets
3. **Performance**: Minimize allocations and avoid blocking operations
4. **Resource Management**: Close stores and resources when done
5. **Testing**: Write unit tests for your business logic separately from the handler
6. **Documentation**: Document your application's expected environment variables and configurations

---

## Further Resources

- [FastEdge Documentation](https://gcore.com/docs/fastedge)
- [WebAssembly Component Model](https://component-model.bytecodealliance.org)
- [Rust HTTP Crate](https://docs.rs/http)
- [Wasmtime Runtime](https://wasmtime.dev/)
- [FastEdge SDK Repository](https://github.com/G-Core/FastEdge-sdk-rust)

---

**Last Updated**: January 2026  
**SDK Version**: 0.3.2
