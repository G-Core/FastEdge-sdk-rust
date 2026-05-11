/*
 * Copyright 2025 G-Core Innovations SARL
 */
/*
Example app demonstrating cache-aside pattern via the cache interface.

On each request the app:
  1. Builds a cache key from the request path.
  2. Returns the cached body immediately on a hit (x-cache: hit).
  3. On a miss, generates a response body, stores it in the cache, and
     returns it (x-cache: miss).

Environment variables:
  CACHE_TTL_MS  How long to cache the generated body in milliseconds
                (default: 30000)

Build:
  cargo build --release
*/

use std::env;

use fastedge::body::Body;
use fastedge::cache;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let ttl_ms: u64 = env::var("CACHE_TTL_MS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(30_000);

    let path = req.uri().path().to_string();
    let cache_key = format!("page:{path}");

    // Cache hit — return stored body
    if let Some(cached) = cache::get(&cache_key)? {
        println!("cache hit: {cache_key}");
        return Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", "text/html")
            .header("x-cache", "hit")
            .body(Body::from(cached))?);
    }

    // Cache miss — generate the response body
    println!("cache miss: {cache_key}");
    let body = generate_body(&path);

    // Store in cache with TTL
    cache::set(&cache_key, body.as_bytes(), Some(ttl_ms))?;

    Ok(Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/html")
        .header("x-cache", "miss")
        .body(Body::from(body))?)
}

/// Simulates an expensive computation or template render.
fn generate_body(path: &str) -> String {
    format!(
        "<!DOCTYPE html>\
        <html><head><title>FastEdge Cache Demo</title></head>\
        <body>\
          <h1>Hello from FastEdge</h1>\
          <p>Path: <code>{path}</code></p>\
          <p>This response was generated and is now cached.</p>\
        </body></html>"
    )
}
