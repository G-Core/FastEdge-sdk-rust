[← Back to examples](../../../README.md)

# Cache (WASI)

Demonstrates the cache-aside pattern with origin forwarding using `fastedge::cache`. Forwards incoming requests to `ORIGIN_HOST`, caches successful response bodies keyed by path and query string, and serves cached bytes directly on subsequent matching requests.

## Configuration

| Env var | Required | Description |
|---|---|---|
| `ORIGIN_HOST` | Yes | Base URL of the upstream origin (e.g. `https://api.example.com`). Returns 500 if unset. |
| `CACHE_TTL_MS` | No | How long to cache responses in milliseconds. Default: `60000` (60 s). |

## How it works

```
GET /data?id=1  →  cache miss  →  forward to ORIGIN_HOST/data?id=1  →  cache 2xx body  →  200 (x-cache: miss)
GET /data?id=1  →  cache hit   →  return cached body                                   →  200 (x-cache: hit)
```

Cache key is `cache:<path>?<query>`. Only 2xx responses from the origin are cached — error responses pass through without being stored. The origin's response headers are replayed on cache miss; cache-hit responses use `content-type: application/octet-stream` since the original content-type is not stored alongside the body bytes.

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip2/release/cache_wasi.wasm
```

## APIs used

- `fastedge::cache::get(key)` — retrieve cached bytes by key; returns `Ok(Option<Vec<u8>>)`
- `fastedge::cache::set(key, bytes, ttl_ms)` — store bytes with optional TTL in milliseconds; `None` means no expiry
- `wstd::http::Client::new().send(req).await` — async outbound HTTP request to origin
