/*
* Copyright 2025 G-Core Innovations SARL
*/
//! HTTP client implementation for outbound requests.
//!
//! This module provides the HTTP client functionality for making requests to backend services
//! from your FastEdge application.

use http::request::Parts;

use crate::body::Body;
use crate::gcore::fastedge::{http::Method, http_client};
use crate::Error;

/// Sends an HTTP request to a backend service.
///
/// This function allows your FastEdge application to make outbound HTTP requests
/// to backend services, APIs, or other web resources.
///
/// # Arguments
///
/// * `req` - The HTTP request to send
///
/// # Returns
///
/// Returns the HTTP response from the backend service, or an error if the request fails.
///
/// # Errors
///
/// Returns an error if:
/// - The HTTP method is not supported
/// - The request is malformed
/// - The backend service is unreachable
/// - The response is invalid
///
/// # Examples
///
/// ```no_run
/// use fastedge::body::Body;
/// use fastedge::http::{Method, Request};
/// use fastedge::send_request;
///
/// // Create a GET request
/// let request = Request::builder()
///     .method(Method::GET)
///     .uri("https://api.example.com/users")
///     .header("User-Agent", "FastEdge/1.0")
///     .header("Accept", "application/json")
///     .body(Body::empty())?;
///
/// // Send the request
/// let response = send_request(request)?;
///
/// println!("Status: {}", response.status());
/// println!("Body length: {}", response.body().len());
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ```no_run
/// use fastedge::body::Body;
/// use fastedge::http::{Method, Request};
/// use fastedge::send_request;
///
/// // POST with JSON body
/// let json_data = r#"{"name": "John", "email": "john@example.com"}"#;
/// let request = Request::builder()
///     .method(Method::POST)
///     .uri("https://api.example.com/users")
///     .header("Content-Type", "application/json")
///     .body(Body::from(json_data))?;
///
/// let response = send_request(request)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn send_request(req: ::http::Request<Body>) -> Result<::http::Response<Body>, Error> {
    // convert http::Request<Body> to http_client::Response
    let (parts, body) = req.into_parts();
    let request = (&parts, &body).try_into()?;

    // call http-backend component send_request
    let response = http_client::send_request(&request).map_err(Error::BindgenHttpError)?;

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

    let body = res.body.map(Body::from).unwrap_or_default();
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
        &::http::Method::OPTIONS => Method::Options,
        method => return Err(Error::UnsupportedMethod(method.to_owned())),
    })
}
