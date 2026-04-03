use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::time::Duration;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpHeadersRoot) });
}}

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}

impl RootContext for HttpHeadersRoot {
    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpHeaders { state: 0 }))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct HttpHeaders {
    state: u32,
}

impl Context for HttpHeaders {
    fn on_http_call_response(
        &mut self,
        token_id: u32,
        num_headers: usize,
        body_size: usize,
        _num_trailers: usize,
    ) {
        println!(
            "Received http call response with token id: {token_id}, num_headers: {num_headers}"
        );
        //If num_headers is 0, then the HTTP call failed.
        if num_headers != 0 {
            let user_agent = self.get_http_call_response_header("user-agent");
            println!("User-Agent: {:?}", user_agent);

            let headers = self.get_http_call_response_headers();
            println!("Response headers:");
            for (name, value) in &headers {
                println!("  {}: {}", name, value);
            }
            let headers_value = self.get_http_call_response_headers_bytes();
            for (name, value) in &headers_value {
                println!("  {}: {:?}", name, value);
            }

            let body = self.get_http_call_response_body(0, body_size);
            println!("Response body: {:?}", body);

            self.state = 1; // Set state to 1 to indicate that the HTTP call response was received successfully.

            self.resume_http_request();
            // or self.resume_http_response()
        } else {
            self.reset_http_request();
        }
    }
}

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        println!("state: {}", self.state);

        if self.state == 1 {
            println!("HTTP call response was received successfully, resuming request.");
            return Action::Continue;
        }

        match self.dispatch_http_call(
            "httpbin.org",
            vec![
                (":scheme", "https"),
                (":authority", "httpbin.org"),
                (":path", "/ip"),
                ("User-Agent", "fastedge"),
            ],
            Some("body".as_bytes()),
            vec![],
            Duration::from_millis(1000),
        ) {
            Ok(token_id) => {
                println!("Dispatched http call with token id: {token_id}");
                Action::Pause
            }
            Err(status) => {
                self.send_http_response(
                    to_status_code(status),
                    vec![],
                    Some(format!("Failed to dispatch http call: {:?}", status).as_bytes()),
                );
                Action::Pause
            }
        }
    }
}

fn to_status_code(status: Status) -> u32 {
    match status {
        Status::Ok => 200,
        Status::NotFound => 404,
        Status::BadArgument => 400,
        Status::SerializationFailure => 500,
        Status::ParseFailure => 400,
        Status::Empty => 204,
        Status::CasMismatch => 409,
        Status::InternalFailure => 500,
        _ => 500,
    }
}
