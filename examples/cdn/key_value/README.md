[← Back to examples](../../README.md)

# Key Value (CDN)

This example shows how to read and write data from a FastEdge KV store from a CDN app.
It intercepts the HTTP response, reads the request query string, and executes a KV operation against a named store.

## What it does

The app supports the following actions:

- `get` — fetch one key
- `scan` — list keys matching a pattern
- `zrange` — read sorted-set entries by score range
- `zscan` — list sorted-set entries matching a pattern
- `bfExists` — check whether a Bloom filter contains an item

The request must include at least:

- `store=<name>` — KV store name
- `action=<operation>` — optional, defaults to `get`

For each action, the app validates required parameters and returns a JSON response body.

## Supported query examples

### Get a key

```text
?store=my_store&action=get&key=user:42
```

### List keys by pattern

```text
?store=my_store&action=scan&match=user:*
```

### Read a sorted set by score range

```text
?store=my_store&action=zrange&key=leaderboard&min=0&max=100
```

### Search sorted-set members by pattern

```text
?store=my_store&action=zscan&key=leaderboard&match=user:*
```

### Check a Bloom filter item

```text
?store=my_store&action=bfExists&key=visitors&item=alice@example.com
```

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/key_value.wasm
```

This example is a CDN app, so it targets `wasm32-wasip1`.

## Response format

The app replaces the response body with JSON and sets `content-type: application/json`.
A successful response looks like this:

```json
{
  "store": "my_store",
  "action": "get",
  "key": "user:42",
  "response": "Alice"
}
```

If a required parameter is missing or the KV operation fails, the app responds with an error payload:

```json
{
  "error": "Missing required param 'key' for 'get' action"
}
```

## Notes

- The app requires query parameters to run.
- If `action` is omitted, it defaults to `get`.
- The code uses `fastedge::proxywasm::key_value::Store` and `proxy_wasm` lifecycle hooks to operate on the KV store.
