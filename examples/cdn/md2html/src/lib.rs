use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use pulldown_cmark::{Parser, Options};
use std::env;

const BAD_REQUEST: u32 = 400;

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
        self.set_http_request_header("Accept-Encoding", None);  // don't want to process gzipped body
        let Ok(base) = env::var("BASE") else {
            println!("BASE is not set - URL is not modified");
            return Action::Continue;
        };
        let url = match self.get_property(vec!["request.path"]) {
            Some(url) => {
                match std::str::from_utf8(&url) {
                    Ok(u) => u.to_string(),
                    Err(e) => {
                        println!("Error parsing URL path: {}", e);
                        self.send_http_response(BAD_REQUEST, vec![], None);
                        return Action::Pause;
                    }
                }
            }
            None => {   // should never happen
                println!("URL path is missing");
                "/".to_string()
            }
        };
        let new_url = format!("{}{}", base.trim_end_matches('/'), url);
        self.set_property(vec!["request.path"], Some(new_url.as_bytes()));
        println!("URL modified: {} -> {}", url, new_url);
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        if let Some(content_type) = self.get_http_response_header("Content-Type") {
            if content_type.starts_with("text/plain") || content_type.starts_with("text/markdown") {
                self.set_http_response_header("Content-Length", None);
                self.set_http_response_header("Transfer-Encoding", Some("Chunked"));
                self.set_http_response_header("Content-Type", Some("text/html"));
                self.set_property(vec!["response.markdown"], Some(b"true"));
                println!("Response is markdown, converting to HTML");
            }
        }
        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action {
        // only process markdown
        if None == self.get_property(vec!["response.markdown"]) {
            return Action::Continue;
        }

        if !end_of_stream {     // wait for complete body
            return Action::Pause;
        }

        let Some(body_bytes) = self.get_http_response_body(0, body_size) else {
            return Action::Continue;
        };
        let Ok(md) = String::from_utf8(body_bytes) else {
            return Action::Continue;
        };

        let parser = Parser::new_ext(
            md.as_str(),
            Options::ENABLE_TABLES | Options::ENABLE_FOOTNOTES
        );
        let mut html = String::new();
        html.push_str("<!DOCTYPE html><html><body>");
        pulldown_cmark::html::push_html(&mut html, parser);
        html.push_str("</body></html>");

        let body = html.as_bytes();
        self.set_http_response_body(0, body.len(), body);
        println!("Converted");

        Action::Continue
    }
}
