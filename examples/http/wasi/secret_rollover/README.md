[← Back to examples](../../../README.md)

# Secret Rollover (WASI)

Demonstrates slot-based secret retrieval for secret rotation scenarios using `secret::get_effective_at()`.

Compares the current secret value with the value effective at a given slot, returning both as JSON.

## Usage

- `x-secret-name` request header — secret name to look up (defaults to `TOKEN_SECRET`)
- `x-slot` request header — slot value to query (defaults to current unix timestamp)

## How Slots Work

Slots use a `>=` matching rule: the slot with the highest value that is `<=` the requested `effective_at` is returned. This supports both index-based and timestamp-based rotation patterns. See the [secret rollover documentation](../../../../FastEdge-sdk-js/github-pages/src/content/docs/reference/fastedge/secret/get-secret-effective-at.md) for detailed examples.
