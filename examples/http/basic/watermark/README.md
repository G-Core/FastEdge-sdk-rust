[← Back to examples](../../../README.md)

# Watermark

Demonstrates outbound S3 fetch and image compositing on the edge. The app retrieves an image from an S3-compatible bucket using request signing (`rusty_s3`), overlays an embedded watermark PNG (`sample.png`) using per-pixel alpha blending, and returns the composited image in the original format.

The watermark file (`src/sample.png`) is embedded at compile time via `include_bytes!` — no runtime filesystem access is needed.

Uses the legacy `#[fastedge::http]` sync handler (`wasm32-wasip1`). For new HTTP apps, prefer `#[wstd::http_server]` (async, `wasm32-wasip2`).

## What it does

1. Rejects non-GET/HEAD requests with `405 Method Not Allowed`.
2. Extracts the image filename from the URL path (e.g. `GET /photo.jpg`).
3. Constructs a time-limited AWS Signature V4 signed URL for the file in the configured S3 bucket.
4. Fetches the image from S3 via `fastedge::send_request`.
5. If the S3 response body is a valid image format, overlays the embedded watermark at the top-left corner.
6. Returns the composited image with the original MIME type.
7. If the S3 body is not a valid image, it is forwarded to the caller unchanged.

## Configuration

All environment variables are set on the deployed FastEdge app.

| Variable | Required | Description |
|---|---|---|
| `ACCESS_KEY` | ✅ | S3 access key ID |
| `SECRET_KEY` | ✅ | S3 secret access key |
| `REGION` | ✅ | S3 region (e.g. `us-east-1`) |
| `BASE_HOSTNAME` | ✅ | S3 endpoint hostname (e.g. `cloud.gcore.lu`) |
| `BUCKET` | ✅ | S3 bucket name |
| `SCHEME` | optional | URL scheme for S3 endpoint (default: `http`) |
| `OPACITY` | optional | Watermark opacity as a float in `0.0`–`1.0` (default: `1.0`) |

## Build

```sh
cargo build --release
# WASM output: target/wasm32-wasip1/release/watermark.wasm
```

## Expected behavior

| Request | Response |
|---|---|
| `POST /image.png` (wrong method) | `405 Method Not Allowed`, body: `This method is not allowed\n` |
| `GET /` (no filename) | `400 Bad Request`, body: `Malformed request - filename expected\n` |
| `GET /image.png` (env vars missing) | `500 Internal Server Error`, body: `App misconfigured\n` |
| `GET /image.png` (configured, image in bucket) | `200 OK`, watermarked image in original format |

## APIs used

| API | Purpose |
|---|---|
| `fastedge::send_request(req)` | Outbound HTTP request to S3 |
| `rusty_s3::{Bucket, Credentials, S3Action}` | AWS Signature V4 URL signing |
| `image::{load_from_memory, DynamicImage}` | Image decode, compositing, encode |
| `include_bytes!("sample.png")` | Embed watermark at compile time |
| `std::env::var` | Read S3 credentials and `OPACITY` from app env |

## Live testing

This example requires a real S3-compatible bucket (Gcore Object Storage or AWS S3) with valid credentials. The deterministic error paths (wrong method, empty path, missing env vars) can be validated locally with the fixture validator. The watermark compositing path requires a live deployment with credentials configured as app env vars.
