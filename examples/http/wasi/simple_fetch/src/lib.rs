/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example app demonstrating the WASI-HTTP interface via the wstd crate.

The app receives an incoming HTTP request and makes an outbound HTTP request
to the URL specified in the `x-fetch-url` header (defaults to https://httpbin.org/get).

Build with cargo-component:
  cargo component build --release
*/

use anyhow::anyhow;
use wstd::http::body::Body;
use wstd::http::{Client, Request, Response};

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let target_url = request
        .headers()
        .get("x-fetch-url")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("https://httpbin.org/get")
        .to_string();

    println!("Fetching: {target_url}");

    let upstream_req = Request::get(&target_url)
        .header("accept", "application/json")
        .body(Body::empty())
        .map_err(|e| anyhow!("failed to build request: {e}"))?;

    let client = Client::new();
    let response = client
        .send(upstream_req)
        .await
        .map_err(|e| anyhow!("request failed: {e}"))?;

    println!("Response status: {}", response.status());

    Ok(response)
}
