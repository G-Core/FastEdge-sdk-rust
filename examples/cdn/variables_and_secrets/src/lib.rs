/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating environment variables and secrets access.

Reads USERNAME from the dictionary (env vars) and PASSWORD from secrets,
then forwards both as request headers to the upstream origin.

Required configuration:
  - Environment variable: USERNAME
  - Secret: PASSWORD
*/

use fastedge::proxywasm::dictionary;
use fastedge::proxywasm::secret;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(VariablesRoot) });
}}

struct VariablesRoot;

impl Context for VariablesRoot {}

impl RootContext for VariablesRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(VariablesContext))
    }
}

struct VariablesContext;

impl Context for VariablesContext {}

impl HttpContext for VariablesContext {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let username = dictionary::get("USERNAME").unwrap_or_default();
        let password = secret::get("PASSWORD")
            .ok()
            .flatten()
            .and_then(|v| String::from_utf8(v).ok())
            .unwrap_or_default();

        proxy_wasm::hostcalls::log(LogLevel::Info, &format!("USERNAME: {}", username)).ok();
        proxy_wasm::hostcalls::log(LogLevel::Info, &format!("PASSWORD: {}", password)).ok();

        self.add_http_request_header("x-env-username", &username);
        self.add_http_request_header("x-env-password", &password);

        Action::Continue
    }
}
