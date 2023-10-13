use http::request::Parts;

use crate::bindgen::gcore::fastedge::{http::Method, http_client};
use crate::body::Body;
use crate::Error;

/// implementation of http_client
pub fn send_request(req: ::http::Request<Body>) -> Result<::http::Response<Body>, Error> {
    // convert http::Request<Body> to http_client::Response
    let (parts, body) = req.into_parts();
    let request = (&parts, &body).try_into()?;

    // call http-backend component send_request
    let response = http_client::send_request(&request).map_err(|e| Error::BindgenHttpError(e))?;

    translate_http_client_to_response(response)
}

/// translate http::Response<Body> from http_client::Response
fn translate_http_client_to_response(
    res: http_client::Response,
) -> Result<::http::Response<Body>, Error> {
    let builder = http::Response::builder().status(res.status);
    let builder = if let Some(headers) = res.headers {
        headers
            .iter()
            .fold(builder, |builder, (k, v)| builder.header(k, v))
    } else {
        builder
    };

    let body = res.body.map(|b| Body::from(b)).unwrap_or_default();
    let response = builder.body(body).map_err(|_| Error::InvalidBody)?;
    Ok(response)
}

impl TryFrom<(&Parts, &Body)> for http_client::Request {
    type Error = Error;

    fn try_from((parts, body): (&Parts, &Body)) -> Result<Self, Self::Error> {
        let method = to_http_client_method(&parts.method)?;

        let headers = parts
            .headers
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    value.to_str().map(|s| s.to_string()).unwrap(),
                )
            })
            .collect::<Vec<(String, String)>>();

        Ok(http_client::Request {
            method,
            uri: parts.uri.to_string(),
            headers,
            body: Some(body.to_vec()),
        })
    }
}

fn to_http_client_method(method: &::http::Method) -> Result<Method, Error> {
    Ok(match method {
        &::http::Method::GET => Method::Get,
        &::http::Method::POST => Method::Post,
        &::http::Method::PUT => Method::Put,
        &::http::Method::DELETE => Method::Delete,
        &::http::Method::HEAD => Method::Head,
        &::http::Method::PATCH => Method::Patch,
        method => return Err(Error::UnsupportedMethod(method.to_owned())),
    })
}
