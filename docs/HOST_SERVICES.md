# FastEdge Rust SDK — Host Services

Host-provided service modules for key-value storage, secret management, configuration dictionaries, and diagnostic utilities.

The host-service modules documented here (`fastedge::key_value`, `fastedge::secret`, `fastedge::dictionary`, `fastedge::utils`) are part of the standard `fastedge` crate. No additional Cargo.toml changes are needed beyond the standard `fastedge` dependency to use these services.

---

## Key-Value Storage

The `fastedge::key_value` module provides persistent storage with support for simple key-value pairs, glob-style key scanning, sorted sets, and bloom filters. Data is organized into named stores; access to a store must be granted via platform configuration.

### Opening a Store

```rust
pub fn new() -> Result<Self, Error>
pub fn open(name: &str) -> Result<Self, Error>
```

`Store::new()` opens the default store. `Store::open(name)` opens a named store. Both return `Err(Error::NoSuchStore)` if the store label is not recognized and `Err(Error::AccessDenied)` if the application is not authorized.

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    // Open the default store
    let _default_store = Store::new()?;

    // Open a named store
    let _named_store = Store::open("user-data")?;

    Ok(Response::builder()
        .status(200)
        .body(Body::from("ok"))?)
}
```

### Reading Values

```rust
pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Error>
```

Returns `Ok(Some(bytes))` if the key exists, `Ok(None)` if it does not.

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = Store::open("user-data")?;

    match store.get("user:123:profile")? {
        Some(data) => Ok(Response::builder()
            .status(200)
            .body(Body::from(data))?),
        None => Ok(Response::builder()
            .status(404)
            .body(Body::from("not found"))?),
    }
}
```

### Pattern Scanning

```rust
pub fn scan(&self, pattern: &str) -> Result<Vec<String>, Error>
```

Scans the store for keys matching a glob-style pattern. Returns a list of matching key names. Returns an empty `Vec` if no keys match.

Supported glob syntax:

| Pattern   | Matches                                     |
| --------- | ------------------------------------------- |
| `*`       | Any sequence of characters within a segment |
| `?`       | Any single character                        |
| `[abc]`   | Any character in the set                    |

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = Store::open("user-data")?;

    let keys = store.scan("user:123:*")?;
    for key in &keys {
        println!("Found key: {}", key);
    }

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!("{} keys found", keys.len())))?)
}
```

### Sorted Sets

Sorted sets store members associated with a `f64` score. Members are ordered from lowest to highest score.

```rust
pub fn zrange_by_score(&self, key: &str, min: f64, max: f64) -> Result<Vec<(Vec<u8>, f64)>, Error>
pub fn zscan(&self, key: &str, pattern: &str) -> Result<Vec<(Vec<u8>, f64)>, Error>
```

`zrange_by_score` returns all members of the sorted set stored at `key` whose score falls in the inclusive range `[min, max]`. Use `f64::NEG_INFINITY` and `f64::INFINITY` for unbounded ranges.

`zscan` returns members of the sorted set at `key` whose member value matches the glob-style `pattern`.

Both return an empty `Vec` when the key does not exist or no members fall within the specified range or pattern.

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = Store::open("game-data")?;

    // Retrieve all leaderboard entries with scores >= 1000
    let top_players = store.zrange_by_score("leaderboard", 1000.0, f64::INFINITY)?;
    for (member, score) in &top_players {
        let name = String::from_utf8_lossy(member);
        println!("Player: {}, Score: {}", name, score);
    }

    // Retrieve sorted set members matching a pattern
    let guild_members = store.zscan("guild:42:members", "player:*")?;
    println!("Guild members found: {}", guild_members.len());

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!("{} top players", top_players.len())))?)
}
```

### Bloom Filters

```rust
pub fn bf_exists(&self, key: &str, item: &str) -> Result<bool, Error>
```

Tests whether `item` is a member of the bloom filter stored at `key`. Returns `true` if the item was probably added to the filter (subject to the false-positive rate of the filter), or `false` if the key does not exist or the item was definitely not added.

Bloom filters cannot produce false negatives: if `bf_exists` returns `false`, the item has not been added.

