[← Back to examples](../../../README.md)

# Streaming Response (WASI)

Generates a response body on the fly — five text chunks, one every 200 ms — using
`Body::from_stream` backed by a `futures_lite::Stream`. Each chunk flows to the client as it
is produced, not all at once at the end.

Demonstrates `wstd::http::body::Body::from_stream`, `futures_lite::stream::unfold` for async
stream generation, and `wstd::time::Timer` for per-chunk delays.

## Testing the streaming behaviour

```sh
curl -N https://<your-app>.fastedge.cdn.gc.onl/
```

`-N` disables curl's client-side buffering; without it you won't see chunks appear one at a
time. You should see `chunk 0`…`chunk 4` print at ~200ms intervals.

## Other streaming patterns

- **Pass-through streaming** — return an upstream response's body directly. See
  [outbound_fetch/](../outbound_fetch/) for the no-buffer variant.
- **Transform streaming** — use `http_body_util::BodyExt::map_frame` on the incoming body,
  then `Body::from_http_body` to wrap it back. Useful for chunk-level rewrites.
- **Stream from bytes** — `Body::from_stream(futures_lite::stream::iter(chunks))` where
  `chunks` is any iterable of `Into<Bytes>`.

## Related

Mirror of `FastEdge-sdk-js/examples/streaming/`.
