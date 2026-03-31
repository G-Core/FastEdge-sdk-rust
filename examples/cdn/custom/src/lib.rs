use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::time::Duration;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpHeadersRoot) });
}}

const BAD_REQUEST: u32 = 400;

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}

impl RootContext for HttpHeadersRoot {
    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpHeaders))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct HttpHeaders;

impl Context for HttpHeaders {}

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let Some(path) = self.get_property(vec!["request.path"]) else {
            self.send_http_response(BAD_REQUEST, vec![], Some(b"Malformed request - no path"));
            return Action::Pause;
        };

        let Ok(path) = std::str::from_utf8(&path) else {
            self.send_http_response(
                BAD_REQUEST,
                vec![],
                Some(b"Malformed request - not utf8 string"),
            );
            return Action::Pause;
        };

        //trim first '/'
        let path = if path.starts_with('/') {
            &path[1..]
        } else {
            path
        };
        let mut segments = path.split('/');

        let Some(status_code) = segments.next() else {
            return Action::Continue;
        };

        if let Some(delay) = segments.next() {
            if let Ok(delay) = delay.parse::<u64>() {
                std::thread::sleep(Duration::from_millis(delay));
            }
        }

        let Ok(status_code) = status_code.parse::<u32>() else {
            self.send_http_response(
                BAD_REQUEST,
                vec![],
                Some(b"Malformed request - invalid status code"),
            );
            return Action::Pause;
        };

        match status_code {
            0 | 200 => Action::Continue,
            code if code < 600 => {
                self.send_http_response(code, vec![], None);
                Action::Pause
            }
            _ => {
                self.send_http_response(BAD_REQUEST, vec![], None);
                Action::Pause
            }
        }
    }
}
