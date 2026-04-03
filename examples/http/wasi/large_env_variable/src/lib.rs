/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example WASI-HTTP app demonstrating access to large environment variables.

Uses `fastedge::dictionary` to read environment variables that may exceed
the 64KB WASI environment variable size limit.

For normal-sized environment variables (< 64KB), prefer `std::env::var()`
instead. The dictionary API is only required when your variable value
may be larger than 64KB.

Required configuration:
  - Environment variable: LARGE_CONFIG (a large configuration payload, e.g. JSON)
*/

use fastedge::dictionary;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    // Use dictionary::get for environment variables that may exceed 64KB.
    // For normal-sized env vars, use std::env::var() instead.
    let config = dictionary::get("LARGE_CONFIG").unwrap_or_default();

    let size = config.len();

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!(
            "LARGE_CONFIG loaded: {} bytes",
            size
        )))?)
}
