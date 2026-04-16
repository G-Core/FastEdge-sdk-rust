/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating API key validation.

Validates requests using an X-API-Key header checked against a stored
secret. Simpler alternative to JWT when token expiry and claims are
not needed.

Required configuration:
  - Secret: API_KEY
*/

use fastedge::proxywasm::secret;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(ApiKeyRoot) });
}}

struct ApiKeyRoot;

impl Context for ApiKeyRoot {}

impl RootContext for ApiKeyRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(ApiKeyContext))
    }
}

struct ApiKeyContext;

impl Context for ApiKeyContext {}

impl HttpContext for ApiKeyContext {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let expected_key = match secret::get("API_KEY") {
            Ok(Some(bytes)) => match String::from_utf8(bytes) {
                Ok(s) if !s.is_empty() => s,
                _ => {
                    self.send_http_response(500, vec![], Some(b"App misconfigured"));
                    return Action::Pause;
                }
            },
            _ => {
                self.send_http_response(500, vec![], Some(b"App misconfigured"));
                return Action::Pause;
            }
        };

        let provided_key = match self.get_http_request_header("X-API-Key") {
            Some(k) if !k.is_empty() => k,
            _ => {
                self.send_http_response(
                    401,
                    vec![("WWW-Authenticate", "API-Key")],
                    Some(b"Missing X-API-Key header"),
                );
                return Action::Pause;
            }
        };

        if provided_key != expected_key {
            proxy_wasm::hostcalls::log(LogLevel::Info, "API key validation failed").ok();
            self.send_http_response(403, vec![], Some(b"Invalid API key"));
            return Action::Pause;
        }

        // Strip the API key header before forwarding to upstream
        self.set_http_request_header("X-API-Key", None);

        proxy_wasm::hostcalls::log(LogLevel::Info, "API key validated successfully").ok();
        Action::Continue
    }
}
