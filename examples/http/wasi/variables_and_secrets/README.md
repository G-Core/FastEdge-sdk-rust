[← Back to examples](../../../README.md)

# Variables and Secrets (WASI)

Demonstrates reading an environment variable (`USERNAME`) and a secret (`PASSWORD`), returning both in the response body.

Environment variables are set via the FastEdge app configuration and accessed with `std::env::var`. Secrets are stored encrypted and accessed with `fastedge::secret::get` — they are never exposed in platform logs or configuration UIs.

## Configuration

| Key | Type | Required | Description |
|---|---|---|---|
| `USERNAME` | Environment variable | No | Username to include in response. Empty string if unset. |
| `PASSWORD` | Secret | No | Password to include in response. Empty string if unset or unavailable. |

## What it returns

```
HTTP/1.1 200 OK

Username: <USERNAME value>, Password: <PASSWORD value>
```

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip2/release/variables_and_secrets.wasm
```

## APIs used

- `std::env::var("USERNAME").unwrap_or_default()` — read env var with fallback
- `fastedge::secret::get("PASSWORD")` — read secret by name; returns `Ok(Some(String))` on success
