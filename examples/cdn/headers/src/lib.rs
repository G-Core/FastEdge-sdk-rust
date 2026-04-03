use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::collections::HashSet;

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
        let mut original_headers = HashSet::new();
        let mut original_headers_bytes = HashSet::new();

        // iterate over the headers and print them
        for (name, value) in self.get_http_request_headers().into_iter() {
            println!("#{} -> {}: {}", self.context_id, name, value);
            original_headers.insert((name, value));
        }
        for (name, value) in self.get_http_request_headers_bytes().into_iter() {
            println!("#{} -> {}: {:?}", self.context_id, name, value);
            original_headers_bytes.insert((name, value));
        }
        if original_headers.is_empty() || original_headers_bytes.is_empty() {
            self.send_http_response(550, vec![], None);
            return Action::Pause;
        }

        // check if the host header is present
        if self.get_http_request_header("host").is_none() {
            self.send_http_response(551, vec![], None);
            return Action::Pause;
        }
        if self.get_http_request_header_bytes("host").is_none() {
            self.send_http_response(551, vec![], None);
            return Action::Pause;
        }

        // add new headers
        self.add_http_request_header("new-header-01", "value-01");
        self.add_http_request_header_bytes("new-header-bytes-01", b"value-bytes-01");

        self.add_http_request_header("new-header-02", "value-02");
        self.add_http_request_header_bytes("new-header-bytes-02", b"value-bytes-02");

        self.add_http_request_header("new-header-03", "value-03");
        self.add_http_request_header_bytes("new-header-bytes-03", b"value-bytes-03");

        //remove header new-headter-01, expected empty value
        self.set_http_request_header("new-header-01", None);
        self.set_http_request_header_bytes("new-header-bytes-01", None);

        // changing header value
        self.set_http_request_header("new-header-02", Some("new-value-02"));
        self.set_http_request_header_bytes("new-header-bytes-02", Some(b"new-value-bytes-02"));

        // add new header with existing name
        self.add_http_request_header("new-header-03", "value-03-a");
        self.add_http_request_header_bytes("new-header-bytes-03", b"value-bytes-03-a");

        // try to set/add response headers
        self.add_http_response_header("new-response-header", "value-01");
        self.set_http_response_header("cache-control", None);
        self.set_http_response_header("new-response-header", Some("value-02"));

        // get new headers
        let headers = self
            .get_http_request_headers()
            .into_iter()
            .collect::<HashSet<(String, String)>>();
        let headers_bytes = self
            .get_http_request_headers_bytes()
            .into_iter()
            .collect::<HashSet<(String, Bytes)>>();

        let expected = [
            ("new-header-01".to_string(), "".to_string()),
            ("new-header-bytes-01".to_string(), "".to_string()),
            ("new-header-02".to_string(), "new-value-02".to_string()),
            (
                "new-header-bytes-02".to_string(),
                "new-value-bytes-02".to_string(),
            ),
            ("new-header-03".to_string(), "value-03".to_string()),
            (
                "new-header-bytes-03".to_string(),
                "value-bytes-03".to_string(),
            ),
            ("new-header-03".to_string(), "value-03-a".to_string()),
            (
                "new-header-bytes-03".to_string(),
                "value-bytes-03-a".to_string(),
            ),
        ];

        let expected = expected.iter().collect::<HashSet<_>>();

        let expected_bytes = [
            ("new-header-01".to_string(), b"".to_vec()),
            ("new-header-bytes-01".to_string(), b"".to_vec()),
            ("new-header-02".to_string(), b"new-value-02".to_vec()),
            (
                "new-header-bytes-02".to_string(),
                b"new-value-bytes-02".to_vec(),
            ),
            ("new-header-03".to_string(), b"value-03".to_vec()),
            (
                "new-header-bytes-03".to_string(),
                b"value-bytes-03".to_vec(),
            ),
            ("new-header-03".to_string(), b"value-03-a".to_vec()),
            (
                "new-header-bytes-03".to_string(),
                b"value-bytes-03-a".to_vec(),
            ),
        ];

        let expected_bytes = expected_bytes.iter().collect::<HashSet<_>>();

        let diff = headers
            .difference(&original_headers)
            .collect::<HashSet<_>>();

        let diff_bytes = headers_bytes
            .difference(&original_headers_bytes)
            .collect::<HashSet<_>>();

        let diff = diff.difference(&expected).collect::<Vec<_>>();

        if !diff.is_empty() {
            println!("different headers: {:?}", diff);
            self.send_http_response(552, vec![], None);
            return Action::Pause;
        }

        let diff_bytes = diff_bytes.difference(&expected_bytes).collect::<Vec<_>>();
        if !diff_bytes.is_empty() {
            println!("different headers bytes: {:?}", diff);
            self.send_http_response(552, vec![], None);
            return Action::Pause;
        }

        // check if the response header is not returned
        let Some(value) = self.get_http_response_header("host") else {
            self.send_http_response(553, vec![], None);
            return Action::Pause;
        };
        if !value.is_empty() {
            self.send_http_response(554, vec![], None);
            return Action::Pause;
        }
        let Some(value) = self.get_http_response_header_bytes("host") else {
            self.send_http_response(553, vec![], None);
            return Action::Pause;
        };
        if !value.is_empty() {
            self.send_http_response(554, vec![], None);
            return Action::Pause;
        }

        let response_headers = self.get_http_response_headers();
        if response_headers.len() != 1 {
            self.send_http_response(555, vec![], None);
            return Action::Pause;
        }
        let Some((name, value)) = response_headers.into_iter().next() else {
            self.send_http_response(555, vec![], None);
            return Action::Pause;
        };
        if name != "new-response-header" || value != "value-02" {
            self.send_http_response(556, vec![], None);
            return Action::Pause;
        }

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        let mut original_headers = HashSet::new();
        let mut original_headers_bytes = HashSet::new();

        // iterate over the headers and print them
        for (name, value) in self.get_http_response_headers().into_iter() {
            println!("#{} -> {}: {}", self.context_id, name, value);
            original_headers.insert((name, value));
        }
        for (name, value) in self.get_http_response_headers_bytes().into_iter() {
            println!("#{} -> {}: {:?}", self.context_id, name, value);
            original_headers_bytes.insert((name, value));
        }
        if original_headers.is_empty() || original_headers_bytes.is_empty() {
            self.send_http_response(550, vec![], None);
            return Action::Pause;
        }

        // check if the host header is present
        if self.get_http_response_header("host").is_none() {
            self.send_http_response(551, vec![], None);
            return Action::Pause;
        }
        if self.get_http_response_header_bytes("host").is_none() {
            self.send_http_response(551, vec![], None);
            return Action::Pause;
        }

        // add new headers
        self.add_http_response_header("new-header-01", "value-01");
        self.add_http_response_header_bytes("new-header-bytes-01", b"value-bytes-01");

        self.add_http_response_header("new-header-02", "value-02");
        self.add_http_response_header_bytes("new-header-bytes-02", b"value-bytes-02");

        self.add_http_response_header("new-header-03", "value-03");
        self.add_http_response_header_bytes("new-header-bytes-03", b"value-bytes-03");

        //remove header new-headter-01, expected empty value
        self.set_http_response_header("new-header-01", None);
        self.set_http_response_header_bytes("new-header-bytes-01", None);

        // changing header value
        self.set_http_response_header("new-header-02", Some("new-value-02"));
        self.set_http_response_header_bytes("new-header-bytes-02", Some(b"new-value-bytes-02"));

        // add new header with existing name
        self.add_http_response_header("new-header-03", "value-03-a");
        self.add_http_response_header_bytes("new-header-bytes-03", b"value-bytes-03-a");

        // get new headers
        let headers = self
            .get_http_response_headers()
            .into_iter()
            .collect::<HashSet<(String, String)>>();
        let headers_bytes = self
            .get_http_response_headers_bytes()
            .into_iter()
            .collect::<HashSet<(String, Bytes)>>();

        let expected = [
            ("new-header-01".to_string(), "".to_string()),
            ("new-header-bytes-01".to_string(), "".to_string()),
            ("new-header-02".to_string(), "new-value-02".to_string()),
            (
                "new-header-bytes-02".to_string(),
                "new-value-bytes-02".to_string(),
            ),
            ("new-header-03".to_string(), "value-03".to_string()),
            (
                "new-header-bytes-03".to_string(),
                "value-bytes-03".to_string(),
            ),
            ("new-header-03".to_string(), "value-03-a".to_string()),
            (
                "new-header-bytes-03".to_string(),
                "value-bytes-03-a".to_string(),
            ),
        ];

        let expected = expected.iter().collect::<HashSet<_>>();

        let expected_bytes = [
            ("new-header-01".to_string(), b"".to_vec()),
            ("new-header-bytes-01".to_string(), b"".to_vec()),
            ("new-header-02".to_string(), b"new-value-02".to_vec()),
            (
                "new-header-bytes-02".to_string(),
                b"new-value-bytes-02".to_vec(),
            ),
            ("new-header-03".to_string(), b"value-03".to_vec()),
            (
                "new-header-bytes-03".to_string(),
                b"value-bytes-03".to_vec(),
            ),
            ("new-header-03".to_string(), b"value-03-a".to_vec()),
            (
                "new-header-bytes-03".to_string(),
                b"value-bytes-03-a".to_vec(),
            ),
        ];

        let expected_bytes = expected_bytes.iter().collect::<HashSet<_>>();

        let diff = headers
            .difference(&original_headers)
            .collect::<HashSet<_>>();

        let diff_bytes = headers_bytes
            .difference(&original_headers_bytes)
            .collect::<HashSet<_>>();

        if expected != diff {
            let diff = diff.difference(&expected).collect::<Vec<_>>();
            println!("different headers: {:?}", diff);
            self.send_http_response(552, vec![], None);
            return Action::Pause;
        }

        if expected_bytes != diff_bytes {
            let diff = diff_bytes.difference(&expected_bytes).collect::<Vec<_>>();
            println!("different headers bytes: {:?}", diff);
            self.send_http_response(552, vec![], None);
            return Action::Pause;
        }

        // check if the reponse header is not returnd
        let Some(value) = self.get_http_response_header("host") else {
            self.send_http_response(553, vec![], None);
            return Action::Pause;
        };
        if !value.is_empty() {
            self.send_http_response(554, vec![], None);
            return Action::Pause;
        }
        let Some(value) = self.get_http_response_header_bytes("host") else {
            self.send_http_response(553, vec![], None);
            return Action::Pause;
        };
        if !value.is_empty() {
            self.send_http_response(554, vec![], None);
            return Action::Pause;
        }

        let request_headers = self.get_http_response_headers();
        if request_headers.is_empty() {
            self.send_http_response(555, vec![], None);
            return Action::Pause;
        }

        Action::Continue
    }

    fn on_log(&mut self) {
        println!("#{} completed.", self.context_id);
    }
}
