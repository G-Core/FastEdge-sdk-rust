/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating environment variables and secrets access.

Reads USERNAME from environment variables and PASSWORD from secrets,
then forwards both as request headers to the upstream origin.

Required configuration:
  - Environment variable: USERNAME
  - Secret: PASSWORD
*/

use fastedge::proxywasm::secret;
use std::env;
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
        let username = env::var("USERNAME").unwrap_or_default();
        let password = secret::get("PASSWORD")
            .ok()
            .flatten()
            .and_then(|v| String::from_utf8(v).ok())
            .unwrap_or_default();

        println!("USERNAME: {}", username);
        // WARNING: Secrets are stored and retrieved as plaintext. Never log secret values
        // in production code — platform logs are visible to operators and may be persisted.
        // This line is shown for demonstration only; remove it in any real application.
        println!("PASSWORD: {}", password);

        self.add_http_request_header("x-env-username", &username);
        // WARNING: Forwarding a secret in a request header exposes it to the upstream origin
        // and any intermediary that can inspect headers. Only do this when the upstream
        // channel is trusted and the header is required by the destination API.
        self.add_http_request_header("x-env-password", &password);

        Action::Continue
    }
}
