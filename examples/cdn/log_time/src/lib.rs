use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::time::SystemTime;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpHeadersRoot) });
}}

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}

impl RootContext for HttpHeadersRoot {
    fn create_http_context(&self, context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpHeaders { context_id }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct HttpHeaders {
    context_id: u32,
}

impl Context for HttpHeaders {}

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let time = self.get_current_time();
        info!(
            "on_http_request_headers: {}",
            time.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                / 3600
        );
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        let time = self.get_current_time();
        info!(
            "on_http_response_headers: {}",
            time.duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs()
                / 3600
        );
        Action::Continue
    }

    fn on_log(&mut self) {
        info!("#{} completed.", self.context_id);
    }
}
