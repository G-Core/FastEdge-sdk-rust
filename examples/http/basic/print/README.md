[← Back to examples](../../../README.md)

# Print

Echoes the incoming request's method, URL, and all headers back in the response body as plain text. Useful for debugging and inspecting what a FastEdge app receives from clients and the platform.

> **Note:** This example uses the legacy `#[fastedge::http]` sync handler (`wasm32-wasip1`). For new apps, prefer `#[wstd::http_server]` (async, `wasm32-wasip2`) — see [`examples/http/wasi/`](../../wasi/).

## What it demonstrates

- Reading request method via `req.method().as_str()`
- Reading the request URI via `req.uri().to_string()`
- Iterating all request headers via `req.headers()`
- Handling non-UTF-8 header values gracefully with a `match` on `v.to_str()`
- Building a plain-text response with `Response::builder()` and `Body::from(...)`

## APIs used

| API | Purpose |
|-----|---------|
| `req.method().as_str()` | HTTP method as a string slice |
| `req.uri().to_string()` | Full request URI as a `String` |
| `req.headers()` | Iterator over `(HeaderName, HeaderValue)` pairs |
| `v.to_str()` | Decode a header value to `&str` (returns `Err` for non-UTF-8) |
| `Response::builder().status(...).body(...)` | Build the HTTP response |
| `Body::from(string)` | Create a response body from a `String` |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/print.wasm
```

## Expected behaviour

For any request, the response body is a plain-text dump of the request details:

```
Method: GET
URL: /some/path?query=value
Headers:
    host: example.com
    accept: */*
    ...
```

- Status: `200 OK`
- Content: plain text (no `content-type` header is set explicitly; the platform may add one)
- Each header appears on its own line, indented with four spaces
- Non-UTF-8 header values are replaced with `not a valid text`
