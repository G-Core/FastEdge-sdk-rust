use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let url = request.uri().to_string();

    Ok(Response::builder()
        .status(200)
        .header("content-type", "text/plain;charset=UTF-8")
        .body(Body::from(format!("Hello, you made a request to {url}")))?)
}
