/*
* Copyright 2025 G-Core Innovations SARL
*/
/*
Example CDN app demonstrating A/B traffic splitting at the CDN layer.

Uses a cookie to assign users to variant A or B, then rewrites the
request path to route to different parts of the origin server.

Required configuration:
  - Environment variable: EXPERIMENT_NAME
  - Environment variable: VARIANT_A_PATH (path prefix for variant A)
  - Environment variable: VARIANT_B_PATH (path prefix for variant B)
*/

use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::env;
use std::time::UNIX_EPOCH;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Info);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(AbTestingRoot) });
}}

struct AbTestingRoot;

impl Context for AbTestingRoot {}

impl RootContext for AbTestingRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(AbTestingContext))
    }
}

struct AbTestingContext;

impl Context for AbTestingContext {}

impl HttpContext for AbTestingContext {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let Ok(experiment_name) = env::var("EXPERIMENT_NAME") else {
            self.send_http_response(
                500,
                vec![],
                Some(b"App misconfigured - EXPERIMENT_NAME must be set"),
            );
            return Action::Pause;
        };

        let Ok(variant_a_path) = env::var("VARIANT_A_PATH") else {
            self.send_http_response(
                500,
                vec![],
                Some(b"App misconfigured - VARIANT_A_PATH must be set"),
            );
            return Action::Pause;
        };

        let Ok(variant_b_path) = env::var("VARIANT_B_PATH") else {
            self.send_http_response(
                500,
                vec![],
                Some(b"App misconfigured - VARIANT_B_PATH must be set"),
            );
            return Action::Pause;
        };

        let cookie_name = format!("fe_exp_{}", experiment_name);

        // Check for existing experiment cookie
        let cookie_header = self
            .get_http_request_header("Cookie")
            .unwrap_or_default();
        let mut assigned = get_cookie_value(&cookie_header, &cookie_name);

        // Assign variant if not already set
        if assigned != "A" && assigned != "B" {
            let now = self
                .get_current_time()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis();
            assigned = if now % 2 == 0 { "A" } else { "B" }.to_string();
        }

        // Rewrite request path
        let path = self
            .get_property(vec!["request.path"])
            .and_then(|bytes| String::from_utf8(bytes).ok())
            .unwrap_or_else(|| "/".to_string());

        let variant_path = if assigned == "A" {
            &variant_a_path
        } else {
            &variant_b_path
        };
        let new_path = format!("{}{}", variant_path, path);

        // Update the request path directly to avoid ambiguous URL rewriting.
        self.set_property(vec!["request.path"], Some(new_path.as_bytes()));

        // Add variant headers for upstream visibility
        self.add_http_request_header("X-Experiment", &experiment_name);
        self.add_http_request_header("X-Variant", &assigned);

        proxy_wasm::hostcalls::log(
            LogLevel::Info,
            &format!(
                "A/B test \"{}\": variant {}, path {}",
                experiment_name, assigned, new_path
            ),
        )
        .ok();

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        // Recover the assigned variant and experiment name from the request headers set in
        // on_http_request_headers. Instance state does not survive the nginx -> core-proxy hop.
        let Some(variant) = self.get_http_request_header("X-Variant") else {
            return Action::Continue;
        };
        let Some(experiment_name) = self.get_http_request_header("X-Experiment") else {
            return Action::Continue;
        };

        let cookie = format!(
            "fe_exp_{}={}; Path=/; Max-Age=86400; SameSite=Lax",
            experiment_name, variant
        );
        self.add_http_response_header("Set-Cookie", &cookie);
        self.add_http_response_header("X-Variant", &variant);

        Action::Continue
    }
}

fn get_cookie_value(cookie_header: &str, name: &str) -> String {
    if cookie_header.is_empty() {
        return String::new();
    }
    let prefix = format!("{}=", name);
    for pair in cookie_header.split(';') {
        let pair = pair.trim();
        if let Some(value) = pair.strip_prefix(&prefix) {
            return value.to_string();
        }
    }
    String::new()
}
