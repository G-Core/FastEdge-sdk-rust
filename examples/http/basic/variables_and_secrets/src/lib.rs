use anyhow::Result;
use fastedge::body::Body;
use fastedge::dictionary;
use fastedge::http::{Request, Response, StatusCode};

#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    let username = dictionary::get("USERNAME").unwrap_or_default();

    let password = match fastedge::secret::get("PASSWORD") {
        Ok(Some(value)) => value,
        _ => String::new(),
    };

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(
            "Username: {username}, Password: {password}"
        )))
        .map_err(Into::into)
}