```rust,no_run
use fastedge::key_value::Store;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = Store::open("rate-limit")?;

    let client_ip = "203.0.113.42";

    if store.bf_exists("blocked_ips", client_ip)? {
        return Ok(Response::builder()
            .status(403)
            .body(Body::from("Blocked"))?);
    }

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

### Error Handling

All `Store` methods return `Result<_, Error>`. The `Error` type has three variants:

| Variant                | Description                                                     |
| ---------------------- | --------------------------------------------------------------- |
| `Error::NoSuchStore`   | The requested store label is not recognized by the host         |
| `Error::AccessDenied`  | The application does not have permission to access the store    |
| `Error::Other(String)` | An implementation-specific error (I/O or internal host failure) |

```rust,no_run
use fastedge::key_value::{Error, Store};
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store = match Store::open("config") {
        Ok(s) => s,
        Err(Error::NoSuchStore) => {
            return Ok(Response::builder()
                .status(500)
                .body(Body::from("Store not configured"))?);
        }
        Err(Error::AccessDenied) => {
            return Ok(Response::builder()
                .status(403)
                .body(Body::from("Access denied"))?);
        }
        Err(Error::Other(msg)) => {
            return Err(anyhow::anyhow!("KV store error: {}", msg));
        }
    };

    let _ = store.get("key")?;

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

CDN apps use `fastedge::proxywasm::key_value` instead — see [CDN_APPS.md](CDN_APPS.md) for the ProxyWasm API surface and usage examples.

---

## Secret Management

The `fastedge::secret` module provides access to encrypted secrets such as API keys, passwords, and certificates. Secrets are encrypted at rest and support versioned retrieval for rotation scenarios.

### Reading Secrets

```rust
pub fn get(key: &str) -> Result<Option<Vec<u8>>, Error>
```

Returns the currently effective value of the named secret. Returns `Ok(None)` if no secret with that name is configured.

```rust,no_run
use fastedge::secret;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let api_key = match secret::get("UPSTREAM_API_KEY")? {
        Some(key) => key,
        None => {
            return Ok(Response::builder()
                .status(500)
                .body(Body::from("API key not configured"))?);
        }
    };

    // Use api_key bytes for authentication — do not log or include in responses
    let _ = api_key;

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

### Time-Based Retrieval

```rust
pub fn get_effective_at(key: &str, at: u32) -> Result<Option<Vec<u8>>, Error>
```

Returns the value of the named secret that was effective at the given Unix timestamp (`at`, seconds since epoch). This is useful during secret rotation: you can verify that both the old and new versions of a secret are accessible before completing a rotation.

Returns `Ok(None)` if no version of the secret was configured at that time.

```rust,no_run
use fastedge::secret;
use std::time::{SystemTime, UNIX_EPOCH};
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    // Retrieve the secret that was valid 5 minutes ago
    let five_minutes_ago = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs() as u32
        - 300;

    let previous_secret = secret::get_effective_at("SIGNING_KEY", five_minutes_ago)?;
    let current_secret = secret::get("SIGNING_KEY")?;

    // Use previous_secret and current_secret for rotation validation
    let _ = (previous_secret, current_secret);

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

### Security Notes

- Never include secret values in HTTP responses, log output, or diagnostic messages.
- Secret values are returned as raw bytes (`Vec<u8>`). Convert to a string only when the secret is defined as UTF-8 text, and handle the conversion error explicitly.
- Access to secrets is controlled by platform configuration. Unauthorized access returns `Err(secret::Error)`, not `Ok(None)`. `Ok(None)` means the secret is not configured or not found — it does not indicate an authorization failure.
- Clear secret material from memory as soon as it is no longer needed. Rust's ownership model helps with this: binding a secret to a local variable ensures it is dropped at the end of its scope.

CDN apps use `fastedge::proxywasm::secret` instead — see [CDN_APPS.md](CDN_APPS.md) for the ProxyWasm API surface and usage examples.

---

## Dictionary

The `fastedge::dictionary` module provides fast, read-only lookups for configuration values that do not change during the lifetime of a deployment.

