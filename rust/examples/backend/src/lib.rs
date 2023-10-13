use anyhow::{anyhow, Error, Result};
use fastedge::body::Body;
use fastedge::http::{Method, Request, Response, StatusCode};

#[allow(dead_code)]
#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let query = req
        .uri()
        .query()
        .ok_or(anyhow!("missing uri query parameter"))?;
    let params = querystring::querify(query);
    let url = params
        .iter()
        .find(|(k, _)| k == &"url")
        .ok_or(anyhow!("missing url parameter"))?;
    let request = Request::builder()
        .uri(url.1)
        .method(Method::GET)
        .body(Body::empty())?;

    let response = fastedge::send_request(request).map_err(Error::msg)?;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!("len = {}", response.body().len())))
        .map_err(Error::msg)
}
