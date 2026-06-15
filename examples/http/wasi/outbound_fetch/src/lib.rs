/*
 * Copyright 2025 G-Core Innovations SARL
 */
/*
Minimal outbound fetch example.

Makes a GET request to an upstream HTTP origin and returns the upstream
response verbatim — status, headers, and body pass through unchanged.

For a variant that reads and transforms the upstream body, see
`outbound_modify_response/`. For a streaming-response demo, see `streaming/`.

Mirror of the FastEdge-sdk-js `outbound-fetch` example.
*/

use anyhow::anyhow;
use wstd::http::body::Body;
use wstd::http::{Client, Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let upstream_req = Request::get("http://jsonplaceholder.typicode.com/users")
        .body(Body::empty())
        .map_err(|e| anyhow!("failed to build request: {e}"))?;

    let upstream_resp = Client::new()
        .send(upstream_req)
        .await
        .map_err(|e| anyhow!("outbound request failed: {e}"))?;

    // Return the upstream response verbatim. The body is passed through
    // without calling `.contents()`, so it streams to the client as upstream
    // produces it.
    let (parts, body) = upstream_resp.into_parts();
    let mut response = Response::new(body);
    *response.status_mut() = parts.status;
    *response.headers_mut() = parts.headers;
    Ok(response)
}
