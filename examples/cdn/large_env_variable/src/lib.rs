/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating access to large environment variables.

Uses `fastedge::proxywasm::dictionary` to read environment variables that
may exceed the 64KB WASI environment variable size limit.

For normal-sized environment variables (< 64KB), prefer `std::env::var()`
instead. The dictionary API is only required when your variable value
may be larger than 64KB.

Required configuration:
  - Environment variable: LARGE_CONFIG (a large configuration payload, e.g. JSON)
*/

use fastedge::proxywasm::dictionary;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(LargeEnvRoot) });
}}

struct LargeEnvRoot;

impl Context for LargeEnvRoot {}

impl RootContext for LargeEnvRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(LargeEnvContext))
    }
}

struct LargeEnvContext;

impl Context for LargeEnvContext {}

impl HttpContext for LargeEnvContext {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        // Use dictionary::get for environment variables that may exceed 64KB.
        // For normal-sized env vars, use std::env::var() instead.
        let config = dictionary::get("LARGE_CONFIG").unwrap_or_default();

        let size = config.len();
        proxy_wasm::hostcalls::log(
            LogLevel::Info,
            &format!("LARGE_CONFIG size: {} bytes", size),
        )
        .ok();

        self.add_http_request_header("x-config-size", &size.to_string());

        Action::Continue
    }
}
