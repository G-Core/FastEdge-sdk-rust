use anyhow::{anyhow, Error, Result};

use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};
use fastedge::key_value::Store;

#[allow(dead_code)]
#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let query = req
        .uri()
        .query()
        .ok_or(anyhow!("no query parameters"))?;
    let params = querystring::querify(query);
    let Some(store) = params.iter().find_map(|(k, v)| {
        if "store".eq_ignore_ascii_case(k) {
            Some(v)
        } else {
            None
        }
    }) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("missing param 'store'".into())
            .map_err(Error::msg);
    };
    let keys = params.iter().filter_map(|(k, v)| {
        if "key".eq_ignore_ascii_case(k) {
            Some(*v)
        } else {
            None
        }
    }).collect::<Vec<&str>>();

    if keys.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("missing param 'key'".into())
            .map_err(Error::msg);
    };

    let Ok(store) = Store::open(store) else {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("store open error".into())
            .map_err(Error::msg);
    };
    let mut body = Vec::new();
    for key in keys {
        let Ok(value) = store.get(key) else {
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("store get error".into())
                .map_err(Error::msg);
        };
        let Some(value) = value else {
            return Response::builder()
                .status(StatusCode::NOT_FOUND)
                .body(Body::empty())
                .map_err(Error::msg);
        };
        body.extend(value);
    }

    let res = Response::builder()
        .status(StatusCode::OK)
        .body(body.into())?;
    Ok(res)
}
