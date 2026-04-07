[← Back to examples](../../README.md)

# Markdown to HTML (CDN)

Converts Markdown documents returned by the origin server to HTML using the proxy-wasm ABI.

## Configuration

- Environment variable: `BASE` — (optional) URL prefix to prepend to the request path

## How it works

Uses three CDN triggers:

1. **on_request_headers** — optionally prepends `BASE` to the request path
2. **on_response_headers** — detects `text/plain` or `text/markdown` responses and sets `Content-Type` to `text/html`
3. **on_response_body** — parses the Markdown body and converts it to HTML using pulldown-cmark
