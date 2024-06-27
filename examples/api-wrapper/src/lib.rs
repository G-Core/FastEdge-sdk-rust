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

const API_BASE: &str = "https://api.smartthings.com/v1/devices/";

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    match req.method() {
        &Method::GET | &Method::HEAD => (),
        _ => return Response::builder().status(StatusCode::METHOD_NOT_ALLOWED).header(header::ALLOW, "GET, HEAD").body(Body::from("This method is not allowed\n"))
    };

    let expected_pass = match env::var("PASSWORD") {
        Err(_) => return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Misconfigured app\n")),
        Ok(r) => r
    };
    let provided_pass = match req.headers().get(header::AUTHORIZATION) {
        None => return Response::builder().status(StatusCode::FORBIDDEN).body(Body::from("No auth header\n")),
        Some(h) => h.to_str().unwrap(),
    };
    if expected_pass != provided_pass {
        return Response::builder().status(StatusCode::FORBIDDEN).body(Body::empty());
    }

    let device = match env::var("DEVICE") {
        Err(_) => return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Misconfigured app\n")),
        Ok(r) => r
    };
    let token = match env::var("TOKEN") {
        Err(_) => return Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body(Body::from("Misconfigured app\n")),
        Ok(r) => r
    };

    let wanted_status = match get_device_status(&token, &device) {
        Err(status) => return Response::builder().status(status).body(Body::empty()),
        Ok(s) => match s.as_str() {
            "off" => "on",
            "on" => "off",
            _ => return Response::builder().status(StatusCode::NOT_FOUND).body(Body::from("Misconfigured app\n")),
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
        .expect("error building the request");

    let rsp = match request(req) {
        Err(status_code) => return Err(status_code),
        Ok(r) => r,
    };

    let json: serde_json::Value = match serde_json::from_str(String::from_utf8(rsp.body().to_vec()).expect("getting device status").as_str()) {
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(j) => j
    };
    let status = match &json["components"]["main"]["switch"]["switch"]["value"] {
        serde_json::Value::String(s) => s.trim_matches('"'),
        _ => return Err(StatusCode::NOT_FOUND)
    };

    Ok(status.to_string())
}

fn send_device_command(token: &str, device: &str, command: &str) -> Result<String, StatusCode> {
    let req = Request::builder()
        .method(Method::POST)
        .header(header::ACCEPT, "application/json")
        .header(header::CONTENT_TYPE, "application/json")
        .header(header::AUTHORIZATION, "Bearer ".to_string() + token)
        .uri(API_BASE.to_string() + device + "/commands")
        .body(Body::from("{\"commands\": [{\"capability\": \"switch\", \"command\": \"".to_string() + command + "\"}]}"))
        .expect("error building the request");

    let rsp = match request(req) {
        Err(status_code) => return Err(status_code),
        Ok(r) => r,
    };

    let json: serde_json::Value = match serde_json::from_str(String::from_utf8(rsp.body().to_vec()).expect("command response").as_str()) {
        Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR),
        Ok(j) => j
    };
    let status = match &json["results"][0]["status"] {
        serde_json::Value::String(s) => s.trim_matches('"'),
        _ => return Err(StatusCode::INTERNAL_SERVER_ERROR)
    };

    Ok(status.to_string())
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
            let new_url = match Url::parse(location.to_str().unwrap()) {
                Ok(u) => u,
                Err(_) => return Err(StatusCode::INTERNAL_SERVER_ERROR)
            };        

            let loc =  new_url.as_str();
            let host = new_url.host().unwrap().to_string();
            println!("Redirect to {}", loc);
            let sub_req = Request::builder()
                .method(Method::GET)
                .header(header::HOST, host)
                .uri(loc)
                .body(Body::empty())
                .expect("error building the request");

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
