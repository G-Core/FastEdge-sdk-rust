[← Back to examples](../../../README.md)

# Diagnostic Logging (WASI)

Pass-through proxy that writes a single `fastedge::utils::set_user_diag` tag per request
summarising the outcome. The tag appears in the FastEdge platform's per-request log viewer,
distinct from stdout, and is designed to be filtered/counted/aggregated by SREs looking at
per-request outcomes.

## Configuration

- `ORIGIN_URL` environment variable — the origin that requests are proxied to.

## Outcomes

Each request writes exactly one of:

| Condition | Tag |
| --- | --- |
| `ORIGIN_URL` missing | `outcome=config_error reason=origin_missing` |
| Origin unreachable | `outcome=origin_unreachable method=<M> path=<P> err=<E>` |
| Request proxied | `outcome=proxied method=<M> path=<P> status=<S>` |

## `set_user_diag` vs `println!`

| | `println!` | `set_user_diag` |
| --- | --- | --- |
| Channel | stdout — general application logs | per-request structured tag in platform log viewer |
| Cardinality | many per request | **one per request** — multiple calls leave only the last or are concatenated (undefined) |
| Best for | verbose traces, debug details | a single filterable outcome label |
| Forbidden | — | secrets and PII (tags appear in platform logs) |

## Convention

Call `set_user_diag` **once**, on every branch, late enough in the handler to know the
outcome. The `logfmt`-ish format (`outcome=<verb> key=value key=value …`) is easy to slice in
log search tooling — keep keys short and stable so they make good filter terms.

## Related

CDN (proxy-wasm) variant: `fastedge::proxywasm::utils::set_user_diag` — same semantics,
different module path. See `docs/CDN_APPS.md`.
