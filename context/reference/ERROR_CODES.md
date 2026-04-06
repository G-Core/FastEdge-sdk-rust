# Error Codes Reference

Error codes encountered when developing and debugging FastEdge SDK apps. Covers both SDK-level errors (in your Rust code) and host-level status codes (returned by the platform).

---

## Host Status Codes (3100-3120)

These codes are returned by the FastEdge host runtime when something goes wrong during WASM execution. You'll see these in platform logs and monitoring — they help diagnose why a request failed.

| Code | Name | Meaning | Common Cause |
|------|------|---------|--------------|
| `3100` | Context Error | Failed to initialize WASM module | Bad module binary, missing exports |
| `3101` | Execution Error | WASM execution failed | Unhandled panic, invalid operation |
| `3102` | Exit Error | Module called `proc_exit()` with non-zero code | Explicit error exit from app code |
| `3103` | Execution Panic | WASM trap (not timeout or OOM) | Unreachable code, division by zero, stack overflow |
| `3110` | Timeout (Interrupt) | Execution exceeded time limit (epoch interrupt) | Handler too slow, infinite loop |
| `3111` | Timeout (Elapsed) | Execution exceeded wall-clock time limit | Long-running async operation |
| `3120` | Out of Memory | WASM linear memory limit exceeded | Large allocations, unbounded data structures |

### Debugging by Code

**3100 — Context Error**
- Check that the WASM binary is valid and was built for `wasm32-wasip1`
- Ensure all required exports exist (`_initialize` or `_start`, handler function)

**3103 — Execution Panic**
- Check for `unreachable!()`, `panic!()`, or `unwrap()` on `None`/`Err` in your code
- Stack overflows from deep recursion also trigger this

**3110/3111 — Timeout**
- Profile your handler — is it doing too much work per request?
- Check for accidental infinite loops
- Consider whether outbound HTTP calls are timing out

**3120 — Out of Memory**
- Avoid unbounded `Vec` growth or large string concatenation
- Check for memory leaks (though per-request instances limit impact)
- Use streaming approaches for large payloads where possible

---

## SDK Error Enum

The `fastedge::Error` enum in `src/lib.rs` covers errors within the SDK's type conversion and HTTP handling layer:

```rust
pub enum Error {
    UnsupportedMethod(http::Method),    // HTTP method not recognized by the platform
    BindgenHttpError(HttpError),        // Error from WIT-generated HTTP types
    HttpError(http::Error),             // Error from the `http` crate
    InvalidBody,                        // Body conversion failure
    InvalidStatusCode(u16),             // Status code out of valid range
}
```

These typically surface when:
- Building an `http::Request` or `http::Response` with invalid parameters
- Calling `send_request()` with an unsupported HTTP method
- Type conversion between SDK and bindgen types fails

---

## Module-Specific Errors

### Key-Value Store (`key_value::Error`)

| Variant | Meaning |
|---------|---------|
| `NoSuchStore` | Store name not found — check deployment configuration |
| `AccessDenied` | App doesn't have permission to access this store |
| `InternalError` | Platform-side storage error |

### Secrets (`secret::Error`)

| Variant | Meaning |
|---------|---------|
| `AccessDenied` | App doesn't have permission to access this secret |
| `DecryptError` | Secret couldn't be decrypted — may be corrupted or expired |
| `Other(String)` | Platform-side error with description |

---

## ProxyWasm FFI Status Codes

The `proxy_*` FFI functions return `u32` status codes:

| Value | Meaning |
|-------|---------|
| `0` | Success |
| `1` | Not found (key doesn't exist) |
| `2` | Bad argument |
| `3` | Not allowed |
| `6` | Internal failure |

The SDK's ProxyWasm wrappers in `src/proxywasm/` translate these into Rust `Result` types — application code doesn't see raw status codes.

---

## Handler Error Behavior

### `#[fastedge::http]` (Sync)

If the handler function returns `Err(...)`, the macro catches it and returns an HTTP 500 response with the error message as the response body. This is a safety net — prefer returning explicit error responses.

### `#[wstd::http_server]` (Async)

Similar behavior — unhandled errors result in a 500 response. Use `anyhow::Result` for ergonomic error handling.

### ProxyWasm (CDN)

Panics in CDN-mode handlers trigger a WASM trap, which the host reports as status code `3103`. Use proper error handling to avoid panics.

---

**Last Updated**: March 2026
