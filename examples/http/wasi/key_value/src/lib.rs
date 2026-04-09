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
use serde_json::json;
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
                .header("content-type", "application/json")
                .body(Body::from(json!({"error": "access denied"}).to_string()))?);
        }
        Err(e) => {
            return Ok(Response::builder()
                .status(500)
                .header("content-type", "application/json")
                .body(Body::from(json!({"error": format!("store open error: {e}")}).to_string()))?);
        }
    };

    let body = match action {
        "get" => handle_get(&store, &params)?,
        "scan" => handle_scan(&store, &params)?,
        "zrange" => handle_zrange(&store, &params)?,
        "zscan" => handle_zscan(&store, &params)?,
        "bfExists" => handle_bf_exists(&store, &params)?,
        _ => {
            return Ok(Response::builder()
                .status(400)
                .header("content-type", "application/json")
                .body(Body::from(json!({"error": format!("Invalid action '{action}'. Supported: get, scan, zrange, zscan, bfExists")}).to_string()))?);
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
            Ok(json!({
                "store": params.get("store").unwrap_or(&""),
                "action": "get",
                "key": key,
                "response": value_str.as_ref()
            }).to_string())
        }
        Ok(None) => Ok(json!({
            "store": params.get("store").unwrap_or(&""),
            "action": "get",
            "key": key,
            "response": null
        }).to_string()),
        Err(e) => Err(anyhow!("KV get error: {e}")),
    }
}

fn handle_scan(store: &Store, params: &HashMap<&str, &str>) -> anyhow::Result<String> {
    let pattern = *params
        .get("match")
        .ok_or(anyhow!("missing param 'match'"))?;
    match store.scan(pattern) {
        Ok(keys) => Ok(json!({
            "store": params.get("store").unwrap_or(&""),
            "action": "scan",
            "match": pattern,
            "response": keys
        }).to_string()),
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
            let entries_json: Vec<serde_json::Value> = entries
                .iter()
                .map(|(value, score)| {
                    let value_str = String::from_utf8_lossy(value);
                    json!({"value": value_str.as_ref(), "score": score})
                })
                .collect();
            Ok(json!({
                "store": params.get("store").unwrap_or(&""),
                "action": "zrange",
                "key": key,
                "min": min,
                "max": max,
                "response": entries_json
            }).to_string())
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
            let entries_json: Vec<serde_json::Value> = entries
                .iter()
                .map(|(value, score)| {
                    let value_str = String::from_utf8_lossy(value);
                    json!({"value": value_str.as_ref(), "score": score})
                })
                .collect();
            Ok(json!({
                "store": params.get("store").unwrap_or(&""),
                "action": "zscan",
                "key": key,
                "match": pattern,
                "response": entries_json
            }).to_string())
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
        Ok(exists) => Ok(json!({
            "store": params.get("store").unwrap_or(&""),
            "action": "bfExists",
            "key": key,
            "item": item,
            "response": exists
        }).to_string()),
        Err(e) => Err(anyhow!("KV bfExists error: {e}")),
    }
}
