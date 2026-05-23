[← Back to examples](../../../README.md)

# Markdown Render

Demonstrates outbound HTTP fetch and content transformation: fetches a file from a configurable origin URL and renders it as an HTML page using [`pulldown-cmark`](https://github.com/raphlinus/pulldown-cmark). Uses the legacy synchronous `#[fastedge::http]` handler (`wasm32-wasip1`). For new apps, prefer the async [`#[wstd::http_server]`](../../wasi/) handler.

## What it teaches

- Making outbound HTTP requests from a FastEdge app with `fastedge::send_request`
- Following HTTP redirects manually (301, 302, 303, 307, 308)
- Transforming response bodies (Markdown → HTML)
- Reading environment variables at request time
- Method guard (405 for non-GET/HEAD)

## Configuration

| Variable | Required | Description |
|---|---|---|
| `BASE` | Yes | Origin base URL. The request path is appended: `BASE + path` |
| `HEAD` | No | Optional HTML injected into `<head>` (e.g. a `<link>` stylesheet tag) |

## Build

```sh
# From this directory:
cargo build --release

# WASM output:
# target/wasm32-wasip1/release/markdown_render.wasm
```

## Expected behaviour

| Condition | Response |
|---|---|
| `BASE` env var not set | 500 `Misconfigured app\n` |
| Method is not GET or HEAD | 405 `This method is not allowed\n`, `Allow: GET, HEAD` |
| Path is empty or `/` | 400 `Missing file path\n` |
| Origin returns non-200 | Forwards the upstream status, empty body |
| Origin returns a redirect | Follows the redirect (up to one level) and retries |
| Normal request | 200 HTML page with rendered Markdown, `Content-Type: text/html` |

## Example

```sh
# Set BASE to a server that serves Markdown files
BASE=https://raw.githubusercontent.com/example/repo/main

# Request /README.md -> fetches BASE/README.md -> renders as HTML
curl http://localhost:<port>/README.md
# Returns: <!DOCTYPE html><html><body><h1>...</h1>...</body></html>

# With a custom stylesheet injected into <head>
HEAD='<link rel="stylesheet" href="https://cdn.example.com/style.css">'
```

## APIs used

| API | Purpose |
|---|---|
| `fastedge::send_request(req)` | Outbound HTTP request to origin |
| `pulldown_cmark::Parser::new_ext` | Parse Markdown with tables and footnotes |
| `pulldown_cmark::html::push_html` | Render Markdown AST to HTML string |
| `std::env::var("BASE")` | Read required env var at request time |
| `url::Url::parse` | Validate redirect `Location` header |
