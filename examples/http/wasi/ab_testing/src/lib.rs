/*
 * Copyright 2025 G-Core Innovations SARL
 */
/*
Cookie-based A/B testing example.

Reads or creates an `x-fastedge-abid` cookie, uses its value to deterministically
assign the visitor to weighted variants of each configured test, then proxies the
request to `OUTBOUND_URL` with the variant assignments attached as `ab-test-<name>`
headers. The origin response is returned verbatim with a `set-cookie` header so
returning visitors receive the same variants on subsequent visits.

Required configuration:
  - Environment variable: OUTBOUND_URL (downstream origin to proxy to)

Mirror of the FastEdge-sdk-js `ab-testing` example.
*/

use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::anyhow;
use wstd::http::body::Body;
use wstd::http::{Client, Request, Response};

struct VariantWeight {
    variant: &'static str,
    weight: f64,
}

struct AbTest {
    name: &'static str,
    variants: &'static [VariantWeight],
}

static TESTS: &[AbTest] = &[
    AbTest {
        name: "logo",
        variants: &[
            VariantWeight { variant: "hops", weight: 50.0 },
            VariantWeight { variant: "bottle", weight: 50.0 },
        ],
    },
    AbTest {
        name: "font",
        variants: &[
            VariantWeight { variant: "exo2", weight: 40.0 },
            VariantWeight { variant: "gloria", weight: 65.0 },
            VariantWeight { variant: "standard", weight: 45.0 },
        ],
    },
];

const AB_COOKIE: &str = "x-fastedge-abid";

#[wstd::http_server]
async fn main(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let outbound_url = match env::var("OUTBOUND_URL") {
        Ok(u) if !u.trim().is_empty() => u,
        _ => {
            return Ok(Response::builder()
                .status(500)
                .body(Body::from(
                    "OUTBOUND_URL environment variable is not configured",
                ))?);
        }
    };

    let raw_cookie = req
        .headers()
        .get("cookie")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_string();

    let (xid, cleaned_cookie) = match extract_abid(&raw_cookie) {
        Some(existing) if is_valid_xid(existing) => {
            (existing.to_string(), strip_abid(&raw_cookie))
        }
        _ => (generate_xid(), raw_cookie.clone()),
    };

    // Build the outbound request: copy incoming headers except `host` and
    // `cookie` (which we handle specially), replace the cookie with a version
    // that has the abid stripped (so the origin never sees it), and attach an
    // `ab-test-<name>` header for every configured test.
    let mut builder = Request::get(&outbound_url);
    for (name, value) in req.headers() {
        let n = name.as_str();
        if n == "host" || n == "cookie" {
            continue;
        }
        if let Ok(v) = value.to_str() {
            builder = builder.header(n, v);
        }
    }
    if !cleaned_cookie.trim().is_empty() {
        builder = builder.header("cookie", cleaned_cookie);
    }
    for test in TESTS {
        if let Some(variant) = assign_variant(&xid, test) {
            builder = builder.header(format!("ab-test-{}", test.name), variant);
        }
    }

    let outbound_req = builder
        .body(Body::empty())
        .map_err(|e| anyhow!("failed to build outbound request: {e}"))?;

    let outbound_resp = Client::new()
        .send(outbound_req)
        .await
        .map_err(|e| anyhow!("outbound request failed: {e}"))?;

    let (parts, mut body) = outbound_resp.into_parts();
    let body_bytes = body.contents().await?;

    let content_type = parts
        .headers
        .get("content-type")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let xid_cookie =
        format!("{AB_COOKIE}={xid}; Max-Age=31536000; Path=/; Secure; HttpOnly; SameSite=Lax");

    Ok(Response::builder()
        .status(parts.status)
        .header("content-type", content_type)
        .header("set-cookie", xid_cookie)
        .body(Body::from(body_bytes))?)
}

fn extract_abid(cookie_header: &str) -> Option<&str> {
    let needle = format!("{AB_COOKIE}=");
    cookie_header
        .split(';')
        .map(str::trim)
        .find_map(|p| p.strip_prefix(needle.as_str()))
}

fn strip_abid(cookie_header: &str) -> String {
    let needle = format!("{AB_COOKIE}=");
    cookie_header
        .split(';')
        .map(str::trim)
        .filter(|p| !p.is_empty())
        .filter(|p| !p.starts_with(needle.as_str()))
        .collect::<Vec<_>>()
        .join("; ")
}

fn is_valid_xid(xid: &str) -> bool {
    matches!(xid.parse::<f64>(), Ok(v) if (0.0..1.0).contains(&v))
}

/// Generate a pseudo-random A/B id of the form `"0.NNNN"`.
///
/// Uses request-time nanoseconds as a weak entropy source. For production,
/// prefer a cryptographic RNG (e.g. `rand` wired to `wasi-random`).
fn generate_xid() -> String {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    format!("0.{:04}", now.subsec_nanos() % 10000)
}

fn assign_variant(xid: &str, test: &AbTest) -> Option<&'static str> {
    let xid_value: f64 = xid.parse().ok()?;
    let xid_percentage = xid_value * 100.0;
    let total: f64 = test.variants.iter().map(|v| v.weight).sum();
    if total == 0.0 {
        return None;
    }
    let mut start = 0.0;
    for vw in test.variants {
        let percentage = (vw.weight / total) * 100.0;
        let end = start + percentage;
        if xid_percentage >= start && xid_percentage < end {
            return Some(vw.variant);
        }
        start = end;
    }
    None
}
