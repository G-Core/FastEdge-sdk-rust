[← Back to examples](../../../README.md)

# Outbound Fetch (Basic HTTP)

Demonstrates outbound HTTP from a FastEdge application using the **legacy sync handler** (`#[fastedge::http]`). Fetches user data from the [JSONPlaceholder](https://jsonplaceholder.typicode.com) public API, selects the first 5 users, and returns them in a paginated JSON envelope.

> **When to use this example:** If you need a synchronous, single-function HTTP handler with outbound requests targeting `wasm32-wasip1`. For new apps, prefer the async WASI handler — see [`examples/http/wasi/`](../../wasi/).

## What it does

On any incoming request:

1. Makes a GET request to `http://jsonplaceholder.typicode.com/users` using `fastedge::send_request`.
2. Parses the JSON response body.
3. Takes the first 5 users from the array.
4. Returns a JSON envelope with pagination metadata.

Example response body:

```json
{
  "users": [ { "id": 1, "name": "Leanne Graham", ... }, ... ],
  "total": 5,
  "skip": 0,
  "limit": 30
}
```

## APIs used

| API | Purpose |
|---|---|
| `#[fastedge::http]` | Sync request-response handler macro |
| `fastedge::send_request` | Outbound HTTP request to upstream API |
| `fastedge::http::{Request, Response, StatusCode}` | HTTP types |
| `fastedge::body::Body` | Request and response bodies |
| `serde_json` | JSON parsing and serialisation |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/outbound_fetch.wasm
```

## Expected behavior

| Request | Response status | Response content-type | Response body |
|---|---|---|---|
| `GET /` | 200 | `application/json` | JSON object with `users` (array of ≤5), `total`, `skip`, `limit` |
