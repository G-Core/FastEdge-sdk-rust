[← Back to examples](../../../README.md)

# Bloom Filter — IP Denylist (WASI)

Rejects requests from IPs present in a KV Store bloom filter. Reads the client IP from the
`x-real-ip` request header (falling back to `x-forwarded-for`), checks it against a
pre-populated bloom filter, and returns **403** on a hit or **200** otherwise.

Demonstrates `fastedge::key_value::Store` + `bf_exists()` and the conventional way to obtain
the client IP from a Component Model HTTP handler.

## Configuration

- Environment variable `DENYLIST_STORE` — name of the KV store that holds the bloom filter.
- Bloom-filter key — hardcoded to `blocked-ips`. Change `BLOOM_KEY` in `src/lib.rs` if your
  key is different.

## Behaviour

| `bf_exists("blocked-ips", ip)` | Response |
| --- | --- |
| `true` | `403` `{ "allowed": false, "ip": "..." }` |
| `false` | `200` `{ "allowed": true, "ip": "..." }` |

## Tradeoff: false positives

Bloom filters answer "**definitely not** in set" vs "**maybe** in set". When `bf_exists`
returns `true`, the IP *probably* was added — but a small fraction of hits will be false
positives, meaning some legitimate IPs will be over-blocked. Acceptable for a denylist; for
allowlists or anything requiring exact membership, use `store.get()` against a regular key
instead.

## Populating the filter

The edge handler is read-only. Populate `blocked-ips` out of band (for example, via the
FastEdge API).

## Related

Mirror of `FastEdge-sdk-js/examples/bloom-filter-denylist/`.
