[← Back to examples](../../../README.md)

# Watermark

Reads an image from S3, applies a watermark with configurable opacity using alpha blending, and returns the composited image.

## Configuration

- Environment variable: `ACCESS_KEY` — S3 access key
- Environment variable: `SECRET_KEY` — S3 secret key
- Environment variable: `REGION` — S3 region
- Environment variable: `BASE_HOSTNAME` — S3 endpoint hostname
- Environment variable: `BUCKET` — S3 bucket name
- Environment variable: `SCHEME` — URL scheme (defaults to `http`)
- Environment variable: `OPACITY` — watermark opacity (0.0 to 1.0)
