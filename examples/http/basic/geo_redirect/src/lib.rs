/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example HTTP app demonstrating geo-based redirects.

Reads the country code from the geoip-country-code request header
and redirects to a country-specific origin URL. Falls back to
BASE_ORIGIN when no country-specific mapping is configured.

Required configuration:
  - Environment variable: BASE_ORIGIN (fallback origin URL)
  - Environment variable: <COUNTRY_CODE> (optional per-country origin URLs, e.g. US, DE, GB)
*/

use anyhow::{anyhow, Error, Result};
use fastedge::body::Body;
use fastedge::dictionary;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let base_origin =
        dictionary::get("BASE_ORIGIN").ok_or_else(|| anyhow!("BASE_ORIGIN is not set"))?;

    let country_code = req
        .headers()
        .get("geoip-country-code")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    let redirect_origin = if !country_code.is_empty() {
        dictionary::get(country_code).unwrap_or(base_origin)
    } else {
        base_origin
    };

    Response::builder()
        .status(StatusCode::FOUND)
        .header("location", &redirect_origin)
        .body(Body::empty())
        .map_err(Error::msg)
}
