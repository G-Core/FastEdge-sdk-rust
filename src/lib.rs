/*
* Copyright 2025 G-Core Innovations SARL
*/
//! # Rust SDK for FastEdge.
//! The `fastedge` crate is being split into two halves:
//!
//! * One half of the crate implements FastEdge WebAssembly component model and is intended for working with [WebAssembly components]. This API resides in the root of the `fastedge` crate's namespace.
//!
//! * The second half of the crate is for use with the [ProxyWasm] API. This implementation is present in [`fastedge::proxywasm`](`proxywasm`).
//!
//! [WebAssembly components]: https://component-model.bytecodealliance.org
//! [ProxyWasm]: https://github.com/proxy-wasm/spec
//!
//! An example of using FastEdge looks like:
//!
//! ```
//! use fastedge::body::Body;
//! use fastedge::http::{Request, Response, StatusCode};
//! #[allow(dead_code)]
//! #[fastedge::http]
//! fn main(_req: Request<Body>) -> Result<Response<Body>> {
//!     let res = Response::builder()
//!         .status(StatusCode::OK)
//!         .body(Body::empty())?;
//!     Ok(res)
//! }
//! ```
pub extern crate http;

pub use fastedge_derive::http;
pub use http_client::send_request;

#[doc(hidden)]
pub use crate::exports::gcore::fastedge::http_handler;
use crate::gcore::fastedge::http::{Error as HttpError, Method, Request, Response};

mod helper;

/// Implementation of Outbound HTTP component
mod http_client;

/// FastEdge ProxyWasm module extension
#[cfg(feature = "proxywasm")]
pub mod proxywasm;

pub mod wasi_nn {
    #![allow(missing_docs)]
    wit_bindgen::generate!({
        world: "ml",
        path: "wasi-nn/wit"
    });
}

wit_bindgen::generate!({
    world: "reactor",
    path: "wit",
    pub_export_macro: true,
});

/// Helper functions for dictionary interface
pub mod dictionary {
    #[doc(inline)]
    pub use crate::gcore::fastedge::dictionary::get;
}

/// Helper functions for secret interface
pub mod secret {
    #[doc(inline)]
    pub use crate::gcore::fastedge::secret::get;
    #[doc(inline)]
    pub use crate::gcore::fastedge::secret::get_effective_at;
    pub use crate::gcore::fastedge::secret::Error;
}

/// FastEdge key-value persistent storage.
/// This module provides an interface for key-value storage, which is implemented by the host.
pub mod key_value {
    #[doc(inline)]
    pub use crate::gcore::fastedge::key_value::Store;
    #[doc(inline)]
    pub use crate::gcore::fastedge::key_value::Error;
}

/// FastEdge-specific utility functions for diagnostics and statistics.
pub mod utils {
    #[doc(inline)]
    pub use crate::gcore::fastedge::utils::set_user_diag;
}

