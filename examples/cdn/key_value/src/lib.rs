/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating KV Store operations via the proxy-wasm interface.

Supports all KV Store operations via query parameters:
  ?store=<name>&action=get&key=<key>
  ?store=<name>&action=scan&match=<pattern>
  ?store=<name>&action=zrange&key=<key>&min=<f64>&max=<f64>
  ?store=<name>&action=zscan&key=<key>&match=<pattern>
  ?store=<name>&action=bfExists&key=<key>&item=<item>

Defaults to action=get if not specified.
*/

use fastedge::proxywasm::key_value::Store;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::collections::HashMap;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(KvStoreRoot) });
}}

struct KvStoreRoot;

impl Context for KvStoreRoot {}

impl RootContext for KvStoreRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(KvStoreContext))
    }
}

struct KvStoreContext;

impl Context for KvStoreContext {}

impl HttpContext for KvStoreContext {
    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        // Remove content-length since we replace the body
        self.set_http_response_header("content-length", None);
        self.set_http_response_header("content-type", Some("application/json"));
        self.set_http_response_header("transfer-encoding", Some("chunked"));
        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::StopIterationAndBuffer;
        }

        let query = self
            .get_property(vec!["request", "query"])
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default();

        if query.is_empty() {
            self.send_error("App must be called with query parameters", body_size);
            return Action::Continue;
        }

        let params: HashMap<&str, &str> = querystring::querify(&query).into_iter().collect();

        let Some(store_name) = params.get("store") else {
            self.send_error("Missing required param 'store'", body_size);
            return Action::Continue;
        };

        let action = params.get("action").copied().unwrap_or("get");

        let store = match Store::open(store_name) {
            Ok(s) => s,
            Err(e) => {
                self.send_error(&format!("Failed to open KvStore '{}': {}", store_name, e), body_size);
                return Action::Continue;
            }
        };

        let result = match action {
            "get" => self.handle_get(&store, &params),
            "scan" => self.handle_scan(&store, &params),
            "zrange" => self.handle_zrange(&store, &params),
            "zscan" => self.handle_zscan(&store, &params),
            "bfExists" => self.handle_bf_exists(&store, &params),
            _ => Err(format!(
                "Invalid action '{}'. Supported: get, scan, zrange, zscan, bfExists",
                action
            )),
        };

        let body = match result {
            Ok(json) => json,
            Err(msg) => {
                self.send_error(&msg, body_size);
                return Action::Continue;
            }
        };

        if let Err(e) = self.set_http_response_body(0, body_size, body.as_bytes()) {
            proxy_wasm::hostcalls::log(LogLevel::Error, &format!("failed to set body: {:?}", e)).ok();
        }

        Action::Continue
    }
}

impl KvStoreContext {
    fn handle_get(&self, store: &Store, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'get' action")?;
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
            Err(e) => Err(format!("KV get error: {}", e)),
        }
    }

    fn handle_scan(&self, store: &Store, params: &HashMap<&str, &str>) -> Result<String, String> {
        let pattern = *params.get("match").ok_or("Missing required param 'match' for 'scan' action")?;
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
            Err(e) => Err(format!("KV scan error: {}", e)),
        }
    }

    fn handle_zrange(&self, store: &Store, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'zrange' action")?;
        let min: f64 = params
            .get("min")
            .ok_or("Missing required param 'min' for 'zrange' action")?
            .parse()
            .map_err(|_| "Invalid 'min' value: must be a number".to_string())?;
        let max: f64 = params
            .get("max")
            .ok_or("Missing required param 'max' for 'zrange' action")?
            .parse()
            .map_err(|_| "Invalid 'max' value: must be a number".to_string())?;

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
            Err(e) => Err(format!("KV zrange error: {}", e)),
        }
    }

    fn handle_zscan(&self, store: &Store, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'zscan' action")?;
        let pattern = *params.get("match").ok_or("Missing required param 'match' for 'zscan' action")?;

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
            Err(e) => Err(format!("KV zscan error: {}", e)),
        }
    }

    fn handle_bf_exists(&self, store: &Store, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'bfExists' action")?;
        let item = *params.get("item").ok_or("Missing required param 'item' for 'bfExists' action")?;

        match store.bf_exists(key, item) {
            Ok(exists) => Ok(format!(
                r#"{{"store":"{}","action":"bfExists","key":"{}","item":"{}","response":{}}}"#,
                params.get("store").unwrap_or(&""),
                key,
                item,
                exists
            )),
            Err(e) => Err(format!("KV bfExists error: {}", e)),
        }
    }

    fn send_error(&self, msg: &str, body_size: usize) {
        proxy_wasm::hostcalls::log(LogLevel::Error, msg).ok();
        self.set_property(
            vec!["response", "status"],
            Some(b"500"),
        );
        let error_body = format!(r#"{{"error":"{}"}}"#, msg);
        self.set_http_response_body(0, body_size, error_body.as_bytes()).ok();
    }
}