### Configuration Lookups

```rust
pub fn get(key: &str) -> Option<String>
```

Returns `Some(value)` if the key exists and its value is valid UTF-8, or `None` if the key is not found or the value cannot be decoded as UTF-8.

Dictionary values are environment variables set at deployment time via the platform — the same management mechanism as secrets, but without encryption. They are not writable from application code.

```rust,no_run
use fastedge::dictionary;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let upstream = dictionary::get("upstream_origin")
        .unwrap_or_else(|| "https://default.example.com".to_string());

    let timeout_ms: u64 = dictionary::get("timeout_ms")
        .and_then(|v| v.parse().ok())
        .unwrap_or(5000);

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!(
            "Upstream: {}, Timeout: {}ms",
            upstream, timeout_ms
        )))?)
}
```

### When to Use Dictionary vs Key-Value vs Secrets

| Criterion                    | `dictionary`                           | `key_value`                               | `secret`                                    |
| ---------------------------- | -------------------------------------- | ----------------------------------------- | ------------------------------------------- |
| **Mutability**               | Read-only; set at deployment time      | Read-only from application code           | Read-only; managed by platform              |
| **Value type**               | UTF-8 strings only                     | Arbitrary bytes                           | Arbitrary bytes                             |
| **Advanced data structures** | No                                     | Sorted sets, bloom filters, glob scan     | No                                          |
| **Confidentiality**          | Not encrypted; visible in config       | Not encrypted at the application layer    | Encrypted at rest; access-controlled        |
| **Typical use cases**        | Feature flags, routing config, tuning  | Caching, counters, state, rate-limit data | API keys, tokens, certificates, credentials |
| **Versioning / rotation**    | No                                     | No                                        | Yes, via `get_effective_at`                 |

Use `dictionary` for simple, non-sensitive string configuration that is known at deployment time. Use `key_value` for larger datasets, binary values, or data that requires advanced query patterns. Use `secret` for any value that must be kept confidential.

CDN apps use `fastedge::proxywasm::dictionary` instead — see [CDN_APPS.md](CDN_APPS.md) for the ProxyWasm API surface and usage examples.

---

## Utilities

The `fastedge::utils` module provides diagnostic functions for monitoring and debugging edge applications.

### Diagnostics

```rust
pub fn set_user_diag(value: &str)
```

Writes a diagnostic string that appears in the FastEdge platform logs associated with the current request. This is intended for debugging and operational monitoring. There is no return value; the function panics if the host rejects the call.

The FastEdge platform captures only **stdout** for application log output. `stderr` is silently discarded and will not appear in the platform's log viewer. Use `println!` (or logging crates that write to stdout) for any output you need to observe. Do not use `eprintln!` — it produces no visible output on the platform.

```rust,no_run
use fastedge::key_value::Store;
use fastedge::utils::set_user_diag;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    set_user_diag("handler entered");

    let store = Store::open("cache")?;

    match store.get("config:version")? {
        Some(v) => {
            set_user_diag(&format!("config version: {}", String::from_utf8_lossy(&v)));
        }
        None => {
            set_user_diag("config version: not found");
        }
    }

    Ok(Response::builder()
        .status(200)
        .body(Body::empty())?)
}
```

One diagnostic message per request is the typical pattern. If `set_user_diag` is called multiple times, the platform may record only the last value or concatenate them depending on runtime behavior.

Do not write sensitive values (secrets, credentials, personally identifiable information) to diagnostics, as the output appears in platform logs that may be accessible to operations personnel.

CDN apps use `fastedge::proxywasm::utils` instead — see [CDN_APPS.md](CDN_APPS.md) for the ProxyWasm API surface and usage examples.

---

## See Also

- [SDK_API.md](SDK_API.md) — Core HTTP handler, `send_request`, `Body`, and the `#[fastedge::http]` macro
- [CDN_APPS.md](CDN_APPS.md) — ProxyWasm API for CDN apps (`fastedge::proxywasm::*`)
- [quickstart.md](quickstart.md) — Getting started guide
- [INDEX.md](INDEX.md) — Documentation index
