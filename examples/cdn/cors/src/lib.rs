/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating CORS header management.

Handles preflight OPTIONS requests and adds CORS response headers
for allowed origins. Supports configurable origin allow-lists,
methods, and exposed headers.

Configuration:
  - Environment variable: ALLOWED_ORIGINS (comma-separated origins or "*")
    When unset or empty the filter is dormant — requests pass through
    without CORS headers, so browsers will block cross-origin access.
  - Environment variable: ALLOWED_METHODS (default: "GET, POST, PUT, DELETE, OPTIONS")
  - Environment variable: MAX_AGE (default: "86400")
  - Environment variable: EXPOSE_HEADERS (response headers to expose)
*/

use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::env;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(CorsRoot) });
}}

struct CorsRoot;

impl Context for CorsRoot {}

impl RootContext for CorsRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(CorsContext))
    }
}

struct CorsContext;

impl Context for CorsContext {}

impl HttpContext for CorsContext {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let origin = match self.get_http_request_header("Origin") {
            Some(o) if !o.is_empty() => o,
            _ => return Action::Continue,
        };

        let allowed_origins = env::var("ALLOWED_ORIGINS").unwrap_or_default();
        if !is_origin_allowed(&origin, &allowed_origins) {
            return Action::Continue;
        }

        // Handle preflight OPTIONS request
        let method = self
            .get_http_request_header(":method")
            .unwrap_or_default();

        if method == "OPTIONS" {
            let allow_methods = env::var("ALLOWED_METHODS")
                .unwrap_or_else(|_| "GET, POST, PUT, DELETE, OPTIONS".to_string());
            let allow_headers = self
                .get_http_request_header("Access-Control-Request-Headers")
                .unwrap_or_else(|| "Content-Type, Authorization".to_string());
            let max_age = env::var("MAX_AGE").unwrap_or_else(|_| "86400".to_string());

            let effective_origin = if allowed_origins == "*" {
                "*".to_string()
            } else {
                origin
            };

            let mut headers = vec![
                ("Access-Control-Allow-Origin", effective_origin.as_str()),
                ("Access-Control-Allow-Methods", allow_methods.as_str()),
                ("Access-Control-Allow-Headers", allow_headers.as_str()),
                ("Access-Control-Max-Age", max_age.as_str()),
                ("Content-Length", "0"),
            ];

            // Vary: Origin is needed when the response varies by origin,
            // so shared caches don't serve a cached response for a different origin.
            // Not needed when Allow-Origin is "*" (response is the same for all origins).
            if effective_origin != "*" {
                headers.push(("Vary", "Origin"));
            }

            self.send_http_response(204, headers, None);

            return Action::Pause;
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        let origin = match self.get_http_request_header("Origin") {
            Some(o) if !o.is_empty() => o,
            _ => return Action::Continue,
        };

        let allowed_origins = env::var("ALLOWED_ORIGINS").unwrap_or_default();
        if !is_origin_allowed(&origin, &allowed_origins) {
            return Action::Continue;
        }

        let effective_origin = if allowed_origins == "*" {
            "*".to_string()
        } else {
            origin
        };

        self.add_http_response_header("Access-Control-Allow-Origin", &effective_origin);
        if effective_origin != "*" {
            self.add_http_response_header("Vary", "Origin");
        }

        if let Ok(expose) = env::var("EXPOSE_HEADERS") {
            if !expose.is_empty() {
                self.add_http_response_header("Access-Control-Expose-Headers", &expose);
            }
        }

        Action::Continue
    }
}

fn is_origin_allowed(origin: &str, allowed: &str) -> bool {
    if allowed.is_empty() {
        return false;
    }
    if allowed == "*" {
        return true;
    }
    allowed.split(',').any(|o| o.trim() == origin)
}
