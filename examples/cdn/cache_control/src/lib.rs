/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating content-type-aware cache control.

Sets Cache-Control response headers based on the content type and
response status, providing fine-grained control over CDN caching.

Optional configuration:
  - Environment variable: STATIC_MAX_AGE (default: 31536000)
  - Environment variable: HTML_MAX_AGE (default: 3600)
  - Environment variable: API_MAX_AGE (default: 0 = no-cache)
*/

use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::env;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(CacheControlRoot) });
}}

struct CacheControlRoot;

impl Context for CacheControlRoot {}

impl RootContext for CacheControlRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(CacheControlContext))
    }
}

struct CacheControlContext;

impl Context for CacheControlContext {}

impl HttpContext for CacheControlContext {
    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        // Read response status
        let status_code = self
            .get_property(vec!["response.status"])
            .and_then(|bytes| {
                if bytes.len() == 2 {
                    Some(u16::from_be_bytes([bytes[0], bytes[1]]))
                } else {
                    None
                }
            })
            .unwrap_or(200);

        // Error responses should never be cached
        if !(200..400).contains(&status_code) {
            self.set_http_response_header("Cache-Control", Some("no-store"));
            return Action::Continue;
        }

        let content_type = self
            .get_http_response_header("Content-Type")
            .unwrap_or_default();

        let static_max_age = env::var("STATIC_MAX_AGE").unwrap_or_else(|_| "31536000".to_string());
        let html_max_age = env::var("HTML_MAX_AGE").unwrap_or_else(|_| "3600".to_string());
        let api_max_age = env::var("API_MAX_AGE").unwrap_or_else(|_| "0".to_string());

        let cache_control = if is_static_asset(&content_type) {
            format!("public, max-age={}, immutable", static_max_age)
        } else if content_type.contains("text/html") {
            self.add_http_response_header("Vary", "Accept-Encoding");
            format!("public, max-age={}, must-revalidate", html_max_age)
        } else if content_type.contains("application/json")
            || content_type.contains("application/xml")
        {
            self.add_http_response_header("Vary", "Accept, Authorization");
            if api_max_age == "0" {
                "no-cache, no-store, must-revalidate".to_string()
            } else {
                format!("private, max-age={}, must-revalidate", api_max_age)
            }
        } else {
            "public, max-age=600".to_string()
        };

        self.set_http_response_header("Cache-Control", Some(&cache_control));

        proxy_wasm::hostcalls::log(
            LogLevel::Info,
            &format!(
                "Cache-Control: {} (content-type: {})",
                cache_control, content_type
            ),
        )
        .ok();

        Action::Continue
    }
}

fn is_static_asset(content_type: &str) -> bool {
    content_type.starts_with("image/")
        || content_type.starts_with("font/")
        || content_type.contains("application/javascript")
        || content_type.contains("text/css")
        || content_type.contains("text/javascript")
        || content_type.contains("application/wasm")
}
