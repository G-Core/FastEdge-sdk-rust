/*
* Copyright 2024 G-Core Innovations SARL
*/
//! Procedural macros for FastEdge applications.
//!
//! This crate provides the `#[fastedge::http]` attribute macro that transforms
//! a regular Rust function into a WebAssembly component handler for HTTP requests.

use proc_macro::TokenStream;

use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// Marks a function as the HTTP request handler for a FastEdge application.
///
/// This attribute macro transforms your function into a WebAssembly component that
/// can process HTTP requests. The function must have the following signature:
///
/// ```ignore
/// fn handler_name(req: Request<Body>) -> Result<Response<Body>>
/// ```
///
/// # Function Requirements
///
/// - **Parameter**: Must accept a single parameter of type `Request<Body>`
/// - **Return Type**: Must return `Result<Response<Body>>` (typically using `anyhow::Result`)
/// - **Function Name**: Can be any valid Rust identifier (commonly `main`)
///
/// # Error Handling
///
/// If your function returns an `Err`, the macro automatically converts it into an
/// HTTP 500 (Internal Server Error) response with the error message in the body.
///
/// # Examples
///
/// ## Basic Handler
///
/// ```no_run
/// use anyhow::Result;
/// use fastedge::body::Body;
/// use fastedge::http::{Request, Response, StatusCode};
///
/// #[fastedge::http]
/// fn main(_req: Request<Body>) -> Result<Response<Body>> {
///     Response::builder()
///         .status(StatusCode::OK)
///         .body(Body::from(\"Hello, World!\"))
///         .map_err(Into::into)
/// }
/// ```
///
/// ## Request Processing
///
/// ```no_run
/// use anyhow::{Result, anyhow};
/// use fastedge::body::Body;
/// use fastedge::http::{Method, Request, Response, StatusCode};
///
/// #[fastedge::http]
/// fn main(req: Request<Body>) -> Result<Response<Body>> {
///     match req.method() {
///         &Method::GET => {
///             Response::builder()
///                 .status(StatusCode::OK)
///                 .body(Body::from(\"GET request received\"))
///                 .map_err(Into::into)
///         }
///         &Method::POST => {
///             let body_data = req.body();
///             Response::builder()
///                 .status(StatusCode::CREATED)
///                 .body(Body::from(format!(\"Received {} bytes\", body_data.len())))
///                 .map_err(Into::into)
///         }
///         _ => {
///             Err(anyhow!(\"Method not allowed\"))
///         }
///     }
/// }
/// ```
///
/// ## With Backend Requests
///
/// ```no_run
/// use anyhow::Result;
/// use fastedge::body::Body;
/// use fastedge::http::{Method, Request, Response, StatusCode};
///
/// #[fastedge::http]
/// fn main(req: Request<Body>) -> Result<Response<Body>> {
///     // Make a request to a backend service
///     let backend_req = Request::builder()
///         .method(Method::GET)
///         .uri(\"https://api.example.com/data\")
///         .body(Body::empty())?;
///     
///     let backend_resp = fastedge::send_request(backend_req)?;
///     
///     // Forward the response
///     Ok(backend_resp)
/// }
/// ```
///
/// ## Error Handling
///
/// ```no_run
/// use anyhow::{Result, Context};
/// use fastedge::body::Body;
/// use fastedge::http::{Request, Response, StatusCode};
///
/// #[fastedge::http]
/// fn main(req: Request<Body>) -> Result<Response<Body>> {
///     let query = req.uri()
///         .query()
///         .context(\"Missing query parameters\")?;
///     
///     // Process query...
///     
///     Response::builder()
///         .status(StatusCode::OK)
///         .body(Body::from(\"Success\"))
///         .map_err(Into::into)
/// }
/// ```
///
/// # Generated Code
///
/// The macro generates a WebAssembly component that:
/// 1. Implements the `Guest` trait for the HTTP handler interface
/// 2. Converts between bindgen types and standard `http` crate types
/// 3. Handles error conversion to HTTP responses
/// 4. Exports the component for use by the FastEdge runtime
///
/// # See Also
///
/// - [`fastedge::http`](https://docs.rs/fastedge/latest/fastedge/http/index.html) - HTTP types module
/// - [`fastedge::body::Body`](https://docs.rs/fastedge/latest/fastedge/body/struct.Body.html) - Body type
#[proc_macro_attribute]
pub fn http(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let func = parse_macro_input!(item as ItemFn);
    let func_name = &func.sig.ident;

    quote!(
        use fastedge::http_handler::Guest;
        struct Component;

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

        impl Guest for Component {
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

        fastedge::export!(Component with_types_in fastedge);


    ).into()
}
