[← Back to examples](../../README.md)

# JWT Validation (CDN)

Validates JWT tokens on incoming requests using the proxy-wasm ABI. Checks the `Authorization` header for a Bearer token, verifies the signature using a secret, and validates expiration.

## Configuration

- Secret: `secret` — the HMAC secret used to verify JWT signatures

## How it works

1. Reads the `secret` from FastEdge secrets
2. Extracts the Bearer token from the `Authorization` header
3. Decodes and validates the JWT signature
4. Checks the `exp` claim against the current time
5. Returns 401/403 for missing/invalid/expired tokens, or continues the request if valid
