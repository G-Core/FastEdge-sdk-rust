/*
 * Copyright 2025 G-Core Innovations SARL
 */
/*
Bloom-filter IP denylist example.

Checks the client IP (from the `x-real-ip` request header, falling back to
`x-forwarded-for`) against a bloom filter stored in FastEdge KV. Returns 403
on a hit, 200 otherwise.

Required configuration:
  - Environment variable: DENYLIST_STORE (KV store name holding the bloom filter)

The bloom-filter key is hardcoded to `blocked-ips`. The handler is read-only;
populate the filter out of band.

Mirror of the FastEdge-sdk-js `bloom-filter-denylist` example.
*/

use std::env;

use anyhow::anyhow;
use fastedge::key_value::{Error as StoreError, Store};
use serde_json::json;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

const BLOOM_KEY: &str = "blocked-ips";

#[wstd::http_server]
async fn main(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let store_name = match env::var("DENYLIST_STORE") {
        Ok(s) if !s.trim().is_empty() => s,
        _ => {
            return json_response(
                500,
                json!({ "error": "DENYLIST_STORE environment variable is not configured" }),
            );
        }
    };

    let headers = req.headers();
    let client_ip = headers
        .get("x-real-ip")
        .or_else(|| headers.get("x-forwarded-for"))
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.split(',').next())
        .map(str::trim)
        .filter(|s| !s.is_empty());

    let Some(ip) = client_ip else {
        return json_response(500, json!({ "error": "client IP not available" }));
    };

    let store = match Store::open(&store_name) {
        Ok(s) => s,
        Err(StoreError::AccessDenied) => {
            return json_response(
                403,
                json!({ "error": "access denied opening denylist store" }),
            );
        }
        Err(e) => {
            return json_response(500, json!({ "error": format!("store open error: {e}") }));
        }
    };

    let blocked = store
        .bf_exists(BLOOM_KEY, ip)
        .map_err(|e| anyhow!("bf_exists error: {e}"))?;

    if blocked {
        // Bloom filter says "maybe in set" — a small fraction of hits will be false
        // positives. Acceptable for a denylist (you over-block some legitimate users);
        // not acceptable for allowlists — use `store.get()` against a regular key instead.
        return json_response(403, json!({ "allowed": false, "ip": ip }));
    }

    json_response(200, json!({ "allowed": true, "ip": ip }))
}

fn json_response(status: u16, value: serde_json::Value) -> anyhow::Result<Response<Body>> {
    Ok(Response::builder()
        .status(status)
        .header("content-type", "application/json")
        .body(Body::from(value.to_string()))?)
}
