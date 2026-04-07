use std::time::{SystemTime, UNIX_EPOCH};
use headers::HeaderValue;
use headers::authorization::{Bearer, Credentials};

use fastedge::proxywasm::secret;
use jsonwebtoken::{decode, DecodingKey, Validation};
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use serde::Deserialize;

proxy_wasm::main! {{
    proxy_wasm::set_log_level(LogLevel::Trace);
    proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpHeadersRoot) });
}}

struct HttpHeadersRoot;

impl Context for HttpHeadersRoot {}

impl RootContext for HttpHeadersRoot {
    fn create_http_context(&self, _context_id: u32) -> Option<Box<dyn HttpContext>> {
        Some(Box::new(HttpHeaders {}))
    }

    fn get_type(&self) -> Option<ContextType> {
        Some(ContextType::HttpContext)
    }
}

struct HttpHeaders {}

impl Context for HttpHeaders {}

const UNAUTHORIZED: u32 = 401;
const FORBIDDEN: u32 = 403;
const INTERNAL_SERVER_ERROR: u32 = 500;

#[derive(Debug, Deserialize, Default)]
struct Claims {
    exp: u64,
}

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let Ok(Some(secret)) = secret::get("secret") else {
            println!("'secret' param not set");
            self.send_http_response(INTERNAL_SERVER_ERROR, vec![], Some(b"App misconfigured"));
            return Action::Pause;
        };
        let Some(value) = self.get_http_request_header("Authorization") else {
            println!("No auth header");
            self.send_http_response(UNAUTHORIZED, vec![], Some(b"No Authorization header"));
            return Action::Pause;
        };

       if value.is_empty() {
            println!("Auth header is empty");
            self.send_http_response(UNAUTHORIZED, vec![], Some(b"No Authorization header"));
            return Action::Pause;
        };

        let Ok(header) = value.parse::<HeaderValue>() else {
            println!("Auth header is invalid");
            self.send_http_response(UNAUTHORIZED, vec![], Some(b"Invalid Authorization header"));
            return Action::Pause;
        };


        let Some(bearer) = Bearer::decode(&header) else {
            println!("Auth header doesn't contain token");
            self.send_http_response(FORBIDDEN, vec![], Some(b"Token not found"));
            return Action::Pause;
        };

        let token = bearer.token();

        let decoding_key = DecodingKey::from_secret(&secret);
        let mut validation = Validation::default();
        validation.set_required_spec_claims(&["exp"]);
        // skip validation af aud and nbf claims
        validation.validate_aud = false;
        validation.validate_nbf = false;
        validation.validate_exp = false;  // will validate expiration separately

        let token_data = match decode::<Claims>(token, &decoding_key, &validation) {
            Ok(token_data) => token_data,
            Err(error) => {
                println!("Token is invalid");
                self.send_http_response(FORBIDDEN, vec![], Some(format!("Could not decode token {}: {}", token, error).as_bytes()));
                return Action::Pause;
            }
        };

        let claims = token_data.claims;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if now > claims.exp {
            println!("Token expired");
            self.send_http_response(FORBIDDEN, vec![], Some(b"Token expired"));
            return Action::Pause;
        }

        println!("Token ok");
        Action::Continue
    }
}
