# Request Lifecycle

How an HTTP request flows through the FastEdge runtime from the SDK developer's perspective. Understanding this lifecycle helps when debugging handler behavior, working with HTTP callouts, or building CDN-mode apps.

---

## Component Model Apps (HTTP Handler)

For apps using `#[fastedge::http]` or `#[wstd::http_server]`, the lifecycle is straightforward:

```
Client Request
    |
    v
Host receives HTTP request
    |
    v
Host instantiates WASM module (fresh instance per request)
    |
    v
Host calls handler: process(request) -> response
    |                    |
    |     [App can call send_request() for outbound HTTP]
    |     [App can access KV, secrets, dictionary, utils]
    |
    v
Host sends response to client
    |
    v
WASM instance is discarded
```

**Key points:**
- Each request gets a **fresh WASM instance** — no state carries between requests
- The handler is synchronous for `#[fastedge::http]`, async for `#[wstd::http_server]`
- `send_request()` makes outbound HTTP calls during handler execution
- Persistent state lives in the KV store, not in memory

---

## ProxyWasm Apps (CDN Mode) — Phase-Based Lifecycle

CDN-mode apps using the proxy-wasm interface have a multi-phase lifecycle. The host calls handler functions in a defined order, and the app returns an **action** after each phase.

### Phase Order

```
1. Module Init       _initialize() or _start()
        |
2. Context Create    proxy_on_context_create(root_id=1, parent=0)    // root context
        |
3. Context Create    proxy_on_context_create(request_id, parent=1)   // request context
        |
4. Request Headers   proxy_on_request_headers(ctx, num_headers)
        |
       [Action: Continue / Pause]
        |
5. Request Body      proxy_on_request_body(ctx, size, end_of_stream)
        |            (only if request has a body)
        |
6. Response Headers  proxy_on_response_headers(ctx, num_headers)
        |
7. Response Body     proxy_on_response_body(ctx, size, end_of_stream)
        |
8. Log               proxy_on_log(ctx)
        |
9. Instance discarded
```

### Actions

Each phase handler returns an action code that controls flow:

| Action | Meaning |
|--------|---------|
| **Continue** | Proceed to next phase |
| **Pause** | Halt processing — waiting for an HTTP callout response |

---

## HTTP Callout (Pause/Resume)

The most complex part of the lifecycle. When a ProxyWasm app needs to fetch data from an external service during request processing, it uses the HTTP callout mechanism.

### Flow

```
1. App calls proxy_http_call(upstream, headers, body, timeout)
       |
       v
2. Host returns call_id immediately
       |
       v
3. App returns Action::Pause from current phase handler
       |
       v
4. Host makes the outbound HTTP request asynchronously
       |
       v
5. When response arrives, host calls:
   proxy_on_http_call_response(ctx, call_id, headers_size, body_size, trailers_size)
       |
       v
6. App reads response via:
   - proxy_get_header_map_pairs(HttpCallResponseHeaders)    // response headers
   - proxy_get_buffer_bytes(HttpCallResponseBody, 0, size)  // response body
       |
       v
7. App resumes — host re-invokes the phase handler
       |
       v
8. App returns Action::Continue to proceed
```

### Multiple Callouts

An app can make multiple sequential HTTP callouts by returning Pause repeatedly. Each callout follows the same pattern: call -> pause -> response delivered -> resume -> call again or continue.

### Key Constraints

- **One active callout at a time** — the app pauses until the response arrives
- **Timeout enforced** — if the upstream doesn't respond in time, the callout fails
- **Non-public hosts blocked** — callouts to internal/private IP ranges are rejected
- **Response is temporary** — the callout response is only available inside `proxy_on_http_call_response`

---

## Local Response (Short-Circuit)

An app can skip upstream processing entirely by sending a local response:

```
proxy_send_local_response(status_code, headers, body)
```

This immediately sets the response and stops further phase processing. Common use cases:
- Returning cached data from KV store
- Rejecting requests based on headers or properties
- Returning error responses before reaching the backend

---

## Header and Body Access by Phase

What data is available in each phase:

| Phase | Can Read | Can Modify |
|-------|----------|------------|
| Request Headers | Request headers, properties | Request headers |
| Request Body | Request body (buffered) | Request body |
| Response Headers | Response headers, properties | Response headers |
| Response Body | Response body (buffered) | Response body |
| HTTP Call Response | Callout response headers + body | N/A (read-only) |
| Log | Request + response metadata | Nothing |

---

## Properties

Request properties are available throughout the lifecycle via `proxy_get_property()`. These provide metadata about the request and client. See `reference/PROPERTIES_REFERENCE.md` for the full list.

---

**Last Updated**: March 2026
