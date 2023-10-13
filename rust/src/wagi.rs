use std::error::Error;
use std::io::{Read, Write};
use std::{env, io};

use http::{Method, Request, Response, Uri};

use crate::body::Body;

pub fn request() -> Result<Request<Body>, Box<dyn Error>> {
    let uri = env::var("X_FULL_URL")?.parse::<Uri>()?;
    let method = env::var("REQUEST_METHOD")?.parse::<Method>()?;
    let builder = Request::builder().method(method).uri(uri);
    let builder = env::vars().fold(builder, |builder, (k, v)| builder.header(k, v));
    let mut body = vec![];
    io::stdin().read(&mut body).expect("read body");
    Ok(builder.body(Body::from(body))?)
}

pub fn response(res: Response<Body>) {
    let mut content_type = false;
    for (k, v) in res.headers() {
        if let Ok(value) = v.to_str() {
            let key = k.as_str().to_uppercase();
            match key.as_str() {
                "CONTENT-TYPE" => {
                    content_type = true;
                    eprintln!("CONTENT-TYPE:{}", value)
                }
                "LOCATION" => {
                    content_type = true;
                    eprintln!("LOCATION:{}", value)
                }
                _ => eprintln!("{}:{}", key, value),
            }
        }
    }
    if !content_type {
        eprintln!("CONTENT-TYPE:{}", res.body().content_type)
    }
    eprint!("\r\n\r\n");
    io::stderr().write(res.body()).expect("write body");
    io::stdout().flush().expect("flush body");
}
