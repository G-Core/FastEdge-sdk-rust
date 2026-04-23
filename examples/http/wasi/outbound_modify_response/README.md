[← Back to examples](../../../README.md)

# Outbound Modify Response (WASI)

Fetch data from an outbound HTTP origin, transform the JSON response (slice to first 5
users), and return it with a fresh `content-type: application/json` header.

Demonstrates reading the upstream body with `body.contents().await`, parsing JSON with
`serde_json`, and composing a new response from scratch.

## Related

- [outbound_fetch](../outbound_fetch/) — the simpler variant that just passes the upstream
  response through unchanged.
- Mirror of `FastEdge-sdk-js/examples/outbound-modify-response/`.
