[← Back to examples](../../../README.md)

# S3 Upload

Uploads files to an S3-compatible bucket via POST/PUT requests, generating signed URLs for the upload.

## Configuration

- Environment variable: `ACCESS_KEY` — S3 access key
- Environment variable: `SECRET_KEY` — S3 secret key
- Environment variable: `REGION` — S3 region (e.g. `s-ed1`)
- Environment variable: `BASE_HOSTNAME` — S3 base hostname (e.g. `cloud.gcore.lu`)
- Environment variable: `BUCKET` — S3 bucket name
- Environment variable: `SCHEME` — (optional) URL scheme, defaults to `http`
- Environment variable: `MAX_FILE_SIZE` — (optional) maximum upload size in bytes

## Usage

Send the file content as the request body using POST or PUT. Specify the filename via the `name` query parameter:

```
PUT /upload?name=photo.jpg
Content-Type: image/jpeg

<file bytes>
```
