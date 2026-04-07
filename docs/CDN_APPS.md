# FastEdge Rust SDK — CDN Apps (Proxy-Wasm)

Complete guide to building CDN apps (proxy-wasm filters) with the FastEdge Rust SDK.

## CDN Apps vs HTTP Apps

CDN apps run as proxy-wasm filters inside Gcore's CDN proxy layer (Envoy-based). They intercept traffic flowing through the CDN infrastructure rather than receiving requests directly as standalone HTTP handlers.

Key differences from HTTP apps:

| Aspect                | HTTP Apps                                                        | CDN Apps (Proxy-Wasm)                                    |
| --------------------- | ---------------------------------------------------------------- | -------------------------------------------------------- |
| Build target          | `wasm32-wasip1` (basic) / `wasm32-wasip2` (wstd)                | `wasm32-wasip1`                                          |
| Entry point           | `#[wstd::http_server]` (recommended) / `#[fastedge::http]`      | `proxy_wasm::main!` + trait impls                        |
| Request model         | Receives requests directly                                       | Intercepts CDN traffic                                   |
| Response model        | Returns response from handler                                    | Modifies pass-through or short-circuits                  |
| Host services feature | None required                                                    | `features = ["proxywasm"]`                               |
| Crate framework       | `fastedge`                                                       | `proxy-wasm` + optional `fastedge`                       |

CDN apps can inspect and modify requests before they reach origin, and inspect and modify responses before they reach clients. Typical use cases include authentication enforcement, header manipulation, geoblocking, URL rewriting, traffic filtering, and custom caching logic.

## Getting Started

### Cargo.toml

CDN apps come in two tiers depending on whether they need FastEdge host services.

**Tier 1 — Basic CDN app** (no FastEdge host services):

```toml
[package]
name = "my-cdn-app"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
proxy-wasm = "0.2"
log = "0.4"
```

**Tier 2 — CDN app with FastEdge host services** (KV, secrets, dictionary):

```toml
[package]
name = "my-cdn-app"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
proxy-wasm = "0.2"
fastedge = { version = "0.3", features = ["proxywasm"] }
```

The `proxywasm` feature flag is required to access `fastedge::proxywasm::*`. Without it, `fastedge` only exposes Component Model APIs, which are not available in the proxy-wasm environment.

### Build

```sh
cargo build --target wasm32-wasip1 --release
```

CDN apps and basic HTTP apps share the same build target: `wasm32-wasip1`. Only async WASI HTTP apps using `#[wstd::http_server]` target `wasm32-wasip2`.

## Proxy-Wasm Lifecycle

The proxy-wasm lifecycle is the core concept for CDN app development. Every CDN app implements the same three-layer structure: an entry point, a root context, and one or more HTTP contexts.

### Entry Point

The `proxy_wasm::main!` macro initializes the filter. It sets the log level and registers the root context factory function.

```rust,no_run
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> {
        Box::new(MyAppRoot)
    });
}}
```

### Root Context

The root context is a singleton created once when the filter loads. Its primary role is to create a new HTTP context for each incoming request.

```rust,no_run
# use proxy_wasm::traits::*;
# use proxy_wasm::types::*;
struct MyAppRoot;

impl Context for MyAppRoot {}

impl RootContext for MyAppRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(MyApp))
    }
}
```

`get_type()` must return `Some(ContextType::HttpContext)` for HTTP traffic interception. `create_http_context` is called once per request and receives a unique `context_id`.

### HTTP Context

The HTTP context is where request and response processing happens. A new instance is created for each request by `create_http_context`.

```rust,no_run
# use proxy_wasm::traits::*;
# use proxy_wasm::types::*;
struct MyApp;

impl Context for MyApp {}

impl HttpContext for MyApp {
    fn on_http_request_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        self.add_http_response_header("x-powered-by", "FastEdge");
        Action::Continue
    }
}
```

Both `Context` and `HttpContext` must be implemented. The `Context` impl can be empty if no shared context callbacks are needed.

### Lifecycle Callbacks

| Callback                                                          | Phase            | Description                                         |
| ----------------------------------------------------------------- | ---------------- | --------------------------------------------------- |
| `on_http_request_headers(num_headers, end_of_stream) -> Action`   | Request headers  | Inspect or modify request headers before forwarding |
| `on_http_request_body(body_size, end_of_stream) -> Action`        | Request body     | Inspect or modify request body before forwarding    |
| `on_http_response_headers(num_headers, end_of_stream) -> Action`  | Response headers | Inspect or modify response headers from origin      |
| `on_http_response_body(body_size, end_of_stream) -> Action`       | Response body    | Inspect or modify response body from origin         |

All callbacks have default no-op implementations. Override only the phases your app needs to process.

### Action Return Values

Every lifecycle callback returns an `Action` that controls what happens next.

