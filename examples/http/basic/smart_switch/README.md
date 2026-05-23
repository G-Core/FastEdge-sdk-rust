[← Back to examples](../../../README.md)

# Smart Switch (Basic HTTP)

Demonstrates **outbound HTTP composition** on the edge: a single GET request to this app triggers two SmartThings API calls (status fetch + toggle command) before returning a result.

Uses the **legacy sync handler** (`#[fastedge::http]`, `wasm32-wasip1`). For new apps prefer the async WASI handler.

> **See also:** [`api_wrapper`](../api_wrapper/README.md) — an earlier version of the same pattern. Both examples implement identical logic; `smart_switch` is the standalone crate extracted for clarity.

## What it does

1. Rejects non-GET/HEAD requests with `405 Method Not Allowed`.
2. Requires an `Authorization` header matching the `PASSWORD` env var — returns `403` if missing or wrong.
3. Calls the SmartThings API to fetch the current switch state (`on`/`off`).
4. Sends a toggle command (`on`→`off` or `off`→`on`).
5. Returns `204 No Content` on success, or forwards the API error status.

Handles HTTP redirects from the SmartThings API automatically.

## Configuration

| Env var | Purpose |
|---|---|
| `PASSWORD` | Password checked against the `Authorization` request header |
| `DEVICE` | SmartThings device ID |
| `TOKEN` | SmartThings API bearer token |

## APIs used

| API | Purpose |
|---|---|
| `#[fastedge::http]` | Sync request-response handler macro |
| `fastedge::send_request(req)` | Outbound HTTP call to the SmartThings API |
| `fastedge::http::{Request, Response, StatusCode, Method}` | HTTP types |
| `std::env::var()` | Read `PASSWORD`, `DEVICE`, `TOKEN` at request time |
| `serde_json` | Parse SmartThings JSON responses |
| `url::Url` | Parse and follow redirect `Location` headers |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/smart_switch.wasm
```

## Expected behavior

| Request | `Authorization` header | Result |
|---|---|---|
| `POST /` (any non-GET/HEAD) | any | `405` — `Allow: GET, HEAD` |
| `GET /` | absent | `403` — `No auth header` |
| `GET /` | wrong value | `403` — empty body |
| `GET /` | correct password | `204` on success; `5xx` if SmartThings API unavailable |
