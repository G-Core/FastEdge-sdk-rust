use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[allow(dead_code)]
#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    let res = Response::builder()
        .status(StatusCode::OK)
        .body(Body::empty())?;
    Ok(res)
}
