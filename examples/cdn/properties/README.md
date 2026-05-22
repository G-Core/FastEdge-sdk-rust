[← Back to examples](../../README.md)

# Properties (CDN)

Extracts and manipulates request properties — URL, path, host, and geo data — using the proxy-wasm ABI. Forwards each extracted value as a response header so downstream clients or logging pipelines can inspect them.

## What it does

On every request, reads the following properties and adds each as a response header:

| Property key | Response header |
|---|---|
| `request.url` | `request-uri` |
| `request.host` | `request-host` |
| `request.path` | `request-path` |
| `request.scheme` | `request-scheme` |
| `request.extension` | `request-extension` |
| `request.query` | `request-query` |
| `request.x_real_ip` | `request-x-real-ip` |
| `request.country` | `request-country` |
| `request.city` | `request-city` |
| `request.asn` | `request-asn` |
| `request.geo.long` | `request-long` |
| `request.geo.lat` | `request-lat` |
| `request.country.name` | `request-country-names` |
| `request.region` | `request-country-region` |
| `request.continent` | `request-continent` |

If any property is missing the handler sends a 55x error response and stops — each property has a unique status code to identify exactly which lookup failed.

## Query-param overrides

After extracting all properties, the handler checks for override query parameters and rewrites the corresponding upstream request property if present:

| Query param | Overwrites |
|---|---|
| `?url=<value>` | `request.url` |
| `?host=<value>` | `request.host` |
| `?path=<value>` | `request.path` |

## nginx log field

Sets `nginx.log_field1` to `"from_wasm nginx.log_field1"` on every request, demonstrating how to write custom values into the CDN access log.

## Build

```sh
cargo build --release
# Output: target/wasm32-wasip1/release/properties.wasm
```

## Notes

`fixtures/force-server-properties.json` is a visual-debugger configuration file (no `.test.json` extension), not a test fixture. It is used to seed server-side properties when running the app in the FastEdge visual debugger.
