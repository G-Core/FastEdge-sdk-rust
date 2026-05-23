[← Back to examples](../../../README.md)

# Cache (Basic)

Demonstrates the cache-aside pattern using `fastedge::cache` — store a generated response body with a TTL and serve it directly on subsequent requests without re-computing it.

## Configuration

| Env var | Required | Description |
|---|---|---|
| `CACHE_TTL_MS` | No | How long to cache each response in milliseconds. Default: `30000` (30 s). |

## How it works

```
GET /api/data  →  cache miss  →  generate body  →  store in cache  →  200 (x-cache: miss)
GET /api/data  →  cache hit   →  return cached body                →  200 (x-cache: hit)
```

The cache key is `page:<request-path>`. Each unique path gets its own cache entry. The response body is a simple HTML page that includes the request path — stand in for any expensive computation or template render.

## What it returns

```
HTTP/1.1 200 OK
content-type: text/html
x-cache: hit | miss

<!DOCTYPE html><html>...</html>
```

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/cache_basic.wasm
```

## APIs used

- `fastedge::cache::get(key)` — retrieve cached bytes by key; returns `Ok(Option<Vec<u8>>)`
- `fastedge::cache::set(key, bytes, ttl_ms)` — store bytes with optional TTL in milliseconds; `None` means no expiry
