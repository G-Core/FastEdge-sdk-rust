/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example WASI-HTTP app demonstrating geo-based redirects.

Reads the country code from the geoip-country-code request header
and redirects to a country-specific origin URL. Falls back to
BASE_ORIGIN when no country-specific mapping is configured.

Required configuration:
  - Environment variable: BASE_ORIGIN (fallback origin URL)
  - Environment variable: <COUNTRY_CODE> (optional per-country origin URLs, e.g. US, DE, GB)
*/

use fastedge::dictionary;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let base_origin = match dictionary::get("BASE_ORIGIN") {
        Some(origin) => origin,
        None => {
            return Ok(Response::builder()
                .status(500)
                .body(Body::from("BASE_ORIGIN is not set"))?);
        }
    };

    let country_code = req
        .headers()
        .get("geoip-country-code")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let redirect_origin = if !country_code.is_empty() {
        dictionary::get(&country_code).unwrap_or(base_origin)
    } else {
        base_origin
    };

    Ok(Response::builder()
        .status(302)
        .header("location", &redirect_origin)
        .body(Body::empty())?)
}
