use anyhow::{anyhow, Error, Result};
use fastedge::body::Body;
use fastedge::http::{Method, Request, Response, StatusCode};

#[allow(dead_code)]
#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let (parts, body) = req.into_parts();
    let query = parts
        .uri
        .query()
        .ok_or(anyhow!("missing uri query parameter"))?;
    let params = querystring::querify(query);
    let url = params
        .iter()
        .find(|(k, _)| k == &"url")
        .ok_or(anyhow!("missing url parameter"))?;
    let url = urlencoding::decode(url.1)?.to_string();
    println!("url = {:?}", url);
    let request = Request::builder().uri(url).method(Method::GET).body(body)?;

    let response = fastedge::send_request(request).map_err(Error::msg)?;

    Response::builder()
        .status(StatusCode::OK)
        .body(Body::from(format!(
            "len = {}, content-type = {:?}",
            response.body().len(),
            response.headers().get("Content-Type")
        )))
        .map_err(Error::msg)
}
