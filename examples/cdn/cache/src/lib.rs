/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating cache operations via the proxy-wasm interface.

Unlike the KV store, the cache has no named stores or handles — every operation is
scoped to the calling application and addressed by key alone.

Supports all cache operations via query parameters:
  ?action=get&key=<key>
  ?action=set&key=<key>&value=<value>[&ttl=<ms>]
  ?action=delete&key=<key>
  ?action=exists&key=<key>
  ?action=incr&key=<key>&delta=<i64>
  ?action=expire&key=<key>&ttl=<ms>
  ?action=purge
  ?action=purgePrefix&prefix=<prefix>

Defaults to action=get if not specified. Omitting `ttl` on `set` stores the value
with no expiry.
*/

use fastedge::proxywasm::cache;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde_json::json;
use std::collections::HashMap;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(CacheRoot) });
}}

struct CacheRoot;

impl Context for CacheRoot {}

impl RootContext for CacheRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(CacheContext))
    }
}

struct CacheContext;

impl Context for CacheContext {}

impl HttpContext for CacheContext {
    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        // Remove content-length since we replace the body
        self.set_http_response_header("content-length", None);
        self.set_http_response_header("content-type", Some("application/json"));
        self.set_http_response_header("transfer-encoding", Some("chunked"));
        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
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

        let action = params.get("action").copied().unwrap_or("get");

        let result = match action {
            "get" => self.handle_get(&params),
            "set" => self.handle_set(&params),
            "delete" => self.handle_delete(&params),
            "exists" => self.handle_exists(&params),
            "incr" => self.handle_incr(&params),
            "expire" => self.handle_expire(&params),
            "purge" => self.handle_purge(),
            "purgePrefix" => self.handle_purge_prefix(&params),
            _ => Err(format!(
                "Invalid action '{}'. Supported: get, set, delete, exists, incr, expire, purge, purgePrefix",
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

        self.set_http_response_body(0, body_size, body.as_bytes());

        Action::Continue
    }
}

impl CacheContext {
    fn handle_get(&self, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'get' action")?;
        match cache::get(key) {
            Ok(Some(value)) => {
                let value_str = String::from_utf8_lossy(&value);
                Ok(json!({
                    "action": "get",
                    "key": key,
                    "response": value_str.as_ref()
                }).to_string())
            }
            Ok(None) => Ok(json!({
                "action": "get",
                "key": key,
                "response": null
            }).to_string()),
            Err(e) => Err(format!("Cache get error: {}", e)),
        }
    }

    fn handle_set(&self, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'set' action")?;
        let value = *params.get("value").ok_or("Missing required param 'value' for 'set' action")?;
        // no 'ttl' param means no expiry
        let ttl_ms = match params.get("ttl") {
            Some(ttl) => Some(
                ttl.parse::<u64>()
                    .map_err(|_| "Invalid 'ttl' value: must be a positive number of milliseconds".to_string())?,
            ),
            None => None,
        };

        match cache::set(key, value.as_bytes(), ttl_ms) {
            Ok(()) => Ok(json!({
                "action": "set",
                "key": key,
                "value": value,
                "ttlMs": ttl_ms,
                "response": true
            }).to_string()),
            Err(e) => Err(format!("Cache set error: {}", e)),
        }
    }

    fn handle_delete(&self, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'delete' action")?;
        match cache::delete(key) {
            Ok(()) => Ok(json!({
                "action": "delete",
                "key": key,
                "response": true
            }).to_string()),
            Err(e) => Err(format!("Cache delete error: {}", e)),
        }
    }

    fn handle_exists(&self, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'exists' action")?;
        match cache::exists(key) {
            Ok(exists) => Ok(json!({
                "action": "exists",
                "key": key,
                "response": exists
            }).to_string()),
            Err(e) => Err(format!("Cache exists error: {}", e)),
        }
    }

    fn handle_incr(&self, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'incr' action")?;
        let delta: i64 = params
            .get("delta")
            .ok_or("Missing required param 'delta' for 'incr' action")?
            .parse()
            .map_err(|_| "Invalid 'delta' value: must be an integer".to_string())?;

        match cache::incr(key, delta) {
            Ok(value) => Ok(json!({
                "action": "incr",
                "key": key,
                "delta": delta,
                "response": value
            }).to_string()),
            Err(e) => Err(format!("Cache incr error: {}", e)),
        }
    }

    fn handle_expire(&self, params: &HashMap<&str, &str>) -> Result<String, String> {
        let key = *params.get("key").ok_or("Missing required param 'key' for 'expire' action")?;
        let ttl_ms: u64 = params
            .get("ttl")
            .ok_or("Missing required param 'ttl' for 'expire' action")?
            .parse()
            .map_err(|_| "Invalid 'ttl' value: must be a positive number of milliseconds".to_string())?;

        match cache::expire(key, ttl_ms) {
            Ok(updated) => Ok(json!({
                "action": "expire",
                "key": key,
                "ttlMs": ttl_ms,
                "response": updated
            }).to_string()),
            Err(e) => Err(format!("Cache expire error: {}", e)),
        }
    }

    fn handle_purge(&self) -> Result<String, String> {
        match cache::purge() {
            Ok(deleted) => Ok(json!({
                "action": "purge",
                "response": deleted
            }).to_string()),
            Err(e) => Err(format!("Cache purge error: {}", e)),
        }
    }

    fn handle_purge_prefix(&self, params: &HashMap<&str, &str>) -> Result<String, String> {
        let prefix = *params
            .get("prefix")
            .ok_or("Missing required param 'prefix' for 'purgePrefix' action")?;

        match cache::purge_prefix(prefix) {
            Ok(deleted) => Ok(json!({
                "action": "purgePrefix",
                "prefix": prefix,
                "response": deleted
            }).to_string()),
            Err(e) => Err(format!("Cache purgePrefix error: {}", e)),
        }
    }

    fn send_error(&self, msg: &str, body_size: usize) {
        println!("{}", msg);
        self.set_property(
            vec!["response", "status"],
            Some(b"500"),
        );
        let error_body = json!({"error": msg}).to_string();
        self.set_http_response_body(0, body_size, error_body.as_bytes());
    }
}
