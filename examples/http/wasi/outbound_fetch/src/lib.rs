use anyhow::anyhow;
use wstd::http::body::Body;
use wstd::http::{Client, Request, Response};
use serde_json::{json, Value};

#[wstd::http_server]
async fn main(_request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let upstream_req = Request::get("http://jsonplaceholder.typicode.com/users")
        .body(Body::empty())
        .map_err(|e| anyhow!("failed to build request: {e}"))?;

    let client = Client::new();
    let upstream_resp = client
        .send(upstream_req)
        .await
        .map_err(|e| anyhow!("request failed: {e}"))?;

    let (_, mut body) = upstream_resp.into_parts();
    let body_bytes = body.contents().await?;
    let users: Value = serde_json::from_slice(body_bytes)?;

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

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Body::from(result.to_string()))?)
}
