use anyhow::Result;
use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};

#[allow(dead_code)]
#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let mut body: String = "Method: ".to_string();
    body.push_str(req.method().as_str());

    body.push_str("\nURL: ");
    body.push_str(req.uri().to_string().as_str());

    body.push_str("\nHeaders:");
    for (h, v) in req.headers() {
        body.push_str("\n    ");
        body.push_str(h.as_str());
        body.push_str(": ");
        match v.to_str() {
            Err(_) => body.push_str("not a valid text"),
            Ok(a) => body.push_str(a),
        }
    }
    let res = Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body))?;
    Ok(res)
}
