/*
 * Copyright 2025 G-Core Innovations SARL
 */
/*
Secret rollover example using slot-based secret retrieval.

Demonstrates how to use `secret::get_effective_at()` with slots to support
secret rotation. Slots use a greatest matching rule: the slot with the highest
value that is <= the requested `effective_at` is returned.

Example secret configuration:
{
  "secret": {
    "name": "token-secret",
    "secret_slots": [
      { "slot": 0, "value": "original_password" },
      { "slot": 1741790697, "value": "new_password" }
    ]
  }
}

Usage as indices:
  get_effective_at("token-secret", 0) -> "original_password"
  get_effective_at("token-secret", 3) -> "original_password"
  get_effective_at("token-secret", 5) -> slot 5's value (if exists)

Usage as timestamps:
  A token's `iat` claim determines which password to validate against.
  get_effective_at("token-secret", claims.iat) returns the password
  that was effective when the token was issued.
*/

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::anyhow;
use fastedge::secret;
use serde_json::json;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    // Read the slot from the x-slot header, defaulting to current timestamp
    let slot: u32 = request
        .headers()
        .get("x-slot")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.parse().ok())
        .unwrap_or_else(|| {
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards")
                .as_secs() as u32
        });

    let secret_name = request
        .headers()
        .get("x-secret-name")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("TOKEN_SECRET");

    // Get the current secret value (latest slot)
    let current = secret::get(secret_name).map_err(|e| anyhow!("secret::get failed: {e}"))?;

    // Get the secret effective at the requested slot
    let effective = secret::get_effective_at(secret_name, slot)
        .map_err(|e| anyhow!("secret::get_effective_at failed: {e}"))?;

    let result = json!({
        "secret_name": secret_name,
        "slot": slot,
        "current": current,
        "effective_at_slot": effective,
        "is_same": current == effective,
    });

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Body::from(result.to_string()))?)
}
