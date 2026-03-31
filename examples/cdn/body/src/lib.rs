use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpBodyRoot) });
}}

struct HttpBodyRoot;

impl Context for HttpBodyRoot {}

impl RootContext for HttpBodyRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpBody))
    }
}

struct HttpBody;

impl Context for HttpBody {}

impl HttpContext for HttpBody {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        self.set_http_request_header("content-length", None);
        Action::Continue
    }

    fn on_http_request_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            // Wait -- we'll be called again when the complete body is buffered
            // at the host side.
            return Action::Pause;
        }

        if let Some(body_bytes) = self.get_http_request_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            if body_str.contains("Client") {
                let new_body = format!("Original message body ({body_size} bytes) redacted.\n");
                self.set_http_request_body(0, body_size, &new_body.into_bytes());
            }
        }
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        // remove content-length as we plan to change the body size
        self.set_http_response_header("content-length", None);
        // set transfer-encoding to chunked as we don't know body length
        self.set_http_response_header("transfer-encoding", Some("Chunked"));

        if let Some(content_type) = self.get_http_response_header("content-type") {
            self.set_property(vec!["response.content_type"], Some(content_type.as_bytes()));
        }

        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }

        let url = if let Some(value) = self.get_property(vec!["request.url"]) {
            let url = String::from_utf8_lossy(&value);
            info!("url={}", url);
            url.to_string()
        } else {
            "".to_string()
        };

        let content_type =
            if let Some(content_type) = self.get_property(vec!["response.content_type"]) {
                let content_type = String::from_utf8_lossy(&content_type);
                info!("content_type={}", content_type);
                content_type.to_string()
            } else {
                "NONE".to_string()
            };

        if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
            let body_str = String::from_utf8(body_bytes).unwrap();
            if body_str.contains("Client") {
                let new_body =
                    format!("Original message body ({body_size} bytes) redacted.\nURL: {url}\nContent-Type: {content_type}\n");
                self.set_http_response_body(0, body_size, &new_body.into_bytes());
            }
        }
        Action::Continue
    }
}
