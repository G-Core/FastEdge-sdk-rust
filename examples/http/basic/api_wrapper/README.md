[← Back to examples](../../../README.md)

# API Wrapper

Demonstrates how to wrap multiple outbound API calls in a single FastEdge edge function, using the legacy synchronous `#[fastedge::http]` handler (`wasm32-wasip1`). The function implements password-based authentication, fetches the current state of a SmartThings smart-switch device, and sends a toggle command to flip it.

> **Handler note:** This example uses `#[fastedge::http]` (sync, `wasm32-wasip1`), which is the legacy handler for basic HTTP apps. For new projects, prefer `#[wstd::http_server]` (async, `wasm32-wasip2`).

## What this example teaches

- How to make multiple sequential outbound HTTP calls with `fastedge::send_request`
- How to implement password-based authentication via a request header
- How to guard against misconfigured apps (missing env vars → 500)
- How to parse JSON responses with `serde_json`
- HTTP redirect handling in the outbound call helper

## APIs used

| API | Description |
|---|---|
| `#[fastedge::http]` | Sync handler macro — entry point |
| `fastedge::send_request(req)` | Outbound HTTP call to the SmartThings API |
| `env::var("NAME")` | Read env vars (PASSWORD, DEVICE, TOKEN) |
| `serde_json::from_str` | Parse JSON from the SmartThings response |
| `Response::builder()` | Build HTTP responses |
| `Request::builder()` | Build outbound HTTP requests |

## Configuration

| Env var | Description |
|---|---|
| `PASSWORD` | Expected password value — compared against the `Authorization` request header |
| `DEVICE` | SmartThings device ID |
| `TOKEN` | SmartThings API bearer token |

## Request format

Send a GET or HEAD request with the password in the `Authorization` header:

```
GET / HTTP/1.1
Authorization: <your-password>
```

Only `GET` and `HEAD` are accepted; any other method returns `405 Method Not Allowed` with an `Allow: GET, HEAD` header.

## Response summary

| Condition | Status | Body |
|---|---|---|
| Missing `Authorization` header | 403 | `No auth header` |
| Wrong password | 403 | _(empty)_ |
| Missing env var (PASSWORD, DEVICE, or TOKEN) | 500 | `Misconfigured app` |
| SmartThings API error | Reflects upstream status | _(empty)_ |
| Device toggled successfully | 204 | _(empty)_ |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/api_wrapper.wasm
```

## App flow

1. Validate HTTP method (GET / HEAD only)
2. Read `PASSWORD` env var — 500 if missing
3. Check `Authorization` header matches `PASSWORD` — 403 if missing or wrong
4. Read `DEVICE` and `TOKEN` env vars — 500 if missing
5. `GET /v1/devices/<DEVICE>/status` → parse `components.main.switch.switch.value` (`"on"` or `"off"`)
6. `POST /v1/devices/<DEVICE>/commands` with the opposite command
7. Return 204 on `ACCEPTED`, or the upstream status code on error
