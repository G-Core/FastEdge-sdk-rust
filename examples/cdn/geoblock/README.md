[← Back to examples](../../README.md)

# Geo Block (CDN)

Blocks requests from blacklisted countries using the proxy-wasm ABI. Supports optional time-window-based blocking.

## Configuration

- Environment variable: `BLACKLIST` — comma-separated list of country codes to block (e.g. `US,CN,RU`)
- Environment variable: `BLACKLIST_TW_START` — (optional) Unix timestamp for time window start
- Environment variable: `BLACKLIST_TW_END` — (optional) Unix timestamp for time window end

## How it works

1. Reads the `BLACKLIST` env var for blocked country codes
2. Checks the request's `request.country` property against the blacklist
3. If time window vars are set, only blocks during the specified period
4. Returns 403 Forbidden for blocked requests
