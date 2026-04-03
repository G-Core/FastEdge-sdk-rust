use log::info;
use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use std::borrow::Cow;

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
pub const REQUEST_SCHEME: &str = "request.scheme";
pub const REQUEST_EXTENSION: &str = "request.extension";
pub const REQUEST_QUERY: &str = "request.query";
pub const REQUEST_X_REAL_IP: &str = "request.x_real_ip";
pub const REQUEST_COUNTRY: &str = "request.country";
pub const REQUEST_CITY: &str = "request.city";
pub const REQUEST_ASN: &str = "request.asn";
pub const REQUEST_GEO_LAT: &str = "request.geo.lat";
pub const REQUEST_GEO_LONG: &str = "request.geo.long";
pub const REQUEST_REGION: &str = "request.region";
pub const REQUEST_CONTINENT: &str = "request.continent";
pub const REQUEST_COUNTRY_NAME: &str = "request.country.name";

impl HttpContext for HttpHeaders {
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action {
        let Some(uri) = self.get_property(vec![REQUEST_URI]) else {
            self.send_http_response(551, vec![], None);
            return Action::Pause;
        };
        println!(" uri = {} ", String::from_utf8_lossy(&uri));
        self.add_http_response_header_bytes("request-uri", &uri);

        let Some(host) = self.get_property(vec![REQUEST_HOST]) else {
            self.send_http_response(552, vec![], None);
            return Action::Pause;
        };
        println!(" host = {} ", String::from_utf8_lossy(&host));
        self.add_http_response_header_bytes("request-host", &host);

        let Some(path) = self.get_property(vec![REQUEST_PATH]) else {
            self.send_http_response(553, vec![], None);
            return Action::Pause;
        };
        println!(" path = {} ", String::from_utf8_lossy(&path));
        self.add_http_response_header_bytes("request-path", &path);

        let Some(scheme) = self.get_property(vec![REQUEST_SCHEME]) else {
            self.send_http_response(554, vec![], None);
            return Action::Pause;
        };
        println!(" scheme = {} ", String::from_utf8_lossy(&scheme));
        self.add_http_response_header_bytes("request-scheme", &scheme);

        let Some(extension) = self.get_property(vec![REQUEST_EXTENSION]) else {
            self.send_http_response(555, vec![], None);
            return Action::Pause;
        };
        println!(" extension = {} ", String::from_utf8_lossy(&extension));
        self.add_http_response_header_bytes("request-extension", &extension);

        let Some(query) = self.get_property(vec![REQUEST_QUERY]) else {
            self.send_http_response(556, vec![], None);
            return Action::Pause;
        };
        println!(" query = {} ", String::from_utf8_lossy(&query));
        self.add_http_response_header_bytes("request-query", &query);

        let Some(client_ip) = self.get_property(vec![REQUEST_X_REAL_IP]) else {
            self.send_http_response(557, vec![], None);
            return Action::Pause;
        };
        println!(" client_ip = {} ", String::from_utf8_lossy(&client_ip));
        self.add_http_response_header_bytes("request-x-real-ip", &client_ip);

        let Some(country) = self.get_property(vec![REQUEST_COUNTRY]) else {
            self.send_http_response(558, vec![], None);
            return Action::Pause;
        };
        println!(" country = {} ", String::from_utf8_lossy(&country));
        self.add_http_response_header_bytes("request-country", &country);

        let Some(city) = self.get_property(vec![REQUEST_CITY]) else {
            self.send_http_response(559, vec![], None);
            return Action::Pause;
        };
        println!(" city = {} ", String::from_utf8_lossy(&city));
        self.add_http_response_header_bytes("request-city", &city);

        let Some(value) = self.get_property(vec![REQUEST_ASN]) else {
            self.send_http_response(561, vec![], None);
            return Action::Pause;
        };
        println!(" asn = {} ", String::from_utf8_lossy(&value));
        self.add_http_response_header_bytes("request-asn", &value);

        let Some(value) = self.get_property(vec![REQUEST_GEO_LONG]) else {
            self.send_http_response(561, vec![], None);
            return Action::Pause;
        };
        println!(" long = {} ", String::from_utf8_lossy(&value));
        self.add_http_response_header_bytes("request-long", &value);

        let Some(value) = self.get_property(vec![REQUEST_GEO_LAT]) else {
            self.send_http_response(562, vec![], None);
            return Action::Pause;
        };
        println!(" lat = {} ", String::from_utf8_lossy(&value));
        self.add_http_response_header_bytes("request-lat", &value);

        let Some(value) = self.get_property(vec![REQUEST_COUNTRY_NAME]) else {
            self.send_http_response(563, vec![], None);
            return Action::Pause;
        };
        println!(" country names = {} ", String::from_utf8_lossy(&value));
        self.add_http_response_header_bytes("request-country-names", &value);

        let Some(value) = self.get_property(vec![REQUEST_REGION]) else {
            self.send_http_response(564, vec![], None);
            return Action::Pause;
        };
        println!(" region = {} ", String::from_utf8_lossy(&value));
        self.add_http_response_header_bytes("request-country-region", &value);

        let Some(value) = self.get_property(vec![REQUEST_CONTINENT]) else {
            self.send_http_response(565, vec![], None);
            return Action::Pause;
        };
        println!(" continent = {} ", String::from_utf8_lossy(&value));
        self.add_http_response_header_bytes("request-continent", &value);

        let query = String::from_utf8_lossy(&query);
        println!("query={}", query);
        let params = querystring::querify(&query);

        if let Some(url) = params.iter().find_map(|(k, v)| {
            if "url".eq_ignore_ascii_case(k) {
                Some(v)
            } else {
                None
            }
        }) {
            println!("change url to: {}", url);
            self.set_property(vec![REQUEST_URI], Some(url.as_bytes()));
        };

        if let Some(host) = params.iter().find_map(|(k, v)| {
            if "host".eq_ignore_ascii_case(k) {
                Some(v)
            } else {
                None
            }
        }) {
            println!("change host to: {}", host);
            self.set_property(vec![REQUEST_HOST], Some(host.as_bytes()));
        };

        if let Some(path) = params.iter().find_map(|(k, v)| {
            if "path".eq_ignore_ascii_case(k) {
                Some(v)
            } else {
                None
            }
        }) {
            println!("change path to: {}", path);
            self.set_property(vec![REQUEST_PATH], Some(path.as_bytes()));
        };

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

pub fn deserialize_country_names(bytes: &[u8]) -> Vec<Cow<'_, str>> {
    let mut path = Vec::new();
    if bytes.is_empty() {
        return path;
    }
    let mut p = 0;
    while p < bytes.len() {
        let s = p;
        while p < bytes.len() && bytes[p] != 0 {
            p += 1;
        }
        path.push(String::from_utf8_lossy(&bytes[s..p]));
        p += 1;
    }
    path
}
