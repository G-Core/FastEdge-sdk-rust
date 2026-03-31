use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let url = req.uri().to_string();

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "text/plain;charset=UTF-8")
        .body(Body::from(format!("Hello, you made a request to {url}")))
        .map_err(Into::into)
}
