[← Back to examples](../../../README.md)

# Geo Redirect (WASI)

Redirects requests to country-specific origins based on the `geoip-country-code` request header. Falls back to `BASE_ORIGIN` when no country-specific mapping is configured.

Demonstrates reading request headers, reading environment variables, and returning redirect responses.

## Configuration

| Env var | Required | Description |
|---|---|---|
| `BASE_ORIGIN` | Yes | Fallback redirect URL (e.g. `https://example.com`). Returns 500 if unset. |
| `<COUNTRY_CODE>` | No | Per-country redirect URL, keyed by 2-letter country code (e.g. `DE`, `US`, `GB`). Falls back to `BASE_ORIGIN` if not set. |

## How it works

```
geoip-country-code: DE  →  env var DE is set  →  302 to DE value
geoip-country-code: FR  →  env var FR not set  →  302 to BASE_ORIGIN
(no header)             →  302 to BASE_ORIGIN
BASE_ORIGIN not set     →  500
```

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip2/release/geo_redirect_wasi.wasm
```

## APIs used

- `request.headers().get("geoip-country-code")` — read geo header injected by the FastEdge edge
- `std::env::var(country_code)` — dynamic env var lookup by country code
- `Response::builder().status(302).header("location", url)` — redirect response
