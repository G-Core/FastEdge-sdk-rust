use std::io::Write;
use anyhow::{anyhow, Error, Result};

use fastedge::body::Body;
use fastedge::http::{Request, Response, StatusCode};
use fastedge::key_value::{Store, Error as StoreError};

#[allow(dead_code)]
#[fastedge::http]
fn main(req: Request<Body>) -> Result<Response<Body>> {
    let query = req.uri().query().ok_or(anyhow!("no query parameters"))?;
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
    let keys = params
        .iter()
        .filter_map(|(k, v)| {
            if "key".eq_ignore_ascii_case(k) {
                Some(*v)
            } else {
                None
            }
        })
        .collect::<Vec<&str>>();

    if keys.is_empty() {
        return Response::builder()
            .status(StatusCode::BAD_REQUEST)
            .body("missing param 'key'".into())
            .map_err(Error::msg);
    };

    let store = match Store::open(store) {
        Ok(store) => store,
        Err(StoreError::AccessDenied) => {
            return Response::builder()
                .status(StatusCode::FORBIDDEN)
                .body("access denied".into())
                .map_err(Error::msg);
        }
        Err(error) => {
            println!("store open error: {:?}", error);
            return Response::builder()
                .status(StatusCode::BAD_REQUEST)
                .body("store open error".into())
                .map_err(Error::msg);
        }
    };

    let mut body = Vec::new();
    body.write_all(b"get()\n")?;
    for key in keys {
        match store.get(key) {
            Ok(Some(value)) => {
                body.write_all(format!("{}=", key).as_bytes())?;
                body.extend(value);
                body.write_all(b"\n")?;
            }
            Ok(None) => {
                body.write_all(format!("{}=NOT_FOUND\n", key).as_bytes())?;
            }
            Err(error) => {
                return Response::builder()
                    .status(StatusCode::INTERNAL_SERVER_ERROR)
                    .body(error.to_string().into())
                    .map_err(Error::msg);
            }
        }
    }

    body.write_all(b"get_by_range()\n")?;
    match store.get_by_range("myset", 0, 100) {
        Ok(values) => {
            for value in values {
                body.write_all(b"get_by_range=")?;
                body.extend(value);
                body.write_all(b"\n")?;
            }
        }
        Err(error) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string().into())
                .map_err(Error::msg);
        }
    }

    body.write_all(b"get_keys()\n")?;
    match store.get_keys("my*") {
        Ok(keys) => {
            for key in keys {
                body.write_all(format!("get_keys={}\n", key).as_bytes())?;
            }
        }
        Err(error) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string().into())
                .map_err(Error::msg);
        }
    }

    body.write_all(b"get_by_prefix()\n")?;
    match store.get_by_prefix("myset", "*") {
        Ok(values) => {
            for value in values {
                body.write_all(b"get_by_prefix=")?;
                body.extend(value);
                body.write_all(b"\n")?;
            }
        }
        Err(error) => {
            return Response::builder()
                .status(StatusCode::INTERNAL_SERVER_ERROR)
                .body(error.to_string().into())
                .map_err(Error::msg);
        }
    }


    let res = Response::builder()
        .status(StatusCode::OK)
        .body(body.into())?;
    Ok(res)
}
