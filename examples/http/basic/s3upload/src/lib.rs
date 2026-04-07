use std::time::Duration;

use fastedge::{
    body::Body,
    http::{header, Error, Method, Request, Response, StatusCode},
};
use rusty_s3::{Bucket, Credentials, S3Action, UrlStyle};
use std::{collections::HashMap, env};
use url::Url;

#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>, Error> {
    match req.method() {
        // Allow only POST and PUT requests
        &Method::POST | &Method::PUT => (),

        &Method::OPTIONS => {
            return Response::builder()
                .status(StatusCode::NO_CONTENT)
                .body(Body::empty());
        }

        // Deny anything else
        _ => {
            return Response::builder()
                .status(StatusCode::METHOD_NOT_ALLOWED)
                .header(header::ALLOW, "PUT, POST")
                .body(Body::from("This method is not allowed\n"));
        }
    };

    /* get request params */
    let query_pairs = |q: &str| {
        q.split('&')
            .filter_map(|q| {
                let mut i = q.splitn(2, '=');
                let k = i.next()?;
                let v = i.next()?;
                Some((k, v))
            })
            .map(|(k, v)| (k.to_owned(), v.to_owned()))
            .collect::<HashMap<String, String>>()
    };
    let hash_query: HashMap<String, String> = req.uri().query().map_or(HashMap::new(), query_pairs);

    let fname = match hash_query.get("name") {
        None => {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body(Body::from("Malformed request\n"))
        }
        Some(i) => i,
    };
    if req.body().is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body(Body::from("Malformed request\n"));
    }
    let content_type = match req.headers().get("Content-Type") {
        None => "application/octet-stream", // default MIME type
        Some(v) => v.to_str().unwrap(),
    };
    let content_type = content_type.to_owned();
    let content = req.into_body();

    match env::var("MAX_FILE_SIZE").ok() {
        None => {}
        Some(l) => match l.parse::<usize>() {
            Err(_) => {}
            Ok(v) => {
                if content.len() > v {
                    let msg = format!("File exceeds allowed limit of {} bytes\n", v);
                    return Response::builder()
                        .status(StatusCode::PAYLOAD_TOO_LARGE)
                        .body(Body::from(msg.as_str().to_owned()));
                }
            }
        },
    }

    let (signed_url, host) = match prepare_s3(fname) {
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::from("App misconfigured\n"))
        }
        Ok((u, h)) => (u, h),
    };

    /* build outgoing req */
    let out_req = Request::builder()
        .method(Method::PUT)
        .uri(signed_url.as_str())
        .header("Host", host)
        .header("Accept-Encoding", "identity")
        .header("Content-Length", content.len().to_string())
        .header("Content-Type", content_type);

    let Ok(req) = out_req.body(content) else {
        return Response::builder()
            .status(StatusCode::INTERNAL_SERVER_ERROR)
            .body(Body::from("Malformed request\n"));
    };

    let rsp = match fastedge::send_request(req) {
        Err(_) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(Body::empty())
        }
        Ok(r) => r,
    };
    let (parts, body) = rsp.into_parts();
    let body = if parts.status == StatusCode::OK {
        let mut tmp_url = signed_url.clone();
        tmp_url.set_query(None);
        Body::from(tmp_url.to_string())
    } else {
        body
    };
    Ok(Response::from_parts(parts, body))
}

fn prepare_s3(fname: &str) -> anyhow::Result<(Url, String)> {
    /* read S3 access params from env */
    let access_key = env::var("ACCESS_KEY")?;
    let secret_key = env::var("SECRET_KEY")?;
    let region = env::var("REGION")?;
    let base_hostname = env::var("BASE_HOSTNAME")?;
    let bucket = env::var("BUCKET")?;
    let scheme = env::var("SCHEME").unwrap_or_else(|_| "http".to_string());

    /* set S3 request params */
    let host = region.clone() + "." + base_hostname.as_str();
    let upload_url = scheme + "://" + host.as_str();
    let parsed_url = upload_url.parse()?;
    let bucket = Bucket::new(parsed_url, UrlStyle::Path, bucket, region)?;

    let creds = Credentials::new(access_key, secret_key);
    let action = bucket.put_object(Some(&creds), fname);
    let signed_url = action.sign(Duration::from_secs(60 * 60));

    Ok((signed_url, host))
}
