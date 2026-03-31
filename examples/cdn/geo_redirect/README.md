[← Back to examples](../../README.md)

# Geo Redirect (CDN)

Routes CDN requests to country-specific origins based on the geoIP country code using the proxy-wasm ABI.

## Configuration

- Environment variable: `BASE_ORIGIN` — fallback origin URL
- Environment variable: `<COUNTRY_CODE>` — optional per-country origin URLs (e.g. `US`, `DE`, `GB`)
