use proxy_wasm::traits::*;
use proxy_wasm::types::*;
use image::*;
use std::{env, env::VarError, io::Cursor, str::from_utf8};

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
    fn on_http_request_headers(&mut self, _: usize, _: bool) -> Action
    {
        // this header is used to select correct image version from cache
        self.add_http_request_header("Image-Format", "original");

        // get extension
        let Some(ext)= self.get_property(vec!["request.extension"]) else {
            println!("No extension in request path, not transforming");
            return Action::Continue;
        };
        let Ok(ext) = from_utf8(&ext) else {
            println!("Invalid UTF-8 in request extension, not transforming");
            return Action::Continue;
        };
        if ext.is_empty() {
            println!("No extension in request path, not transforming");
            return Action::Continue;
        }

        // FORMATS_TO_TRANSFORM contains list of file extensions to transfor
        // note that jpg and jpeg are different extensions
        let Ok(image_list) = str_param("FORMATS_TO_TRANSFORM") else {
            println!("FORMATS_TO_TRANSFORM param is not set, not transforming");
            return Action::Continue;
        };
        if !image_list.split(',').any(|entry| entry == ext) {
            println!("extension {} is not in the list of formats to transform: {}, not transforming", ext, image_list);
            return Action::Continue;
        }

        // requests from User agents that match substrings in the IGNORED_UA_LIST param are not transformed
        let Some(ua) = self.get_http_request_header("User-Agent") else {
            println!("User-Agent header is not set, not transforming");
            return Action::Continue;
        };
        if let Ok(ua_to_ignore) = str_param("IGNORED_UA_LIST") {
            if ua_to_ignore.split(",").any(|entry| ua.contains(entry)) {
                println!("User-Agent is in ignore list, not transforming");
                return Action::Continue;
            }
        }

        // indicator for on_response_headers and for cache key
        self.set_http_request_header("Image-Format", Some("image/avif"));

        Action::Continue
    }

    fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action
    {
        // only process 200 responses
        if let Some(status) = self.rsp_status() {
            if status != 200 {
                println!("Response status is {} instead of expected 200, not transforming", status);
                return Action::Continue;
            }
        } else {
            println!("Response status is not set, not transforming");
            return Action::Continue;
        }

        // if "Image-Format" request header is not set, don't convert the image
        let Some(content_type) = self.get_http_request_header("Image-Format") else {
            return Action::Continue;
        };
        // instruct cache to vary by this header so "original" and "image/avif" are cached separately
        self.add_http_response_header("Vary", "Image-Format");

        if content_type == "original" {
            return Action::Continue;
        };

        // image to be transformed, set headers accordingly
        self.set_http_response_header("Content-Length", None);
        self.set_http_response_header("Transfer-Encoding", Some("Chunked"));
        self.set_http_response_header("Content-Type", Some(content_type.as_str()));

        // indicate to on_http_response_body that transformation is needed
        self.set_property(vec!["response.content-type"], Some(content_type.as_bytes()));

        Action::Continue
    }

    fn on_http_response_body(&mut self, body_size: usize, end_of_stream: bool) -> Action
    {
        if !end_of_stream { // wait till we get complete body
            return Action::Pause;
        }

        let Some(content_type)= self.get_property(vec!["response.content-type"]) else {
            return Action::Continue;
        };

        let Ok(content_type) = from_utf8(&content_type) else {
            // should never happen
            println!("Invalid UTF-8 in Content-Type");
            self.send_http_response(500, vec![], None);
            return Action::Pause;
        };

        if content_type != "image/avif" {
            // should never happen
            println!("Content-Type {} is not supported, not transforming", content_type);
            return Action::Continue;
        }

        if let Some(body_bytes) = self.get_http_response_body(0, body_size) {
            let buf = body_bytes.as_bytes();
            let img = match load_from_memory(buf) {
                Ok(i) => i,
                Err(e) => {
                    println!("cannot load image to memory {}, not converting", e);
                    return Action::Continue
                }
            };

            let mut out = Vec::new();
            let mut c = Cursor::new(&mut out);
            let res = img.write_with_encoder(
                    codecs::avif::AvifEncoder::new_with_speed_quality(
                        &mut c,
                        u8_param("AVIF_SPEED", 1, 10, 5),
                        u8_param("AVIF_QUALITY", 1, 100, 70))
            );

            match res {
                Ok(_) => {
                    println!("{} bytes -> {} bytes {}", body_size, out.len(), content_type);
                    self.set_http_response_body(0, body_size, &out)
                }
                Err(e) => println!("cannot store transformed image {}", e)
            }
        } else {
            println!("No response body to transform");
        }

        Action::Continue
    }
}

impl HttpBody {
    fn rsp_status(&mut self) -> Option<u16> {
        if let Some(status)= self.get_property(vec!["response.status"]) {
            if status.len() != 2 {
                println!("HTTP status property is not 2 bytes");
                return None;
            }
            return Some(u16::from_be_bytes([status[0], status[1]]));
        }
        None
    }
}

fn str_param(name: &str) -> Result<String, VarError>
{
    let val = env::var(name)?;
    if val.is_empty() {
        return Err(VarError::NotPresent);
    }

    Ok(val)
}

fn u8_param(name: &str, min: u8, max: u8, default: u8) -> u8
{
    let Ok(val) = env::var(name) else {
        println!("Param {} is not set, using default value {}", name, default);
        return default;
    };
    if val.is_empty() {
        println!("Param {} is not set, using default value {}", name, default);
        return default;
    }

    let val = match val.parse() {
        Err(_) => {
            println!("Param {} is not a valid number, using default value {}", name, default);
            return default;
        }
        Ok(v) => v,
    };
    if val < min {
        println!("Param {} is below minimum {}, using default value {}", name, min, default);
        return default;
    }
    if val > max {
        println!("Param {} is above maximum {}, using default value {}", name, max, default);
        return default;
    }

    val
}
