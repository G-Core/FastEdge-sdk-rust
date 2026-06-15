/*
 * Copyright 2025 G-Core Innovations SARL
 */
/*
Static assets example.

Embeds three files (`index.html`, `style.css`, `logo.svg`) into the wasm binary
at compile time via `include_str!` and serves them by path at request time. The
wasm runtime has no file system, so all assets must be embedded this way.

For binary assets, use `include_bytes!` and wrap the result in
`bytes::Bytes::from_static` when constructing the response body.

Mirror of the FastEdge-sdk-js `static-assets` example.
*/

use wstd::http::body::Body;
use wstd::http::{Request, Response, StatusCode};

struct Asset {
    content_type: &'static str,
    body: &'static str,
}

static INDEX_HTML: Asset = Asset {
    content_type: "text/html; charset=utf-8",
    body: include_str!("../assets/index.html"),
};
static STYLE_CSS: Asset = Asset {
    content_type: "text/css; charset=utf-8",
    body: include_str!("../assets/style.css"),
};
static LOGO_SVG: Asset = Asset {
    content_type: "image/svg+xml",
    body: include_str!("../assets/logo.svg"),
};

fn lookup(path: &str) -> Option<&'static Asset> {
    match path {
        "/" | "/index.html" => Some(&INDEX_HTML),
        "/style.css" => Some(&STYLE_CSS),
        "/logo.svg" => Some(&LOGO_SVG),
        _ => None,
    }
}

#[wstd::http_server]
async fn main(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let path = req.uri().path();
    match lookup(path) {
        Some(asset) => Ok(Response::builder()
            .status(StatusCode::OK)
            .header("content-type", asset.content_type)
            .body(Body::from(asset.body))?),
        None => Ok(Response::builder()
            .status(StatusCode::NOT_FOUND)
            .header("content-type", "text/plain; charset=utf-8")
            .body(Body::from(format!("Not found: {path}\n")))?),
    }
}
