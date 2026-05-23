[← Back to examples](../../../README.md)

# Hello World (Basic HTTP)

The simplest possible FastEdge application using the **legacy sync handler** (`#[fastedge::http]`). Returns a greeting with the full request URI in the response body.

> **When to use this example:** If you need a synchronous, single-function HTTP handler targeting `wasm32-wasip1`. For new apps, prefer the async WASI handler — see [`examples/http/wasi/hello_world`](../../wasi/hello_world/README.md).

## What it does

Handles any GET request and returns a plain-text body containing the request URI:

```
Hello, you made a basic request to /your/path?query=params
```

The handler uses `req.uri().to_string()` to include the path and query string in the response.

## APIs used

| API | Purpose |
|---|---|
| `#[fastedge::http]` | Sync request-response handler macro |
| `fastedge::http::{Request, Response, StatusCode}` | HTTP types |
| `fastedge::body::Body` | Response body |
| `Response::builder()` | Fluent response construction |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/hello_world.wasm
```

## Expected behavior

| Request | Response status | Response body |
|---|---|---|
| `GET /api/hello/world?name=FastEdge` | 200 | `Hello, you made a basic request to /api/hello/world?name=FastEdge` |
| `GET /` | 200 | `Hello, you made a basic request to /` |

Response always includes `content-type: text/plain;charset=UTF-8`.
