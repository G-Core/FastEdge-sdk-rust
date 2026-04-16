[← Back to examples](../../README.md)

# API Key (CDN)

Validates requests using an `X-API-Key` header checked against a stored secret. Returns 401 if missing, 403 if invalid, and strips the header before forwarding to upstream.