/// Error type returned by [`send_request`]
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Unknown request method type
    #[error("method `{0}` is not supported")]
    UnsupportedMethod(::http::Method),
    /// Wrap FastEdge bindgen ['Error'][crate::http::Error] to this error type
    #[error("http error: {0}")]
    BindgenHttpError(#[from] HttpError),
    /// Wrap ['Error'][::http::Error] to this error type
    #[error("http error: {0}")]
    HttpError(#[from] ::http::Error),
    /// Wraps response Builder::body() error
    #[error("invalid http body")]
    InvalidBody,
    /// Wraps response InvalidStatusCode error
    #[error("invalid status code {0}")]
    InvalidStatusCode(u16),
}

/// Helper types for http component
pub mod body {
    use std::ops::Deref;

    use bytes::Bytes;

    /// FastEdge request/response body
    #[derive(Debug)]
    pub struct Body {
        pub(crate) content_type: String,
        pub(crate) inner: Bytes,
    }

    impl Deref for Body {
        type Target = Bytes;

        fn deref(&self) -> &Self::Target {
            &self.inner
        }
    }

    impl From<String> for Body {
        fn from(value: String) -> Self {
            Body {
                content_type: mime::TEXT_PLAIN_UTF_8.to_string(),
                inner: Bytes::from(value),
            }
        }
    }

    impl From<&'static str> for Body {
        fn from(value: &'static str) -> Self {
            Body {
                content_type: mime::TEXT_PLAIN_UTF_8.to_string(),
                inner: Bytes::from(value),
            }
        }
    }

    impl From<Vec<u8>> for Body {
        fn from(value: Vec<u8>) -> Self {
            Body {
                content_type: mime::APPLICATION_OCTET_STREAM.to_string(),
                inner: Bytes::from(value),
            }
        }
    }

    impl From<&'static [u8]> for Body {
        fn from(value: &'static [u8]) -> Self {
            Body {
                content_type: mime::APPLICATION_OCTET_STREAM.to_string(),
                inner: Bytes::from(value),
            }
        }
    }

    #[cfg(feature = "json")]
    impl TryFrom<serde_json::Value> for Body {
        type Error = serde_json::Error;
        fn try_from(value: serde_json::Value) -> Result<Self, Self::Error> {
            Ok(Body {
                content_type: mime::APPLICATION_JSON.to_string(),
                inner: Bytes::from(serde_json::to_vec(&value)?),
            })
        }
    }

    impl Default for Body {
        fn default() -> Self {
            Self {
                content_type: mime::TEXT_PLAIN_UTF_8.to_string(),
                inner: Bytes::default(),
            }
        }
    }

    impl Body {
        /// Default empty body factory function
        pub fn empty() -> Self {
            Body::default()
        }

        /// Body content type
        pub fn content_type(&self) -> String {
            self.content_type.to_owned()
        }
    }
}

impl From<Method> for ::http::Method {
    fn from(method: Method) -> Self {
        match method {
            Method::Get => ::http::Method::GET,
            Method::Post => ::http::Method::POST,
            Method::Put => ::http::Method::PUT,
            Method::Delete => ::http::Method::DELETE,
            Method::Head => ::http::Method::HEAD,
            Method::Patch => ::http::Method::PATCH,
            Method::Options => ::http::Method::OPTIONS,
        }
    }
}

impl TryFrom<Request> for ::http::Request<body::Body> {
    type Error = Error;

    fn try_from(req: Request) -> Result<Self, Self::Error> {
        let builder = ::http::Request::builder()
            .method(::http::Method::from(req.method))
            .uri(req.uri.to_string());
        let builder = req
            .headers
            .iter()
            .fold(builder, |builder, (k, v)| builder.header(k, v));

        let body = req.body.map_or_else(body::Body::empty, body::Body::from);
        builder.body(body).map_err(|_| Error::InvalidBody)
    }
}

impl From<::http::Response<body::Body>> for Response {
    fn from(res: ::http::Response<body::Body>) -> Self {
        let status = res.status().as_u16();
        let headers = if !res.headers().is_empty() {
            Some(
                res.headers()
                    .iter()
                    .map(|(name, value)| (name.to_string(), value.to_str().unwrap().to_string()))
                    .collect::<Vec<(String, String)>>(),
            )
        } else {
            None
        };

        let body = Some(res.into_body().to_vec());

        Response {
            status,
            headers,
            body,
        }
    }
}

impl TryFrom<Response> for ::http::Response<body::Body> {
    type Error = Error;

    fn try_from(res: Response) -> Result<Self, Self::Error> {
        let builder = ::http::Response::builder().status(
            ::http::StatusCode::try_from(res.status)
                .map_err(|_| Error::InvalidStatusCode(res.status))?,
        );
        let builder = if let Some(headers) = res.headers {
            headers
                .iter()
                .fold(builder, |builder, (k, v)| builder.header(k, v))
        } else {
            builder
        };

        let body = res.body.map_or_else(body::Body::empty, body::Body::from);
        builder.body(body).map_err(|_| Error::InvalidBody)
    }
}
