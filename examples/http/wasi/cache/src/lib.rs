/*
 * Copyright 2025 G-Core Innovations SARL
 */
/*
Example app demonstrating response caching via the async cache interface.

The app reads ORIGIN_HOST from the environment, forwards the incoming request
to that origin, and caches the response body keyed by the request path.
On subsequent requests for the same path the cached body is returned directly
without hitting the origin.

Environment variables:
  ORIGIN_HOST   Base URL of the upstream origin, e.g. https://api.example.com
  CACHE_TTL_MS  How long to cache responses in milliseconds (default: 60000)

Build:
  cargo build --release
*/

use std::env;

use anyhow::anyhow;
use fastedge::cache;
use wstd::http::body::Body;
use wstd::http::{Client, Request, Response};

#[wstd::http_server]
async fn main(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let origin = env::var("ORIGIN_HOST")
        .map_err(|_| anyhow!("ORIGIN_HOST environment variable is not set"))?;

    let ttl_ms: u64 = env::var("CACHE_TTL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60_000);

    // Build cache key from the request path (and query string if present)
    let path_and_query = req
        .uri()
        .path_and_query()
        .map(|pq| pq.as_str())
        .unwrap_or("/");
    let cache_key = format!("cache:{path_and_query}");

    // Return cached response if available
    if let Some(cached) = cache::get(cache_key.clone()).await? {
        println!("cache hit: {cache_key}");
        return Ok(Response::builder()
            .status(200)
            .header("content-type", "application/octet-stream")
            .header("x-cache", "hit")
            .body(Body::from(cached))?);
    }

    // Cache miss — forward request to origin
    let upstream_url = format!("{}{}", origin.trim_end_matches('/'), path_and_query);
    println!("cache miss: {cache_key} → {upstream_url}");

    let upstream_req = Request::get(&upstream_url)
        .body(Body::empty())
        .map_err(|e| anyhow!("failed to build upstream request: {e}"))?;

    let upstream_resp = Client::new()
        .send(upstream_req)
        .await
        .map_err(|e| anyhow!("upstream request failed: {e}"))?;

    let status = upstream_resp.status();
    let headers: Vec<(String, String)> = upstream_resp
        .headers()
        .iter()
        .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
        .collect();

    // Read body bytes
    let mut body = upstream_resp.into_body();
    let body_bytes = body.contents().await?.to_vec();

    // Only cache successful responses
    if status.is_success() {
        cache::set(cache_key, body_bytes.clone(), Some(ttl_ms)).await?;
    }

    // Replay original response
    let mut builder = Response::builder()
        .status(status)
        .header("x-cache", "miss");
    for (k, v) in &headers {
        builder = builder.header(k, v);
    }
    Ok(builder.body(Body::from(body_bytes))?)
}

