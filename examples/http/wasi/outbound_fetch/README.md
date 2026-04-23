[← Back to examples](../../../README.md)

# Outbound Fetch (WASI)

Fetch data from an outbound HTTP origin and return the response directly — status, headers,
and body pass through unchanged.

The body is never buffered (no `.contents().await`), so upstream chunks stream to the client
as they arrive.

## Related

- [outbound_modify_response](../outbound_modify_response/) — same fetch, but reads the body
  and reshapes it into a new JSON response.
- [streaming](../streaming/) — a handler that generates its own streaming response body.
- Mirror of `FastEdge-sdk-js/examples/outbound-fetch/`.
