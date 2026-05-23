[← Back to examples](../../../README.md)

# Secret

Demonstrates accessing encrypted secrets injected by the FastEdge platform using `secret::get()` and `secret::get_effective_at()`. Shows how to handle all error variants (access denied, decrypt error) and the time-based secret rotation API.

## What it does

On every request, the handler:

1. Calls `secret::get("SECRET")` — retrieves the current value of the secret named `SECRET`.
2. If the secret is missing (returned as `None`), returns **404**.
3. Calls `secret::get_effective_at("SECRET", <unix_ts>)` — retrieves the secret value effective at the current Unix timestamp, demonstrating the rotation/versioning API.
4. If that value is also missing, returns **404**.
5. On success, returns **200** with both values in the body (Debug format).

## APIs used

| API | Purpose |
|---|---|
| `#[fastedge::http]` | Sync request-response handler macro |
| `fastedge::secret::get(key)` | Retrieve the current value of a named secret |
| `fastedge::secret::get_effective_at(key, timestamp)` | Retrieve the secret value effective at a specific Unix timestamp |
| `fastedge::secret::Error` | Error variants: `AccessDenied`, `Other(msg)`, `DecryptError` |
| `fastedge::http::{Request, Response, StatusCode}` | HTTP types |
| `fastedge::body::Body` | Response body |

## Secret error variants

| Variant | HTTP response | Meaning |
|---|---|---|
| `Ok(Some(value))` | 200 with body | Secret found |
| `Ok(None)` | 404 empty | Secret name is valid but not set |
| `Err(AccessDenied)` | 403 empty | App is not permitted to read this secret |
| `Err(Other(msg))` | 403 with `msg` body | Other denial with a human-readable message |
| `Err(DecryptError)` | 500 empty | Secret exists but could not be decrypted |

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/secret.wasm
```

## Expected behavior

| Scenario | Secret `SECRET` | Response status | Response body |
|---|---|---|---|
| Happy path | `"my-value"` | 200 | `get=Some("my-value")\nget_efective_at=Some("my-value")\n` |
| Secret not set | (absent) | 404 | (empty) |
| Access denied | — | 403 | (empty) |

> **Note:** The response body contains a typo in the field name (`get_efective_at` instead of `get_effective_at`). This is a known cosmetic issue in the source.

## Local testing

Inject the secret via a `.env` file in your fixtures directory using the `FASTEDGE_VAR_SECRET_<NAME>` prefix:

```
# fixtures/.env
FASTEDGE_VAR_SECRET_SECRET=my-test-value
```

Run with the fixture validator:

```sh
node tools/fixture-validator/index.mjs \
  FastEdge-sdk-rust/examples/http/basic/secret/ \
  --wasm FastEdge-sdk-rust/examples/http/basic/secret/target/wasm32-wasip1/release/secret.wasm
```
