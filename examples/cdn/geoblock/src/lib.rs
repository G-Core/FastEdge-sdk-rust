use std::env;
use std::time::{SystemTime, UNIX_EPOCH};

use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpHeadersRoot) });
}}

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}

impl RootContext for HttpHeadersRoot {
    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpHeaders {}))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct HttpHeaders {}

impl Context for HttpHeaders {}

const BAD_GATEWAY: u32 = 502;
const FORBIDDEN: u32 = 403;
const INTERNAL_SERVER_ERROR: u32 = 500;

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let Ok(blacklist) = env::var("BLACKLIST") else {
            self.send_http_response(INTERNAL_SERVER_ERROR, vec![], Some(b"App misconfigured"));
            return Action::Pause;
        };

        let mut blacklist = blacklist.split(',');

        let Some(country) = self.get_property(vec!["request.country"]) else {
            self.send_http_response(BAD_GATEWAY, vec![], Some(b"Malformed request - no country field"));
            return Action::Pause;
        };

        let Ok(country) = std::str::from_utf8(&country) else {
            self.send_http_response(BAD_GATEWAY, vec![], Some(b"Malformed request - country not utf8 string"));
            return Action::Pause;
        };

        if blacklist.any(|b| country.eq_ignore_ascii_case(b)) {
            let tw_start = env::var("BLACKLIST_TW_START").ok();
            let tw_end = env::var("BLACKLIST_TW_END").ok();

            if let Some((tw_start, tw_end)) = tw_start.zip(tw_end) {
                let Ok(tw_start) = tw_start.parse::<u64>() else {
                    self.send_http_response(INTERNAL_SERVER_ERROR, vec![], Some(b"App misconfigured"));
                    return Action::Pause;
                };

                let Ok(tw_end) = tw_end.parse::<u64>() else {
                    self.send_http_response(INTERNAL_SERVER_ERROR, vec![], Some(b"App misconfigured"));
                    return Action::Pause;
                };
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                if now > tw_start || now <= tw_end {
                    self.send_http_response(FORBIDDEN, vec![], Some(b"Request blacklisted"));
                    return Action::Pause;
                }
            } else {
                self.send_http_response(FORBIDDEN, vec![], Some(b"Request blacklisted"));
                return Action::Pause;
            }
        }


        Action::Continue
    }
}
