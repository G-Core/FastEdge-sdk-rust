use crate::body::Body;
use crate::{witx_bindgen::http_backend, Error};

/// implementation of http_backend
pub fn send_request(req: ::http::Request<Body>) -> Result<::http::Response<Body>, Error> {
    // convert http::Request<Body> to http_backend::Request
    //let (parts, body) = req.into_parts();
    let method = to_http_client_method(&req.method())?;
    let headers = req
        .headers()
        .iter()
        .map(|(name, value)| http_backend::Header {
            key: name.as_str().as_bytes(),
            value: value.to_str().unwrap().as_bytes(),
        })
        .collect::<Vec<http_backend::Header<'_>>>();
    let uri = req.uri().to_string();
    let request = http_backend::Request {
        method,
        uri: uri.as_bytes(),
        headers: headers.as_slice(),
        body: &req.body(),
    };

    println!("http_backend::send_request()");

    // call http-backend component send_request
    let response =
        unsafe { http_backend::send_request(request) }.map_err(|e| Error::BackendError(e))?;

    println!("http_backend::send_request() done");
    let builder = http::Response::builder().status(response.status);
    let builder = response
        .headers
        .iter()
        .fold(builder, |builder, h| builder.header(h.key, h.value));

    let body = Body::from(response.body);
    let response = builder.body(body).map_err(|_| Error::InvalidBody)?;
    Ok(response)
}

fn to_http_client_method(method: &::http::Method) -> Result<http_backend::Method, Error> {
    Ok(match method {
        &::http::Method::GET => http_backend::METHOD_GET,
        &::http::Method::POST => http_backend::METHOD_POST,
        &::http::Method::PUT => http_backend::METHOD_PUT,
        &::http::Method::DELETE => http_backend::METHOD_DELETE,
        &::http::Method::HEAD => http_backend::METHOD_HEAD,
        &::http::Method::PATCH => http_backend::METHOD_PATCH,
        method => return Err(Error::UnsupportedMethod(method.to_owned())),
    })
}
