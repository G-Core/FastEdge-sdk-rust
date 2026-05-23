[← Back to examples](../../../README.md)

# Key Value (WASI)

Demonstrates all KV store operations via a query-parameter driven HTTP API: get, scan, zrange, zscan, and bfExists.

## Usage

All operations require `?store=<name>` and `?action=<op>` query parameters:

| Action | Additional params | Description |
|---|---|---|
| `get` | `key=<key>` | Fetch a single value by key |
| `scan` | `match=<pattern>` | List keys matching a glob pattern |
| `zrange` | `key=<key>&min=<f64>&max=<f64>` | Fetch sorted-set members by score range |
| `zscan` | `key=<key>&match=<pattern>` | List sorted-set members matching a pattern |
| `bfExists` | `key=<key>&item=<item>` | Check bloom filter membership |

`action` defaults to `get` if omitted.

## Example

```
GET /?store=my-store&action=get&key=hello
→ 200 {"store":"my-store","action":"get","key":"hello","response":"world"}

GET /?store=my-store&action=scan&match=user:*
→ 200 {"store":"my-store","action":"scan","match":"user:*","response":["user:1","user:2"]}
```

## Error responses

| Condition | Status | Body |
|---|---|---|
| Store not found / access denied | 403 | `{"error":"access denied"}` |
| Missing required params | 530 | Runtime error |
| Store open error | 500 | `{"error":"store open error: ..."}` |
| Invalid action | 400 | `{"error":"Invalid action '...'. Supported: ..."}` |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip2/release/key_value_wasi.wasm
```

## APIs used

- `fastedge::key_value::Store::open(name)` — open a named KV store
- `store.get(key)` — fetch value by key; returns `Ok(Option<Vec<u8>>)`
- `store.scan(pattern)` — list keys by glob pattern
- `store.zrange_by_score(key, min, max)` — range query on sorted set
- `store.zscan(key, pattern)` — pattern scan on sorted set
- `store.bf_exists(key, item)` — bloom filter membership test
