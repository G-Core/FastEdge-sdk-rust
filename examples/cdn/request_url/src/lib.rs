use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

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

pub const REQUEST_URI: &str = "request.url";
pub const REQUEST_HOST: &str = "request.host";
pub const REQUEST_PATH: &str = "request.path";
pub const REQUEST_QUERY: &str = "request.query";

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {

        let Some(uri) = self.get_property(vec![REQUEST_URI]) else {
            self.send_http_response(551, vec![], None);
            return Action::Pause;
        };
        self.add_http_request_header_bytes("request-uri", &uri);

        let Some(host) = self.get_property(vec![REQUEST_HOST]) else {
            self.send_http_response(552, vec![], None);
            return Action::Pause;
        };
        self.add_http_request_header_bytes("request-host", &host);

        let Some(path) = self.get_property(vec![REQUEST_PATH]) else {
            self.send_http_response(553, vec![], None);
            return Action::Pause;
        };
        self.add_http_request_header_bytes("request-path", &path);

        let Some(query) = self.get_property(vec![REQUEST_QUERY]) else {
            self.send_http_response(554, vec![], None);
            return Action::Pause;
        };
        self.add_http_request_header_bytes("request-query", &query);

        if let Some(new_url) = self.get_http_request_header("set-url") {
            self.set_property(
                vec!["request.url"],
                Some(new_url.as_bytes()),
            );
        }

        if let Some(new_host) = self.get_http_request_header("set-host") {
            self.set_property(
                vec!["request.host"],
                Some(new_host.as_bytes()),
            );
        }

        if let Some(new_path) = self.get_http_request_header("set-path") {
            self.set_property(
                vec!["request.path"],
                Some(new_path.as_bytes()),
            );
        }

        if let Some(new_query) = self.get_http_request_header("set-query") {
            self.set_property(
                vec!["request.query"],
                Some(new_query.as_bytes()),
            );
        }

        self.set_property(
            vec!["nginx.log_field1"],
            Some(b"from_wasm nginx.log_field1"),
        );

        Action::Continue
    }

    fn on_log(&mut self) {
        info!("#{} completed.", self.context_id);
    }
}


