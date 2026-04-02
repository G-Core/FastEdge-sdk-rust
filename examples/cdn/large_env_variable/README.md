[← Back to examples](../../README.md)

# Large Environment Variable (CDN)

Demonstrates how to read **large environment variables** (> 64KB) using `fastedge::proxywasm::dictionary`.

## When to use `dictionary` vs `std::env`

| Method | Use when |
|--------|----------|
| `std::env::var("KEY")` | Variable value is under 64KB (most cases) |
| `fastedge::proxywasm::dictionary::get("KEY")` | Variable value may exceed the 64KB WASI env var size limit |

The WASI environment variable interface has a **64KB size limit** per variable. If your app needs to read larger values (e.g. large JSON configs, certificates, policy documents), use the `dictionary` API which bypasses this limit.

For all other environment variable access, prefer `std::env::var()` as it is the standard, idiomatic Rust approach.

## Required configuration

- **Environment variable**: `LARGE_CONFIG` - a large configuration payload (e.g. JSON, PEM certificate)
