
use anyhow::{Error, Result};

use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};
use fastedge::secret;

#[allow(dead_code)]
#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    let value = match secret::get("SECRET") {
        Ok(value) => value,
        Err(secret::Error::AccessDenied) => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::empty())
                .map_err(Error::msg);
        },
        Err(secret::Error::Other(msg)) => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body(Body::from(msg))
                .map_err(Error::msg);
        },
        Err(secret::Error::DecryptError) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
                .map_err(Error::msg);
        }
    };

    if value.is_none() {
        return Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .map_err(Error::msg);
    }

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(value.unwrap_or_default()))
        .map_err(Error::msg)
}
