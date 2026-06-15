[← Back to examples](../../../README.md)

# S3 Upload

FastEdge edge function that accepts a file upload, signs an S3 PUT request on the fly, uploads the file directly to an S3-compatible bucket, and returns the clean object URL to the caller.

> **Legacy handler:** Uses `#[fastedge::http]` (sync, `wasm32-wasip1`). For new apps prefer the async WASI handler — see [`examples/http/wasi/`](../../wasi/).

## What it does

1. Accepts `POST` or `PUT` only — returns 405 for other methods
2. Requires `?name=<filename>` query parameter and a non-empty body — returns 400 otherwise
3. Enforces `MAX_FILE_SIZE` if set — returns 413 if exceeded
4. Calls `prepare_s3()` to build a 1-hour presigned `PUT` URL using `rusty_s3`
5. Forwards the file body to S3 via `fastedge::send_request`
6. On success (S3 returns 200): responds with the clean object URL (no query string)
7. On S3 error: forwards the S3 status and error body back to the caller

## Configuration

| Env var | Required | Description |
|---|---|---|
| `ACCESS_KEY` | ✅ | S3 access key |
| `SECRET_KEY` | ✅ | S3 secret key |
| `REGION` | ✅ | S3 region (e.g. `s-ed1`) |
| `BASE_HOSTNAME` | ✅ | S3 base hostname (e.g. `cloud.gcore.lu`) |
| `BUCKET` | ✅ | S3 bucket name |
| `SCHEME` | optional | URL scheme — defaults to `http` |
| `MAX_FILE_SIZE` | optional | Maximum upload size in bytes — no limit if unset |

The constructed endpoint is `<SCHEME>://<REGION>.<BASE_HOSTNAME>/<BUCKET>/<name>`.

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/s3upload.wasm
```

## Usage

```
POST /upload?name=photo.jpg
Content-Type: image/jpeg

<file bytes>
```

On success (200), the response body is the clean S3 object URL (presign query parameters stripped).

## Notes

- The `OPTIONS` method returns 204 but does **not** include CORS headers — add `Access-Control-Allow-*` headers if browser preflight support is needed.
- The presigned URL expires after 1 hour, but since the upload is performed server-side this has no practical impact.
- `MAX_FILE_SIZE` is silently ignored if set to a non-numeric value.
