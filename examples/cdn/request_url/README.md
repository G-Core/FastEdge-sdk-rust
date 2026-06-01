[← Back to examples](../../README.md)

# Request URL (CDN)

A CDN proxy-wasm filter that rewrites outbound request properties — URL, host, path, and query string — based on incoming request headers.

## What it does

On each incoming HTTP request the filter reads the following special headers and, if present, overwrites the corresponding request property before the request is forwarded to the origin:

| Header | Property overwritten |
|--------|----------------------|
| `set-url` | `request.url` — full request URL |
| `set-host` | `request.host` — host name |
| `set-path` | `request.path` — URL path |
| `set-query` | `request.query` — query string |

It also writes a custom Nginx log field `nginx.log_field1` on every request.

## Use cases

- Rewrite the upstream URL at the edge without changing client-visible headers.
- Override the request host for multi-tenant routing.
- Strip or replace the path / query string before hitting the origin.

## Build

```bash
cargo build --target wasm32-wasip1 --release
```

## Example

Forward a request but override the path and query string:

```http
GET /original-path HTTP/1.1
Host: example.com
set-path: /new-path
set-query: page=2&limit=10
```

The filter rewrites `request.path` to `/new-path` and `request.query` to `page=2&limit=10` before the request reaches the origin.
