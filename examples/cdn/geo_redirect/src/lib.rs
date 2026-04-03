/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating geo-based origin routing.

Routes requests to country-specific origins based on the request's
geo-IP country code. Falls back to a DEFAULT origin when no
country-specific mapping is configured.

Required configuration:
  - Environment variable: DEFAULT (fallback origin URL)
  - Environment variable: <COUNTRY_CODE> (optional per-country origin URLs, e.g. US, DE, GB)
*/

use proxy_wasm::traits::*;
use std::env;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(GeoRedirectRoot) });
}}

struct GeoRedirectRoot;

impl Context for GeoRedirectRoot {}

impl RootContext for GeoRedirectRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(GeoRedirectContext))
    }
}

struct GeoRedirectContext;

impl Context for GeoRedirectContext {}

impl HttpContext for GeoRedirectContext {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let Ok(default_origin) = env::var("DEFAULT") else {
            self.send_http_response(500, vec![], Some(b"App misconfigured - DEFAULT must be set"));
            return Action::Pause;
        };

        let country_code = self
            .get_property(vec!["request.country"])
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_default();

        if country_code.is_empty() {
            self.send_http_response(502, vec![], Some(b"Missing country information"));
            return Action::Pause;
        }

        let origin = env::var(&country_code).unwrap_or(default_origin);
        let origin = origin.trim_end_matches('/');

        let path = self
            .get_property(vec!["request.path"])
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_else(|| "/".to_string());

        // Preserve the Host header if present
        if let Some(host) = self
            .get_property(vec!["request.host"])
            .and_then(|bytes| String::from_utf8(bytes).ok())
        {
            self.set_http_request_header("Host", Some(&host));
        }

        let request_url = format!("{}{}", origin, path);

        proxy_wasm::hostcalls::log(LogLevel::Info, &format!("Redirecting to: {}", request_url)).ok();

        self.set_property(vec!["request.url"], Some(request_url.as_bytes()));

        Action::Continue
    }
}
