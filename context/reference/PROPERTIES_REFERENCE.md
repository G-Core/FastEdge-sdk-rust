# Request Properties Reference

Properties are read-only metadata about the current request and client, available via `proxy_get_property()` in ProxyWasm apps. They provide context that isn't in the HTTP headers themselves.

---

## Available Properties

### Client Information

| Property Path | Type | Description |
|--------------|------|-------------|
| `request.x_real_ip` | string | Client's real IP address |
| `request.country` | string | Client's country (from IP geolocation) |
| `request.city` | string | Client's city (from IP geolocation) |
| `request.asn` | string | Client's Autonomous System Number |
| `request.geo.lat` | string | Client's latitude |
| `request.geo.long` | string | Client's longitude |

### Request Metadata

| Property Path | Type | Description |
|--------------|------|-------------|
| `request.host` | string | Request hostname (from CDN headers) |
| `request.uri` | string | Full request URI (scheme + host + path) |
| `request.scheme` | string | Request scheme (http/https) |
| `request.path` | string | Request path |

### Tracing

| Property Path | Type | Description |
|--------------|------|-------------|
| `request.traceparent` | string | W3C Trace Context traceparent header for distributed tracing |

---

## Usage in ProxyWasm Apps

```rust
// In an HttpContext implementation
fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
    // Get client's country
    if let Some(country) = self.get_property(vec!["request", "country"]) {
        let country_str = String::from_utf8(country).unwrap_or_default();
        // Use for geo-routing, access control, etc.
    }

    // Get client IP
    if let Some(ip) = self.get_property(vec!["request", "x_real_ip"]) {
        // Use for rate limiting, logging, etc.
    }

    Action::Continue
}
```

---

## Usage in Component Model Apps

In `#[fastedge::http]` or `#[wstd::http_server]` apps, some of these values are available through request headers or the `dictionary` interface rather than properties directly. The property system is primarily a ProxyWasm concept.

For Component Model apps:
- **Client IP** — available via request headers (e.g., `X-Real-IP`)
- **Geo data** — available via `dictionary::get()` with appropriate keys
- **Host/URI** — available from the `http::Request` object directly

---

## Property Caching

Properties are cached per-request by the host after first access. Repeated lookups for the same property within a single request are fast. Geo-related properties (country, city, lat/long) may involve an IP lookup on first access.

---

## Notes

- All property values are returned as raw bytes (`Vec<u8>`). The SDK developer is responsible for parsing (usually UTF-8 strings).
- Properties not available for the current request return `None`.
- The property set may expand over time as the platform adds capabilities.

---

**Last Updated**: March 2026
