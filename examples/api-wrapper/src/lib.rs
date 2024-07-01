/*
Example app to demonstrate how to wrap several API calls into single edge app.
This example gets device status from SmartThings API and toggles it
App needs following env vars to be set:
PASSWORD - password to check user's permissions, simplest form of authentication
DEVICE - device ID in SmartThings
TOKEN - SmartThings API token
*/

use std::env;
use fastedge::{
    body::Body,
    http::{header, Error, Method, Request, Response, StatusCode},
};
use url::Url;
use serde_json::{Value, from_str};

const API_BASE: &str = "https://api.smartthings.com/v1/devices/";

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    match req.method() {
        &Method::GET | &Method::HEAD => (),
        _ => return Response::builder().status(StatusCode::METHOD_NOT_ALLOWED).header(header::ALLOW, "GET, HEAD").body(Body::from("This method is not allowed\n"))
    };

    let Ok(expected_pass) = env::var("PASSWORD") else {
        return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Misconfigured app\n"));
    };
    let provided_pass = match req.headers().get(header::AUTHORIZATION) {
        None => return Response::builder().status(StatusCode::FORBIDDEN).body(Body::from("No auth header\n")),
        Some(h) => match h.to_str() {
            Ok(v) => v,
            Err(_) => return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("cannot process auth header"))
        }
    };
    if expected_pass != provided_pass {
        return Response::builder().status(StatusCode::FORBIDDEN).body(Body::empty());
    }

    let Ok(device) = env::var("DEVICE") else {
        return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Misconfigured app\n"))
    };
    let Ok(token) = env::var("TOKEN") else {
        return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Misconfigured app\n"))
    };

    let wanted_status = match get_device_status(&token, &device) {
        Err(status) => {
            println!("cannot get device's current status");
            return Response::builder().status(status).body(Body::empty())
        },
        Ok(s) => match s.as_str() {
            "off" => "on",
            "on" => "off",
            _ => return Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("Unsupported device status\n")),
        }
    };

    let res = match send_device_command(&token, &device, wanted_status) {
        Err(status) => status,
        Ok(s) => match s.as_str() {
            "ACCEPTED" => StatusCode::NO_CONTENT,
            _ => StatusCode::NOT_FOUND
        }
    };

    Response::builder().status(res).body(Body::empty())
}

fn get_device_status(token: &str, device: &str) -> Result<String, StatusCode> {
    let req = Request::builder()
        .method(Method::GET)
        .header(header::ACCEPT, "application/json")
        .header(header::AUTHORIZATION, "Bearer ".to_string() + token)
        .header(header::PRAGMA, "no-cache")
        .uri(API_BASE.to_string() + device + "/status")
        .body(Body::empty())
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    let rsp = match request(req) {
        Err(status_code) => return Err(status_code),
        Ok(r) => r,
    };

    let json: Value = match from_str(String::from_utf8(rsp.body().to_vec()).or(Err(StatusCode::INTERNAL_SERVER_ERROR))?.as_str()) {
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(j) => j
    };
    let status = json.get(&"components").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .get(&"main").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .get(&"switch").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .get(&"switch").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?   // this is correct, "switch" two times, this is the structuire of this JSON schema
        .get(&"value").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?.to_string();

    Ok(status.trim_matches('"').to_string())
}

fn send_device_command(token: &str, device: &str, command: &str) -> Result<String, StatusCode> {
    let req = Request::builder()
        .method(Method::POST)
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer ".to_string() + token)
        .uri(API_BASE.to_string() + device + "/commands")
        .body(Body::from("{\"commands\": [{\"capability\": \"switch\", \"command\": \"".to_string() + command + "\"}]}"))
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    let rsp = match request(req) {
        Err(status_code) => return Err(status_code),
        Ok(r) => r,
    };

    let json: Value = from_str(String::from_utf8(rsp.body().to_vec()).or(Err(StatusCode::INTERNAL_SERVER_ERROR))?.as_str())
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;
    let status = json.get(&"results").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?
        .as_array().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?[0]
        .get(&"status").ok_or(StatusCode::INTERNAL_SERVER_ERROR)?.to_string();

    Ok(status.trim_matches('"').to_string())
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

            let loc = new_url.as_str();
            let host = new_url.host().ok_or(StatusCode::INTERNAL_SERVER_ERROR)?.to_string();
            println!("Redirect to {}", loc);
            let sub_req = Request::builder()
                .method(Method::GET)
                .header(header::HOST, host)
                .uri(loc)
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
