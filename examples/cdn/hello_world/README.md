[← Back to examples](../../README.md)

# Hello World (CDN)

Minimal CDN app demonstrating the proxy-wasm lifecycle. Logs a message at each request/response phase and adds an `x-powered-by: FastEdge` response header. Use as a starting point for new CDN apps.

## What it does

Implements all four proxy-wasm HTTP lifecycle hooks. Each hook logs a message and returns `Continue` so every request passes through unchanged:

| Hook | Phase |
|---|---|
| `on_http_request_headers` | Incoming request headers |
| `on_http_request_body` | Incoming request body |
| `on_http_response_headers` | Origin response headers — also adds `x-powered-by: FastEdge` |
| `on_http_response_body` | Origin response body |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/hello_world.wasm
```

## Expected output

Logs (in order):
```
Hello from on_http_request_headers
Hello from on_http_request_body
Hello from on_http_response_headers
Hello from on_http_response_body
```

Response header added: `x-powered-by: FastEdge`
