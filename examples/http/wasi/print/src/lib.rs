use wstd::http::body::Body;
use wstd::http::{Request, Response, StatusCode};

#[wstd::http_server]
async fn main(request: Request<Body>) -> anyhow::Result<Response<Body>> {
    let mut body: String = "Method: ".to_string();
    body.push_str(request.method().as_str());

    body.push_str("\nURL: ");
    body.push_str(request.uri().to_string().as_str());

    body.push_str("\nHeaders:");
    for (h, v) in request.headers() {
        body.push_str("\n    ");
        body.push_str(h.as_str());
        body.push_str(": ");
        match v.to_str() {
            Err(_) => body.push_str("not a valid text"),
            Ok(a) => body.push_str(a),
        }
    }

    println!("{}", body);

    Ok(Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(body))?)
}
