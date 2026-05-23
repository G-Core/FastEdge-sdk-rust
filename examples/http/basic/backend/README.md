[← Back to examples](../../../README.md)

# Backend (URL Proxy)

A FastEdge application that accepts a `?url=` query parameter, makes an outbound GET request to that URL via `fastedge::send_request`, and returns a summary of the upstream response (`len` and `content-type`) in the response body.

> **When to use this example:** When you want to see how to make outbound HTTP requests from a FastEdge edge function using the legacy sync handler (`#[fastedge::http]`). For new apps, prefer the async WASI handler — see [`examples/http/wasi/hello_world`](../../wasi/hello_world/README.md).

## What it does

1. Parses the `?url=` query parameter from the request URI (percent-decodes it via `urlencoding::decode`).
2. Builds an outbound `GET` request to that URL using `fastedge::send_request`.
3. Returns HTTP 200 with a plain-text body:
   ```
   len = <body-length>, content-type = <upstream-content-type>
   ```
4. Returns HTTP 500 with an error message if `?url=` is absent or the query string is missing.

## APIs used

| API | Purpose |
|---|---|
| `#[fastedge::http]` | Sync request-response handler macro |
| `fastedge::send_request(request)` | Blocking outbound HTTP request |
| `fastedge::http::{Request, Response, StatusCode, Method}` | HTTP types |
| `fastedge::body::Body` | Request and response bodies |
| `querystring::querify` | Parse query string into key-value pairs |
| `urlencoding::decode` | Percent-decode the `?url=` value |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/backend.wasm
```

## Expected behavior

| Request | Response status | Response body |
|---|---|---|
| `GET /?url=https%3A%2F%2Fhttpbin.org%2Fget` | 200 | `len = <N>, content-type = Some("<mime>")` |
| `GET /?q=hello` (no `url` key) | 500 | `missing url parameter` |
| `GET /` (no query string) | 500 | `missing uri query parameter` |

The `len` value is the byte length of the upstream response body. The `content-type` value is the `Content-Type` header returned by the upstream server, formatted as a Rust `Option<HeaderValue>` debug string (e.g. `Some("application/json")`).
