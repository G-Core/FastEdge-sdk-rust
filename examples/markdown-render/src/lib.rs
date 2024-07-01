/*
Example app to demonstrate how to remder Markdown docs to HTML
App needs following env vars to be set:
BASE - origin server URL. Whateven passed in URL path for this app is concatenanted to BASE
*/

use fastedge::{
    http::{
        header,
        Request,
        Response,
        StatusCode,
        Method,
        Error
    },
    body::Body
};
use std::env;
use url::Url;
use pulldown_cmark::{Parser, Options};

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    match req.method() {
        &Method::GET | &Method::HEAD => (),
        _ => return Response::builder().status(StatusCode::METHOD_NOT_ALLOWED).header(header::ALLOW, "GET, HEAD").body(Body::from("This method is not allowed\n"))
    };

    let Ok(base) = env::var("BASE") else {
        return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Misconfigured app\n"))
    };
    let base = base.trim_end_matches('/').to_string();

    let path = req.uri().path();
    if path.is_empty() || path == "/" {
        return Response::builder().status(StatusCode::BAD_REQUEST).body(Body::from("Missing file path\n"))
    }

    let Ok(sub_req) = Request::builder()
        .method(Method::GET)
        .header(header::USER_AGENT, "fastedge")
        .uri(base + path)
        .body(Body::empty()) else {
            return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty());
        };

    let rsp = match request(sub_req) {
        Err(status) => return Response::builder().status(status).body(Body::empty()),
        Ok(s) => s
    };
    let Ok(md) = String::from_utf8(rsp.body().to_vec()) else {
        return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::empty());
    };

    let parser = Parser::new_ext(
        md.as_str(),
        Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES
    );

    let mut html = String::new();
    html.push_str("<!DOCTYPE html><html>");
    match env::var("HEAD").ok() {
        None => {},
        Some(h) => {
            html.push_str("<head>");
            html.push_str(h.as_str());
            html.push_str("</head>");
        }
    }
    
    html.push_str("<body>");
    pulldown_cmark::html::push_html(&mut html, parser);
    html.push_str("</body></html>");

    let rsp = Response::builder()
        .status(StatusCode::OK)
        .header(header::CONTENT_TYPE, mime::TEXT_HTML.to_string())
        .body(Body::from(html))?;

    Ok(rsp)
}

fn request(req: Request<Body>) -> Result<Response<Body>, StatusCode> {
    let rsp = match fastedge::send_request(req) {
        Err(error) => {
            let status_code = match error {
                fastedge::Error::UnsupportedMethod(_) => StatusCode::METHOD_NOT_ALLOWED,
                fastedge::Error::BindgenHttpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                fastedge::Error::HttpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
                fastedge::Error::InvalidBody => StatusCode::BAD_REQUEST,
                fastedge::Error::InvalidStatusCode(_) => StatusCode::BAD_REQUEST
            };
            return Err(status_code);
        }
        Ok(r) => r,
    };

    let status = rsp.status();
    if is_redirect(status) {
        if let Some(location) = rsp.headers().get(header::LOCATION) {
            let new_url = Url::parse(
                location.to_str().or(Err(StatusCode::INTERNAL_SERVER_ERROR))?)
                .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

            let sub_req = Request::builder()
                .method(Method::GET)
                .uri(new_url.as_str())
                .body(Body::empty())
                .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

            return request(sub_req);
        }
    }
    if status == StatusCode::OK {
        return Ok(rsp);
    }

    Err(status)
}

// List of acceptible 300-series redirect codes.
const REDIRECT_CODES: &[StatusCode] = &[
    StatusCode::MOVED_PERMANENTLY,
    StatusCode::FOUND,
    StatusCode::SEE_OTHER,
    StatusCode::TEMPORARY_REDIRECT,
    StatusCode::PERMANENT_REDIRECT,
];

fn is_redirect(status_code: StatusCode) -> bool {
    return REDIRECT_CODES.contains(&status_code)
}
