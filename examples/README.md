# FastEdge Rust Examples

Rust examples for building applications on the [FastEdge](https://gcore.com/fastedge)
network using the [`fastedge`](https://crates.io/crates/fastedge) crate.

Examples are organized into three categories:

- **[http/basic](./http/basic/)** — Synchronous HTTP apps using the `#[fastedge::http]` macro
- **[http/wasi](./http/wasi/)** — Async WASI-HTTP apps using the `#[wstd::http_server]` macro
- **[cdn](./cdn/)** — CDN-integrated apps using the proxy-wasm ABI

## Getting Started Examples

### http/basic (sync)

| Example | Description |
| --- | --- |
| [hello_world](./http/basic/hello_world/) | Simplest request handler — returns a greeting with the request URL |
| [headers](./http/basic/headers/) | Echo request headers and add a custom header from a environment variable |
| [variables_and_secrets](./http/basic/variables_and_secrets/) | Read a environment variable and a secret |
| [print](./http/basic/print/) | Print request method, URL, and headers to the response body |

### http/wasi (async)

| Example | Description |
| --- | --- |
| [hello_world](./http/wasi/hello_world/) | Simplest async request handler — returns a greeting with the request URL |
| [headers](./http/wasi/headers/) | Echo request headers and add a custom header from a environment variable |
| [variables_and_secrets](./http/wasi/variables_and_secrets/) | Read a environment variable and a secret |
| [simple_fetch](./http/wasi/simple_fetch/) | Fetch from a configurable URL and return the response |

## Full Examples

### http/basic (sync)

| Example | Description |
| --- | --- |
| [geo_redirect](./http/basic/geo_redirect/) | Redirect requests to country-specific origins based on geoIP |
| [key_value](./http/basic/key_value/) | KV store operations — get, scan, zrange, zscan |
| [outbound_fetch](./http/basic/outbound_fetch/) | Fetch from a remote JSON API and transform the response |
| [secret](./http/basic/secret/) | Secret retrieval with timestamp-based rotation support |
| [backend](./http/basic/backend/) | API wrapper with authentication and device toggling |
| [api_wrapper](./http/basic/api_wrapper/) | Wrap multiple API calls to get and toggle device state |
| [watermark](./http/basic/watermark/) | Read an image from S3 and apply a watermark overlay |
| [markdown_render](./http/basic/markdown_render/) | Fetch Markdown files and render them to HTML |

### http/wasi (async)

| Example | Description |
| --- | --- |
| [geo_redirect](./http/wasi/geo_redirect/) | Redirect requests to country-specific origins based on geoIP |
| [key_value](./http/wasi/key_value/) | KV store operations — get, scan, zrange, zscan, bfExists |
| [outbound_fetch](./http/wasi/outbound_fetch/) | Make outbound HTTP requests to a JSON API and transform the response |
| [secret_rollover](./http/wasi/secret_rollover/) | Slot-based secret retrieval for secret rotation scenarios |

### cdn (proxy-wasm)

| Example | Description |
| --- | --- |
| [headers](./cdn/headers/) | Validate and manipulate HTTP request/response headers |
| [body](./cdn/body/) | Redact HTTP request/response bodies matching a pattern |
| [properties](./cdn/properties/) | Extract request properties — URL, path, host, geo data |
| [log_time](./cdn/log_time/) | Log request and response timestamps |
| [custom](./cdn/custom/) | Return HTTP status codes based on request path with optional delay |
| [http_call](./cdn/http_call/) | Make asynchronous HTTP calls to external services |
| [key_value](./cdn/key_value/) | KV store operations via query parameters |
| [geo_redirect](./cdn/geo_redirect/) | Route requests to country-specific origins based on geoIP |
| [variables_and_secrets](./cdn/variables_and_secrets/) | Read environment variables and secrets for request forwarding |

## Usage

Each example is a standalone project. To build one:

```sh
cd <example-name>
cargo build --target wasm32-wasip1 --release
```

Each example depends on the [`fastedge`](https://crates.io/crates/fastedge) crate from crates.io.
