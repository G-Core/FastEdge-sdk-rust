use wstd::http::body::Body;
use wstd::http::{Request, Response, Client, HeaderValue};

#[wstd::http_server]
async fn main(_: Request<Body>) -> Result<Response<Body>, anyhow::Error> {
    let request = Request::get("https://92.113.151.43")
        .header("Host", HeaderValue::from_str("example.com")?)
        .body(Body::empty())?;

    Client::new().send(request).await
}