| Action                           | Meaning                                                                    |
| -------------------------------- | -------------------------------------------------------------------------- |
| `Action::Continue`               | Pass the request or response through to the next stage                     |
| `Action::Pause`                  | Stop processing; used after `send_http_response` to short-circuit origin   |
| `Action::StopIterationAndBuffer` | Buffer the current body chunk; continue accumulating until `end_of_stream` |

For body callbacks, return `Action::StopIterationAndBuffer` until `end_of_stream` is `true`, then process the full body and return `Action::Continue`.

```rust,no_run
# use proxy_wasm::traits::*;
# use proxy_wasm::types::*;
# struct MyApp;
# impl Context for MyApp {}
impl HttpContext for MyApp {
    fn on_http_response_body(&mut self, _body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::StopIterationAndBuffer;
        }
        // process complete body here
        Action::Continue
    }
}
```

## Request and Response Manipulation

### Reading Headers and Properties

```rust,no_run
# use proxy_wasm::traits::*;
# use proxy_wasm::types::*;
# struct MyApp;
# impl Context for MyApp {}
impl HttpContext for MyApp {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        // Read a request header
        if let Some(auth) = self.get_http_request_header("Authorization") {
            // use auth value
            let _ = auth;
        }

        // Read a request property
        if let Some(path_bytes) = self.get_property(vec!["request.path"]) {
            if let Ok(path) = std::str::from_utf8(&path_bytes) {
                // use path
                let _ = path;
            }
        }

        Action::Continue
    }
}
```

Properties return `Option<Vec<u8>>` and must be decoded to a string as needed.

### Modifying Headers

```rust,no_run
# use proxy_wasm::traits::*;
# use proxy_wasm::types::*;
# struct MyApp;
# impl Context for MyApp {}
impl HttpContext for MyApp {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        // Add a new request header (does not replace existing)
        self.add_http_request_header("x-forwarded-app", "my-filter");
        // Set (replace) a request header
        self.set_http_request_header("x-request-id", Some("abc-123"));
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        // Add a new response header
        self.add_http_response_header("x-powered-by", "FastEdge");
        // Remove a response header
        self.set_http_response_header("server", None);
        Action::Continue
    }
}
```

### Generating Responses

To short-circuit the request and respond directly to the client without forwarding to origin, call `send_http_response` and return `Action::Pause`.

```rust,no_run
# use proxy_wasm::traits::*;
# use proxy_wasm::types::*;
# struct MyApp;
# impl Context for MyApp {}
impl HttpContext for MyApp {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let authorized = false; // replace with actual check
        if !authorized {
            self.send_http_response(
                401,
                vec![("content-type", "text/plain")],
                Some(b"Unauthorized"),
            );
            return Action::Pause;
        }
        Action::Continue
    }
}
```

`send_http_response` signature: `fn send_http_response(&self, status_code: u32, headers: Vec<(&str, &str)>, body: Option<&[u8]>)`

### Request Properties

CDN apps access request metadata through `self.get_property(vec![...])`. The return type is `Option<Vec<u8>>`.

| Property path         | Description                                    |
| --------------------- | ---------------------------------------------- |
| `["request.path"]`    | Request URL path                               |
| `["request.query"]`   | Query string                                   |
| `["request.country"]` | Client country code (geo-IP lookup)            |
| `["response.status"]` | Response status code (response phase only)     |

```rust,no_run
# use proxy_wasm::traits::*;
# use proxy_wasm::types::*;
# struct MyApp;
# impl Context for MyApp {}
impl HttpContext for MyApp {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let country = self
            .get_property(vec!["request.country"])
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_default();

        if country == "XX" {
            self.send_http_response(403, vec![], Some(b"Forbidden"));
            return Action::Pause;
        }
        Action::Continue
    }
}
```

## Host Services for CDN Apps

CDN apps access FastEdge host services through the `fastedge::proxywasm` module. These APIs use the ProxyWasm FFI transport instead of the Component Model. Requires `features = ["proxywasm"]` in `Cargo.toml`.

### Key-Value Storage (`fastedge::proxywasm::key_value`)

Provides persistent key-value storage. The API shape mirrors `fastedge::key_value` but communicates via ProxyWasm FFI.

#### `Store`

```rust,ignore
pub struct Store { /* ... */ }
```

