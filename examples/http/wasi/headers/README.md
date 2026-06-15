[← Back to examples](../../../README.md)

# Headers (WASI)

Echoes all request headers back in the response and adds a custom `x-my-custom-header` whose value comes from an environment variable.

Demonstrates reading request headers via `request.headers()`, building a response with `Response::builder()`, and injecting environment-variable values into response headers.

## Configuration

| Env var | Required | Description |
|---|---|---|
| `MY_CUSTOM_ENV_VAR` | No | Value placed in the `x-my-custom-header` response header. Empty string if unset. |

## What it returns

All request headers are copied to the response, then `x-my-custom-header` is appended.

```
HTTP/1.1 200 OK
x-my-custom-header: <MY_CUSTOM_ENV_VAR value>
<...all other request headers echoed back...>

Returned all headers with a custom header added
```

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip2/release/headers.wasm
```

## APIs used

- `request.headers()` — iterate over incoming request headers
- `Response::builder().header(name, value)` — build response with individual headers
- `std::env::var("KEY").unwrap_or_default()` — read optional env var
