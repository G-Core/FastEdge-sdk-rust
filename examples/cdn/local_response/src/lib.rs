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
        self.send_http_response(
            251,
            vec![("Powered-By", "proxy-wasm"), ("Key", "Value")], // Headers
            Some(b"on_http_request_headers response from proxywasm local response example"),
        );
        Action::Pause
    }

    fn on_http_request_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        self.send_http_response(
            253,
            vec![("Powered-By", "proxy-wasm"), ("Key", "Value")], // Headers
            Some(b"on_http_request_body response from proxywasm local response example"),
        );
        Action::Pause
    }

    fn on_http_response_headers(&mut self, _num_headers: usize, _end_of_stream: bool) -> Action {
        self.send_http_response(
            252,
            vec![("Powered-By", "proxy-wasm"), ("Key", "Value")], // Headers
            Some(b"on_http_response_headers response from proxywasm local response example"),
        );
        Action::Pause
    }

    fn on_http_response_body(&mut self, _body_size: usize, _end_of_stream: bool) -> Action {
        self.send_http_response(
            254,
            vec![("Powered-By", "proxy-wasm"), ("Key", "Value")], // Headers
            Some(b"on_http_response_body response from proxywasm local response example"),
        );
        Action::Pause
    }
}
