[← Back to examples](../../README.md)

# Cache (CDN)

Implements cache operations via query parameters — get, set, delete, exists, incr, expire, purge, and purgePrefix — using the proxy-wasm ABI.

Unlike the [key_value](../key_value/) example there is no `store` parameter: the cache has no named stores or handles, so every operation is scoped to the calling application and addressed by key alone.

## Usage

| Query | Operation |
|---|---|
| `?action=get&key=<key>` | Read a value. `response` is `null` when the key is absent. |
| `?action=set&key=<key>&value=<value>&ttl=<ms>` | Store a value. Omit `ttl` for no expiry. |
| `?action=delete&key=<key>` | Delete a key. No-op when the key is absent. |
| `?action=exists&key=<key>` | Key membership check. |
| `?action=incr&key=<key>&delta=<i64>` | Atomic increment. `delta` may be negative; a missing key starts at `0`. |
| `?action=expire&key=<key>&ttl=<ms>` | Set or update a key's expiry. `response` is `false` when the key is absent. |
| `?action=purge` | Delete every key owned by this app; returns the number deleted. |
| `?action=purgePrefix&prefix=<prefix>` | Delete this app's keys starting with `prefix`; returns the number deleted. |

Defaults to `action=get` when `action` is omitted. All responses are JSON; errors return status 500 with `{"error": "..."}`.

```sh
curl 'https://<your-app>/?action=set&key=hits&value=0&ttl=60000'
curl 'https://<your-app>/?action=incr&key=hits&delta=1'
curl 'https://<your-app>/?action=get&key=hits'
```

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/cache.wasm
```

## APIs used

- `fastedge::proxywasm::cache::get(key)` — retrieve cached bytes by key; returns `Ok(Option<Vec<u8>>)`
- `fastedge::proxywasm::cache::set(key, bytes, ttl_ms)` — store bytes with optional TTL in milliseconds; `None` means no expiry
- `fastedge::proxywasm::cache::delete(key)` / `exists(key)` — remove a key, or test for its presence
- `fastedge::proxywasm::cache::incr(key, delta)` — atomic counter update, returns the new value
- `fastedge::proxywasm::cache::expire(key, ttl_ms)` — re-arm a key's expiry, returns whether the key existed
- `fastedge::proxywasm::cache::purge()` / `purge_prefix(prefix)` — bulk delete, returns the number of keys removed