| Method                                                       | Return Type                          | Description                                               |
| ------------------------------------------------------------ | ------------------------------------ | --------------------------------------------------------- |
| `Store::new()`                                               | `Result<Self, Error>`                | Open the default store                                    |
| `Store::open(name: &str)`                                    | `Result<Self, Error>`                | Open a named store                                        |
| `Store::get(key: &str)`                                      | `Result<Option<Vec<u8>>, Error>`     | Get the value for a key; `None` if key does not exist     |
| `Store::scan(pattern: &str)`                                 | `Result<Vec<String>, Error>`         | List keys matching a glob-style pattern                   |
| `Store::zrange_by_score(key: &str, min: f64, max: f64)`      | `Result<Vec<(Vec<u8>, f64)>, Error>` | Get sorted-set members with scores between min and max    |
| `Store::zscan(key: &str, pattern: &str)`                     | `Result<Vec<(Vec<u8>, f64)>, Error>` | Scan sorted-set members matching a pattern                |
| `Store::bf_exists(key: &str, item: &str)`                    | `Result<bool, Error>`                | Test whether an item is in a Bloom filter                 |

#### `Error`

```rust,ignore
pub enum Error {
    NoSuchStore,
    AccessDenied,
    Other(String),
}
```

| Variant         | Description                                                 |
| --------------- | ----------------------------------------------------------- |
| `NoSuchStore`   | The store label is not recognized by the host               |
| `AccessDenied`  | The application does not have access to the specified store |
| `Other(String)` | An implementation-specific error (e.g., I/O failure)        |

#### Example — Bloom filter check in request headers phase

```rust,no_run
use fastedge::proxywasm::key_value::Store;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(RateLimitRoot) });
}}

struct RateLimitRoot;
impl Context for RateLimitRoot {}
impl RootContext for RateLimitRoot {
    fn get_type(&self) -> Option<ContextType> { Some(ContextType::HttpContext) }
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(RateLimitFilter))
    }
}

struct RateLimitFilter;
impl Context for RateLimitFilter {}

impl HttpContext for RateLimitFilter {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let store = match Store::open("rate-limits") {
            Ok(s) => s,
            Err(e) => {
                self.send_http_response(500, vec![], Some(format!("{}", e).as_bytes()));
                return Action::Pause;
            }
        };

        let client_ip = self.get_http_request_header("x-forwarded-for")
            .unwrap_or_default();

        match store.bf_exists("blocked-ips", &client_ip) {
            Ok(true) => {
                self.send_http_response(429, vec![], Some(b"Rate limit exceeded"));
                Action::Pause
            }
            _ => Action::Continue,
        }
    }
}
```

### Secret Management (`fastedge::proxywasm::secret`)

Provides access to encrypted secrets stored in the FastEdge platform.

```rust,ignore
pub fn get(key: &str) -> Result<Option<Vec<u8>>, u32>
pub fn get_effective_at(key: &str, at: u32) -> Result<Option<Vec<u8>>, u32>
```

| Function                               | Return Type                    | Description                                        |
| -------------------------------------- | ------------------------------ | -------------------------------------------------- |
| `get(key: &str)`                       | `Result<Option<Vec<u8>>, u32>` | Get the current value of a secret                  |
| `get_effective_at(key: &str, at: u32)` | `Result<Option<Vec<u8>>, u32>` | Get the secret value effective at a Unix timestamp |

**Critical difference from the Component Model version**: The error type is `u32` (a raw host status code), not a typed `Error` enum. Map errors explicitly if you need to distinguish failure causes.

`get_effective_at` is useful for secret rotation: pass a past Unix timestamp to retrieve the version of a secret that was valid at that point in time.

Never log or expose secret values in application output.

#### Example — JWT validation using a secret signing key

```rust,no_run
use fastedge::proxywasm::secret;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(AuthRoot) });
}}

struct AuthRoot;
impl Context for AuthRoot {}
impl RootContext for AuthRoot {
    fn get_type(&self) -> Option<ContextType> { Some(ContextType::HttpContext) }
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(AuthFilter))
    }
}

struct AuthFilter;
impl Context for AuthFilter {}

impl HttpContext for AuthFilter {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let signing_key = match secret::get("JWT_SIGNING_KEY") {
            Ok(Some(key)) => key,
            Ok(None) => {
                self.send_http_response(500, vec![], Some(b"App misconfigured"));
                return Action::Pause;
            }
            Err(_status) => {
                self.send_http_response(500, vec![], Some(b"Secret retrieval failed"));
                return Action::Pause;
            }
        };

        // use signing_key for JWT validation
        let _ = signing_key;
        Action::Continue
    }
}
```

### Dictionary (`fastedge::proxywasm::dictionary`)

Provides read-only key-value lookups for configuration data. Values are returned as `String`.

```rust,ignore
pub fn get(key: &str) -> Option<String>
```

Returns `Some(value)` if the key exists and the value is valid UTF-8, `None` otherwise.

#### Example — Reading upstream configuration

