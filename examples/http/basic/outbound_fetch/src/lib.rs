use anyhow::{Error, Result};
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};
use serde_json::{json, Value};

#[fastedge::http]
fn main(_req: Request<Body>) -> Result<Response<Body>> {
    let upstream_req = Request::builder()
        .uri("http://jsonplaceholder.typicode.com/users")
        .body(Body::empty())?;

    let upstream_resp = fastedge::send_request(upstream_req).map_err(Error::msg)?;

    let body_bytes = upstream_resp.body().to_vec();
    let users: Value = serde_json::from_slice(&body_bytes)?;

    let sliced_users = match users.as_array() {
        Some(arr) => Value::Array(arr.iter().take(5).cloned().collect()),
        None => Value::Array(vec![]),
    };

    let result = json!({
        "users": sliced_users,
        "total": 5,
        "skip": 0,
        "limit": 30,
    });

    Response::builder()
        .status(StatusCode::OK)
        .header("content-type", "application/json")
        .body(Body::from(result.to_string()))
        .map_err(Into::into)
}
