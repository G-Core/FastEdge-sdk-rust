/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example app demonstrating KV Store operations via the WASI-HTTP interface.

Supports all KV Store operations via query parameters:
  ?store=<name>&action=get&key=<key>
  ?store=<name>&action=scan&match=<pattern>
  ?store=<name>&action=zrange&key=<key>&min=<f64>&max=<f64>
  ?store=<name>&action=zscan&key=<key>&match=<pattern>
  ?store=<name>&action=bfExists&key=<key>&item=<item>

Defaults to action=get if not specified.
*/

use std::collections::HashMap;

use anyhow::anyhow;
use fastedge::key_value::{Store, Error as StoreError};
use wstd::http::body::Body;
use wstd::http::{Request, Response};

#[wstd::http_server]
async fn main(req: Request<Body>) -> anyhow::Result<Response<Body>> {
    let query = req.uri().query().ok_or(anyhow!("no query parameters"))?;
    let params: HashMap<&str, &str> = querystring::querify(query).into_iter().collect();

    let store_name = *params
        .get("store")
        .ok_or(anyhow!("missing param 'store'"))?;

    let action = params.get("action").copied().unwrap_or("get");

    let store = match Store::open(store_name) {
        Ok(s) => s,
        Err(StoreError::AccessDenied) => {
            return Ok(Response::builder()
                .status(403)
                .body(Body::from("access denied"))?);
        }
        Err(e) => {
            return Ok(Response::builder()
                .status(500)
                .body(Body::from(format!("store open error: {e}")))?);
        }
    };

    let body = match action {
        "get" => handle_get(&store, &params)?,
        "scan" => handle_scan(&store, &params)?,
        "zrange" => handle_zrange(&store, &params)?,
        "zscan" => handle_zscan(&store, &params)?,
        "bfExists" => handle_bf_exists(&store, &params)?,
        _ => {
            return Ok(Response::builder().status(400).body(Body::from(format!(
                "Invalid action '{action}'. Supported: get, scan, zrange, zscan, bfExists"
            )))?);
        }
    };

    Ok(Response::builder()
        .status(200)
        .header("content-type", "application/json")
        .body(Body::from(body))?)
}

fn handle_get(store: &Store, params: &HashMap<&str, &str>) -> anyhow::Result<String> {
    let key = *params.get("key").ok_or(anyhow!("missing param 'key'"))?;
    match store.get(key) {
        Ok(Some(value)) => {
            let value_str = String::from_utf8_lossy(&value);
            Ok(format!(
                r#"{{"store":"{}","action":"get","key":"{}","response":"{}"}}"#,
                params.get("store").unwrap_or(&""),
                key,
                value_str
            ))
        }
        Ok(None) => Ok(format!(
            r#"{{"store":"{}","action":"get","key":"{}","response":null}}"#,
            params.get("store").unwrap_or(&""),
            key
        )),
        Err(e) => Err(anyhow!("KV get error: {e}")),
    }
}

fn handle_scan(store: &Store, params: &HashMap<&str, &str>) -> anyhow::Result<String> {
    let pattern = *params
        .get("match")
        .ok_or(anyhow!("missing param 'match'"))?;
    match store.scan(pattern) {
        Ok(keys) => {
            let keys_json: Vec<String> = keys.iter().map(|k| format!(r#""{}""#, k)).collect();
            Ok(format!(
                r#"{{"store":"{}","action":"scan","match":"{}","response":[{}]}}"#,
                params.get("store").unwrap_or(&""),
                pattern,
                keys_json.join(",")
            ))
        }
        Err(e) => Err(anyhow!("KV scan error: {e}")),
    }
}

fn handle_zrange(store: &Store, params: &HashMap<&str, &str>) -> anyhow::Result<String> {
    let key = *params.get("key").ok_or(anyhow!("missing param 'key'"))?;
    let min: f64 = params
        .get("min")
        .ok_or(anyhow!("missing param 'min'"))?
        .parse()
        .map_err(|_| anyhow!("invalid 'min': must be a number"))?;
    let max: f64 = params
        .get("max")
        .ok_or(anyhow!("missing param 'max'"))?
        .parse()
        .map_err(|_| anyhow!("invalid 'max': must be a number"))?;

    match store.zrange_by_score(key, min, max) {
        Ok(entries) => {
            let entries_json: Vec<String> = entries
                .iter()
                .map(|(value, score)| {
                    let value_str = String::from_utf8_lossy(value);
                    format!(r#"{{"value":"{}","score":{}}}"#, value_str, score)
                })
                .collect();
            Ok(format!(
                r#"{{"store":"{}","action":"zrange","key":"{}","min":{},"max":{},"response":[{}]}}"#,
                params.get("store").unwrap_or(&""),
                key,
                min,
                max,
                entries_json.join(",")
            ))
        }
        Err(e) => Err(anyhow!("KV zrange error: {e}")),
    }
}

fn handle_zscan(store: &Store, params: &HashMap<&str, &str>) -> anyhow::Result<String> {
    let key = *params.get("key").ok_or(anyhow!("missing param 'key'"))?;
    let pattern = *params
        .get("match")
        .ok_or(anyhow!("missing param 'match'"))?;

    match store.zscan(key, pattern) {
        Ok(entries) => {
            let entries_json: Vec<String> = entries
                .iter()
                .map(|(value, score)| {
                    let value_str = String::from_utf8_lossy(value);
                    format!(r#"{{"value":"{}","score":{}}}"#, value_str, score)
                })
                .collect();
            Ok(format!(
                r#"{{"store":"{}","action":"zscan","key":"{}","match":"{}","response":[{}]}}"#,
                params.get("store").unwrap_or(&""),
                key,
                pattern,
                entries_json.join(",")
            ))
        }
        Err(e) => Err(anyhow!("KV zscan error: {e}")),
    }
}

fn handle_bf_exists(store: &Store, params: &HashMap<&str, &str>) -> anyhow::Result<String> {
    let key = *params.get("key").ok_or(anyhow!("missing param 'key'"))?;
    let item = *params
        .get("item")
        .ok_or(anyhow!("missing param 'item'"))?;

    match store.bf_exists(key, item) {
        Ok(exists) => Ok(format!(
            r#"{{"store":"{}","action":"bfExists","key":"{}","item":"{}","response":{}}}"#,
            params.get("store").unwrap_or(&""),
            key,
            item,
            exists
        )),
        Err(e) => Err(anyhow!("KV bfExists error: {e}")),
    }
}
