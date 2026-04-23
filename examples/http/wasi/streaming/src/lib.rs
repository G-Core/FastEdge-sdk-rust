/*
 * Copyright 2025 G-Core Innovations SARL
 */
/*
Streaming response example.

Generates a response body on the fly — five text chunks, one every 200ms —
using `Body::from_stream` backed by a `futures_lite::Stream`. The runtime
polls the stream as the body is sent, so chunks flow to the client as they
are produced instead of all at once at the end.

Watch it stream with `curl -N https://<app-url>/` (`-N` disables client-side
buffering).

Mirror of the FastEdge-sdk-js `streaming` example.
*/

use futures_lite::stream;
use wstd::http::body::Body;
use wstd::http::{Request, Response};
use wstd::time::{Duration, Timer};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let chunk_stream = stream::unfold(0u32, |i| async move {
        if i >= 5 {
            return None;
        }
        Timer::after(Duration::from_millis(200)).wait().await;
        Some((format!("chunk {i}\n"), i + 1))
    });

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain; charset=utf-8")
        .body(Body::from_stream(chunk_stream))?)
}
