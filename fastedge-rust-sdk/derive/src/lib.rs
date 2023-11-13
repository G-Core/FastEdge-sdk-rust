/*
* Copyright 2023 G-Core Innovations SARL
*/
use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Main function attribute for a FastEdge application.
///
/// ## Usage
///
/// The `main` function takes a request and returns a response or an error. For example:
///
/// ```rust,no_run
/// use anyhow::Result;
/// use fastedge::http::{Request, Response, StatusCode};
/// use fastedge::body::Body;
///
/// #[fastedge::http]
/// fn main(req: Request<Body>) -> Result<Response<Body>> {
///     Response::builder().status(StatusCode::OK).body(Body::empty())
/// }
#[proc_macro_attribute]
pub fn http(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;

    quote!(
        use fastedge::bindgen::__link_section;
        use fastedge::bindgen::exports;

        struct Component;
        fastedge::export_http_reactor!(Component);

        #[inline(always)]
        fn internal_error(body: &str) -> ::fastedge::http_handler::Response {
            ::fastedge::http_handler::Response {
                status: ::fastedge::http::StatusCode::INTERNAL_SERVER_ERROR.as_u16(),
                headers: Some(vec![]),
                body: Some(body.as_bytes().to_vec()),
            }
        }

        #[inline(always)]
        #[no_mangle]
        #func

        impl ::fastedge::http_handler::HttpHandler for Component {
            #[no_mangle]
            fn process(req: ::fastedge::http_handler::Request) -> ::fastedge::http_handler::Response {

                let Ok(request) = req.try_into() else {
                    return internal_error("http request decode error")
                };

                let res = match #func_name(request) {
                    Ok(res) => res,
                    Err(error) => {
                        return internal_error(error.to_string().as_str());
                    }
                };

                let Ok(response) = ::fastedge::http_handler::Response::try_from(res) else {
                    return internal_error("http response encode error")
                };
                response
            }
        }

    ).into()
}
