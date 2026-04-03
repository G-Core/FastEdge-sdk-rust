[← Back to examples](../../../README.md)

# Geo Redirect (WASI)

Redirects requests to country-specific origins based on the `geoip-country-code` request header.

Falls back to `BASE_ORIGIN` when no country-specific mapping is configured.

## Configuration

- Environment variable: `BASE_ORIGIN` — fallback origin URL
- Environment variable: `<COUNTRY_CODE>` — optional per-country origin URLs (e.g. `US`, `DE`, `GB`)
