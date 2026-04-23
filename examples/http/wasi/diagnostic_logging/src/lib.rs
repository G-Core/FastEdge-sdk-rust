/*
 * Copyright 2025 G-Core Innovations SARL
 */
/*
Diagnostic logging example.

Tiny pass-through proxy that writes a single `set_user_diag` tag per request
summarising the outcome (config_error, origin_unreachable, or proxied). The
tag appears in the FastEdge platform's per-request log viewer and is distinct
from stdout — it's intended for filterable outcome labels, not verbose
traces.

Required configuration:
  - Environment variable: ORIGIN_URL (origin to proxy to)

Uses `logfmt`-ish formatting (`outcome=<verb> key=value ...`) so the tag is
easy to slice in log search tooling.
*/

use std::env;

use fastedge::utils::set_user_diag;
use wstd::http::body::Body;
use wstd::http::{Client, Request, Response};

#[wstd::http_server]
async fn main(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let method = req.method().as_str().to_string();
    let path = req.uri().path().to_string();

    let origin = match env::var("ORIGIN_URL") {
        Ok(u) if !u.trim().is_empty() => u,
        _ => {
            set_user_diag("outcome=config_error reason=origin_missing");
            return Ok(Response::builder()
                .status(500)
                .header("content-type", "text/plain; charset=utf-8")
                .body(Body::from("ORIGIN_URL is not configured"))?);
        }
    };

    let outbound = Request::get(&origin).body(Body::empty())?;
    let resp = match Client::new().send(outbound).await {
        Ok(r) => r,
        Err(e) => {
            set_user_diag(&format!(
                "outcome=origin_unreachable method={method} path={path} err={e}"
            ));
            return Ok(Response::builder()
                .status(502)
                .header("content-type", "text/plain; charset=utf-8")
                .body(Body::from("origin unreachable"))?);
        }
    };

    let status = resp.status().as_u16();
    set_user_diag(&format!(
        "outcome=proxied method={method} path={path} status={status}"
    ));

    let (parts, mut body) = resp.into_parts();
    let bytes = body.contents().await?;
    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    Ok(Response::builder()
        .status(parts.status)
        .header("content-type", content_type)
        .body(Body::from(bytes))?)
}
