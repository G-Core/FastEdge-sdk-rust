use fastedge::secret;
use std::env;
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let username = env::var("USERNAME").unwrap_or_default();
    let password = match secret::get("PASSWORD") {
        Ok(Some(value)) => value,
        _ => String::new(),
    };

    Ok(Response::builder()
        .status(200)
        .body(Body::from(format!(
            "Username: {username}, Password: {password}"
        )))?)
}
