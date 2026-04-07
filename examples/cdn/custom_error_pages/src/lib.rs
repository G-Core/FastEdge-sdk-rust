use handlebars::Handlebars;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde_json::json;
use std::collections::HashMap;
use std::env;

// Include the generated image map
include!(concat!(env!("OUT_DIR"), "/image_map.rs"));
include!(concat!(env!("OUT_DIR"), "/message_map.rs"));

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
    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        if let Some(status) = self.get_property(vec!["response.status"]) {
            if status.len() == 2 {
                let status_code = u16::from_be_bytes([status[0], status[1]]);
                if (400..600).contains(&status_code) {
                    // Remove the Content-Length header if it exists, we are going to change the response body
                    self.set_http_response_header("Content-Length", None);
                    self.set_http_response_header("Transfer-Encoding", Some("Chunked"));
                    self.set_http_response_header("Content-Type", Some("text/html"));
                }
            }
        }
        Action::Continue
    }

    fn on_http_response_body(&mut self, _body_size: usize, end_of_stream: bool) -> Action {
        // only process 4xx/5xx error responses
        let Some(status) = self.get_property(vec!["response.status"]) else {
            return Action::Continue;
        };
        if status.len() != 2 {
            return Action::Continue;
        }
        let status_code = u16::from_be_bytes([status[0], status[1]]);
        if !(400..600).contains(&status_code) {
            return Action::Continue;
        }

        if !end_of_stream {
            // wait for complete body
            return Action::Pause;
        }

        // Get the image and message maps
        let image_map = get_image_map();
        let message_map = get_message_map();

        // Get the Base64-encoded image for the status code or its fallback
        let base64_image = image_map
            .get(&status_code)
            .or_else(|| {
                if (400..500).contains(&status_code) {
                    image_map.get(&4000)
                } else if (500..600).contains(&status_code) {
                    image_map.get(&5000)
                } else {
                    None
                }
            })
            .unwrap_or(&"");

        // Get the message and description for the status code or its fallback
        let (message, description) = message_map
            .get(&status_code)
            .or_else(|| {
                if (400..500).contains(&status_code) {
                    message_map.get(&4000)
                } else if (500..600).contains(&status_code) {
                    message_map.get(&5000)
                } else {
                    None
                }
            })
            .map(|(msg, desc)| (msg.to_string(), desc.to_string()))
            .unwrap_or_else(|| {
                (
                    "Unexpected Error".to_string(),
                    "The server responded with a {{status}} error.".to_string(),
                )
            });

        let mut handlebars = Handlebars::new();
        // Use handlebars to complete message and description text allowing for usage of {{ status }} variable
        handlebars
            .register_template_string("message_template", message)
            .unwrap();
        handlebars
            .register_template_string("description_template", description)
            .unwrap();

        let msg_data = json!({
            "status": status_code.to_string(),
        });

        let complete_message = handlebars.render("message_template", &msg_data).unwrap();
        let complete_description = handlebars
            .render("description_template", &msg_data)
            .unwrap();

        // Render the error page using Handlebars
        let error_template = include_str!("../templates/error_page.hbs");
        handlebars
            .register_template_string("error_template", error_template)
            .unwrap();

        let styles = include_str!("../public/styles.css");
        let page_data = json!({
            "styles": styles,
            "status": status_code.to_string(),
            "message": complete_message,
            "description": complete_description,
            "image": base64_image,
        });

        let html_body = handlebars.render("error_template", &page_data).unwrap();
        let body = html_body.as_bytes();
        self.set_http_response_body(0, body.len(), body);

        Action::Continue
    }
}
