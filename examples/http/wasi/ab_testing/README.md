[← Back to examples](../../../README.md)

# A/B Testing (WASI)

Cookie-based A/B testing. Reads or creates an `x-fastedge-abid` cookie, uses its value to
deterministically assign the visitor to weighted variants of two tests (`logo` and `font`),
then proxies the request to an outbound origin with the variant assignments attached as
`ab-test-<name>` headers. The response sets the cookie so returning visitors receive the
same variants on subsequent visits.

Demonstrates cookie parsing, request-header mutation, deterministic assignment from a
persistent identifier, and an outbound fetch with header overrides.

## Configuration

- `OUTBOUND_URL` environment variable — the downstream origin that consumes the
  `ab-test-*` headers (for example, a templated backend).

## How assignment works

The `xid` cookie value is a decimal like `0.4532`. Multiplied by 100, it selects a slot in
each test's cumulative weight range. Weights don't need to sum to 100 — they're normalised
at assignment time. See `TESTS` in `src/lib.rs` to tweak the split.

## Response

The origin response is returned verbatim, with one added header:

```
set-cookie: x-fastedge-abid=<xid>; Max-Age=31536000; Path=/; Secure; HttpOnly; SameSite=Lax
```

## Related

Mirror of `FastEdge-sdk-js/examples/ab-testing/`.
