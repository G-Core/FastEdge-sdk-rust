[← Back to examples](../../README.md)

# Convert Image (CDN)

Converts images to AVIF format on the fly using the proxy-wasm ABI. Only transforms requests matching configured file extensions and skips specified user agents.

## Configuration

- Environment variable: `FORMATS_TO_TRANSFORM` — comma-separated list of file extensions to convert (e.g. `jpg,jpeg,png`)
- Environment variable: `IGNORED_UA_LIST` — (optional) comma-separated list of User-Agent substrings to skip
- Environment variable: `AVIF_SPEED` — (optional) AVIF encoding speed, 1-10 (default: 5)
- Environment variable: `AVIF_QUALITY` — (optional) AVIF encoding quality, 1-100 (default: 70)

## How it works

1. **on_request_headers** — checks file extension and User-Agent, sets `Image-Format` header for cache variation
2. **on_response_headers** — sets response headers for AVIF content type on 200 responses
3. **on_response_body** — decodes the original image and re-encodes it as AVIF