```rust,no_run
use fastedge::proxywasm::dictionary;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(ConfigRoot) });
}}

struct ConfigRoot;
impl Context for ConfigRoot {}
impl RootContext for ConfigRoot {
    fn get_type(&self) -> Option<ContextType> { Some(ContextType::HttpContext) }
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(ConfigFilter))
    }
}

struct ConfigFilter;
impl Context for ConfigFilter {}

impl HttpContext for ConfigFilter {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let timeout_ms = dictionary::get("request_timeout_ms")
            .and_then(|v| v.parse::<u64>().ok())
            .unwrap_or(5000);

        self.add_http_request_header("x-timeout-ms", &timeout_ms.to_string());
        Action::Continue
    }
}
```

### Diagnostics (`fastedge::proxywasm::utils`)

```rust,ignore
pub fn set_user_diag(value: &str)
```

Writes a diagnostic message visible in FastEdge platform logs. Panics if the host returns a non-zero status. Use for debugging and operational monitoring; do not log sensitive values.

#### Example

```rust,no_run
use fastedge::proxywasm::utils;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(DiagRoot) });
}}

struct DiagRoot;
impl Context for DiagRoot {}
impl RootContext for DiagRoot {
    fn get_type(&self) -> Option<ContextType> { Some(ContextType::HttpContext) }
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(DiagFilter))
    }
}

struct DiagFilter;
impl Context for DiagFilter {}

impl HttpContext for DiagFilter {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        utils::set_user_diag("request received");
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        utils::set_user_diag("response forwarded");
        Action::Continue
    }
}
```

### Environment Variables

CDN apps read non-secret configuration via `std::env::var()`. This works identically to HTTP apps — no proxy-wasm-specific API is involved.

```rust,no_run
use std::env;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(EnvRoot) });
}}

struct EnvRoot;
impl Context for EnvRoot {}
impl RootContext for EnvRoot {
    fn get_type(&self) -> Option<ContextType> { Some(ContextType::HttpContext) }
    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(EnvFilter))
    }
}

struct EnvFilter;
impl Context for EnvFilter {}

impl HttpContext for EnvFilter {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let Ok(blocklist) = env::var("COUNTRY_BLOCKLIST") else {
            self.send_http_response(500, vec![], Some(b"App misconfigured"));
            return Action::Pause;
        };

        let country = self
            .get_property(vec!["request.country"])
            .and_then(|b| String::from_utf8(b).ok())
            .unwrap_or_default();

        if blocklist.split(',').any(|c| c.eq_ignore_ascii_case(&country)) {
            self.send_http_response(403, vec![], Some(b"Forbidden"));
            return Action::Pause;
        }

        Action::Continue
    }
}
```

For sensitive configuration, use `fastedge::proxywasm::secret::get()` instead of environment variables.

### Logging

CDN apps can write log output using `println!` or the `proxy_wasm::hostcalls::log` function:

```rust,no_run
use proxy_wasm::hostcalls;
use proxy_wasm::types::LogLevel;

// Direct stdout write
println!("Request received");

// Proxy-wasm log API (routes through the configured log level)
hostcalls::log(LogLevel::Info, "Request received").ok();
```

The `log` crate macros (`info!`, `warn!`, `error!`, etc.) work when `proxy_wasm::set_log_level()` is configured in the entry point, which routes them through the proxy-wasm log infrastructure.

**Platform constraint**: Only stdout is captured by the FastEdge platform log viewer. Output written to stderr via `eprint!` or `eprintln!` is silently discarded and will not appear in logs. Always use `println!`, `log::info!`, or `proxy_wasm::hostcalls::log` for any output you need to observe.

## API Comparison: HTTP vs CDN

| Service       | HTTP Apps (Component Model)                                         | CDN Apps (ProxyWasm)                                     |
| ------------- | ------------------------------------------------------------------- | -------------------------------------------------------- |
| Key-Value     | `fastedge::key_value::Store`                                        | `fastedge::proxywasm::key_value::Store`                  |
| Secrets       | `fastedge::secret::get`                                             | `fastedge::proxywasm::secret::get`                       |
| Dictionary    | `fastedge::dictionary::get`                                         | `fastedge::proxywasm::dictionary::get`                   |
| Diagnostics   | `fastedge::utils::set_user_diag`                                    | `fastedge::proxywasm::utils::set_user_diag`              |
| Error types   | Typed `Error` enums                                                 | `u32` status codes (secret) or typed `Error` (key_value) |
| Cargo feature | None required                                                       | `features = ["proxywasm"]`                               |
| Build target  | `wasm32-wasip1` (basic) / `wasm32-wasip2` (wstd)                    | `wasm32-wasip1`                                          |
| Handler       | `#[wstd::http_server]` (recommended) / `#[fastedge::http]` (basic) | `proxy_wasm::main!` + traits                             |

## See Also

- [SDK_API.md](SDK_API.md) — HTTP app handler macro, `Body` type, outbound HTTP (`send_request`)
- [HOST_SERVICES.md](HOST_SERVICES.md) — Component Model host services (KV, secrets, dictionary) for HTTP apps
