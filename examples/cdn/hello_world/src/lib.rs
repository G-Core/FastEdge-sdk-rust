use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HelloWorldRoot) });
}}

struct HelloWorldRoot;

impl Context for HelloWorldRoot {}

impl RootContext for HelloWorldRoot {
    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }

    fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HelloWorld))
    }
}

struct HelloWorld;

impl Context for HelloWorld {}

impl HttpContext for HelloWorld {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        info!("Hello from on_http_request_headers");
        Action::Continue
    }

    fn on_http_request_body(&mut self, _: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        info!("Hello from on_http_request_body");
        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
        self.add_http_response_header("x-powered-by", "FastEdge");
        info!("Hello from on_http_response_headers");
        Action::Continue
    }

    fn on_http_response_body(&mut self, _: usize, end_of_stream: bool) -> Action {
        if !end_of_stream {
            return Action::Pause;
        }
        info!("Hello from on_http_response_body");
        Action::Continue
    }
}
