/*
* Copyright 2025 G-Core Innovations SARL
*/
//! # FastEdge Rust SDK
//!
//! A comprehensive toolkit for building high-performance edge computing applications using WebAssembly.
//!
//! ## Overview
//!
//! The `fastedge` crate provides two runtime models:
//!
//! * **WebAssembly Component Model** (default): Modern WebAssembly component model using [WIT] (WebAssembly Interface Types)
//!   for type-safe interfaces. This API resides in the root of the `fastedge` crate's namespace.
//!
//! * **ProxyWasm API**: Compatibility layer for [ProxyWasm] environments (Envoy, etc.).
//!   Available via the [`fastedge::proxywasm`](`proxywasm`) module when the `proxywasm` feature is enabled.
//!
//! [WIT]: https://component-model.bytecodealliance.org/design/wit.html
//! [WebAssembly components]: https://component-model.bytecodealliance.org
//! [ProxyWasm]: https://github.com/proxy-wasm/spec
//!
//! ## Features
//!
//! - **HTTP Request Handling**: Process incoming HTTP requests at the edge
//! - **Outbound HTTP Client**: Make HTTP requests to backend services via [`send_request`]
//! - **Key-Value Storage**: Persistent storage with advanced operations (scan, sorted sets, bloom filters)
//! - **Secret Management**: Secure access to encrypted secrets with time-based retrieval
//! - **Dictionary**: Fast read-only key-value lookups for configuration
//! - **WASI-NN**: Machine learning inference support via [`wasi_nn`]
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! fastedge = "0.3"
//! anyhow = "1.0"
//!
//! [lib]
//! crate-type = ["cdylib"]
//! ```
//!
//! Create a simple HTTP handler:
//!
//! ```no_run
//! use anyhow::Result;
//! use fastedge::body::Body;
//! use fastedge::http::{Request, Response, StatusCode};
//!
//! #[fastedge::http]
//! fn main(_req: Request<Body>) -> Result<Response<Body>> {
//!     Response::builder()
//!         .status(StatusCode::OK)
//!         .body(Body::from("Hello, FastEdge!"))
//!         .map_err(Into::into)
//! }
//! ```
//!
//! Build for WebAssembly:
//!
//! ```bash
//! rustup target add wasm32-wasip1
//! cargo build --target wasm32-wasip1 --release
//! ```
//!
//! ## Feature Flags
//!
//! - `proxywasm` (default): Enable ProxyWasm compatibility layer
//! - `json`: Enable JSON body support via `serde_json`
//!
//! ## Examples
//!
//! ### Making HTTP Requests
//!
//! ```no_run
//! use anyhow::Result;
//! use fastedge::body::Body;
//! use fastedge::http::{Method, Request, Response, StatusCode};
//!
//! #[fastedge::http]
//! fn main(_req: Request<Body>) -> Result<Response<Body>> {
//!     // Create a request to a backend service
//!     let backend_request = Request::builder()
//!         .method(Method::GET)
//!         .uri("https://api.example.com/data")
//!         .header("User-Agent", "FastEdge/1.0")
//!         .body(Body::empty())?;
//!     
//!     // Send the request
//!     let backend_response = fastedge::send_request(backend_request)?;
//!     
//!     // Return the response
//!     Response::builder()
//!         .status(StatusCode::OK)
//!         .body(backend_response.into_body())
//!         .map_err(Into::into)
//! }
//! ```
//!
//! ### Using Key-Value Storage
//!
//! ```no_run
//! use anyhow::Result;
//! use fastedge::body::Body;
//! use fastedge::http::{Request, Response, StatusCode};
//! use fastedge::key_value::Store;
//!
//! #[fastedge::http]
//! fn main(_req: Request<Body>) -> Result<Response<Body>> {
//!     // Open a key-value store
//!     let store = Store::open("my-store")?;
//!     
//!     // Get a value
//!     if let Some(value) = store.get("user:123")? {
//!         return Response::builder()
//!             .status(StatusCode::OK)
//!             .body(Body::from(value))
//!             .map_err(Into::into);
//!     }
//!     
//!     Response::builder()
//!         .status(StatusCode::NOT_FOUND)
//!         .body(Body::empty())
//!         .map_err(Into::into)
//! }
//! ```
//!
//! ### Accessing Secrets
//!
//! ```no_run
//! use anyhow::Result;
//! use fastedge::body::Body;
//! use fastedge::http::{Request, Response, StatusCode};
//! use fastedge::secret;
//!
//! #[fastedge::http]
//! fn main(_req: Request<Body>) -> Result<Response<Body>> {
//!     // Get a secret value
//!     match secret::get("API_KEY")? {
//!         Some(api_key) => {
//!             // Use the API key
//!             let key = String::from_utf8_lossy(&api_key);
//!             Response::builder()
//!                 .status(StatusCode::OK)
//!                 .body(Body::from("Secret retrieved"))
//!                 .map_err(Into::into)
//!         }
//!         None => {
//!             Response::builder()
//!                 .status(StatusCode::NOT_FOUND)
//!                 .body(Body::from("Secret not found"))
//!                 .map_err(Into::into)
//!         }
//!     }
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

/// Fast read-only key-value dictionary for configuration.
///
/// The dictionary provides efficient access to read-only configuration values.
/// It's optimized for fast lookups and is ideal for static configuration that
/// doesn't change during request processing.
///
/// # Examples
///
/// ```no_run
/// use fastedge::dictionary;
///
/// // Get a configuration value
/// if let Some(config) = dictionary::get("api_endpoint")? {
///     let endpoint = String::from_utf8_lossy(&config);
///     println!("API endpoint: {}", endpoint);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub mod dictionary {
    #[doc(inline)]
    pub use crate::gcore::fastedge::dictionary::get;
}

/// Secure access to encrypted secrets and credentials.
///
/// The secret module provides secure storage and retrieval of sensitive data such as
/// API keys, passwords, and certificates. Secrets are encrypted at rest and support
/// versioning with time-based retrieval.
///
/// # Security
///
/// - Secrets are encrypted and can only be accessed by authorized applications
/// - Access is controlled via platform permissions
/// - Never log or expose secret values in responses
///
/// # Examples
///
/// ```no_run
/// use fastedge::secret;
///
/// // Get current secret value
/// match secret::get("DATABASE_PASSWORD")? {
///     Some(password) => {
///         let pwd = String::from_utf8_lossy(&password);
///         // Use the password to connect to database
///     }
///     None => {
///         eprintln!("Secret not found");
///     }
/// }
///
/// // Get secret value at a specific time (for rotation scenarios)
/// use std::time::{SystemTime, UNIX_EPOCH};
/// let timestamp = SystemTime::now()
///     .duration_since(UNIX_EPOCH)?
///     .as_secs() as u32;
/// let historical = secret::get_effective_at("API_KEY", timestamp)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub mod secret {
    #[doc(inline)]
    pub use crate::gcore::fastedge::secret::get;
    #[doc(inline)]
    pub use crate::gcore::fastedge::secret::get_effective_at;
    pub use crate::gcore::fastedge::secret::Error;
}

/// Persistent key-value storage with advanced data structures.
///
/// The key-value module provides a persistent storage interface with support for:
/// - Simple key-value pairs
/// - Pattern-based scanning with glob-style patterns
/// - Sorted sets with score-based range queries
/// - Bloom filters for probabilistic set membership testing
///
/// # Storage Model
///
/// Data is organized into named stores. Applications must be granted access to specific
/// stores via platform configuration.
///
/// # Examples
///
/// ## Basic Operations
///
/// ```no_run
/// use fastedge::key_value::Store;
///
/// // Open a store
/// let store = Store::open("user-data")?;
///
/// // Get a value
/// if let Some(data) = store.get("user:123:profile")? {
///     let profile = String::from_utf8_lossy(&data);
///     println!("Profile: {}", profile);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Pattern Scanning
///
/// ```no_run
/// use fastedge::key_value::Store;
///
/// let store = Store::open("user-data")?;
///
/// // Find all keys matching a pattern
/// let user_keys = store.scan("user:123:*")?;
/// for key in user_keys {
///     println!("Found key: {}", key);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Sorted Sets (Leaderboards)
///
/// ```no_run
/// use fastedge::key_value::Store;
///
/// let store = Store::open("game-data")?;
///
/// // Get top players by score (score between 1000 and infinity)
/// let top_players = store.zrange_by_score("leaderboard", 1000.0, f64::INFINITY)?;
/// for (player_id, score) in top_players {
///     println!("Player: {:?}, Score: {}", player_id, score);
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
///
/// ## Bloom Filters
///
/// ```no_run
/// use fastedge::key_value::Store;
///
/// let store = Store::open("cache")?;
///
/// // Check if an item is in a bloom filter
/// if store.bf_exists("seen_urls", "https://example.com")? {
///     println!("URL was probably seen before");
/// }
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub mod key_value {
    #[doc(inline)]
    pub use crate::gcore::fastedge::key_value::Store;
    #[doc(inline)]
    pub use crate::gcore::fastedge::key_value::Error;
}

/// FastEdge-specific utility functions for diagnostics and statistics.
///
/// This module provides utilities for debugging and monitoring your FastEdge applications.
///
/// # Examples
///
/// ```no_run
/// use fastedge::utils::set_user_diag;
///
/// // Set diagnostic information for debugging
/// set_user_diag("Processing user request: 12345");
/// ```
///
/// Diagnostic messages can be viewed in the FastEdge platform logs.
pub mod utils {
    #[doc(inline)]
    pub use crate::gcore::fastedge::utils::set_user_diag;
}

/// Errors that can occur when using the FastEdge SDK.
///
/// This error type is returned by [`send_request`] and other SDK functions.
///
/// # Examples
///
/// ```
/// use fastedge::Error;
/// use fastedge::http::Method;
///
/// // Handling errors
/// fn handle_error(err: Error) {
///     match err {
///         Error::UnsupportedMethod(method) => {
///             eprintln!("Method {} is not supported", method);
///         }
///         Error::InvalidStatusCode(code) => {
///             eprintln!("Invalid status code: {}", code);
///         }
///         _ => {
///             eprintln!("An error occurred: {}", err);
///         }
///     }
/// }
/// ```
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The HTTP method is not supported by the FastEdge runtime.
    ///
    /// Only the following methods are supported: GET, POST, PUT, DELETE, HEAD, PATCH, OPTIONS.
    #[error("method `{0}` is not supported")]
    UnsupportedMethod(::http::Method),
    /// An error occurred in the underlying HTTP client component.
    #[error("http error: {0}")]
    BindgenHttpError(#[from] HttpError),
    /// An error occurred while building or parsing an HTTP message.
    #[error("http error: {0}")]
    HttpError(#[from] ::http::Error),
    /// The HTTP body is invalid or could not be processed.
    #[error("invalid http body")]
    InvalidBody,
    /// The HTTP status code is invalid (not in range 100-599).
    #[error("invalid status code {0}")]
    InvalidStatusCode(u16),
}

/// HTTP request and response body types.
///
/// This module provides the [`Body`] type which wraps [`Bytes`] and tracks content-type information.
/// The body type automatically handles content-type detection based on the input data.
///
/// # Examples
///
/// ```
/// use fastedge::body::Body;
///
/// // Create from string - automatically sets content-type to text/plain
/// let text_body = Body::from("Hello, world!");
/// assert_eq!(text_body.content_type(), "text/plain; charset=utf-8");
///
/// // Create from bytes - automatically sets content-type to application/octet-stream
/// let bytes_body = Body::from(vec![1, 2, 3, 4]);
/// assert_eq!(bytes_body.content_type(), "application/octet-stream");
///
/// // Create empty body
/// let empty_body = Body::empty();
/// assert_eq!(empty_body.len(), 0);
/// ```
pub mod body {
    use std::ops::Deref;

    use bytes::Bytes;

    /// HTTP request/response body with content-type tracking.
    ///
    /// The `Body` type wraps [`Bytes`] and maintains content-type information.
    /// It automatically detects and sets appropriate MIME types based on the input data.
    ///
    /// # Examples
    ///
    /// ```
    /// use fastedge::body::Body;
    ///
    /// // From string
    /// let body = Body::from("Hello");
    ///
    /// // From bytes
    /// let body = Body::from(vec![1, 2, 3]);
    ///
    /// // Empty body
    /// let body = Body::empty();
    /// ```
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
        /// Creates an empty body with default content-type.
        ///
        /// # Examples
        ///
        /// ```
        /// use fastedge::body::Body;
        ///
        /// let body = Body::empty();
        /// assert_eq!(body.len(), 0);
        /// ```
        pub fn empty() -> Self {
            Body::default()
        }

        /// Returns the MIME content-type of this body.
        ///
        /// The content-type is automatically determined when the body is created:
        /// - Text strings: `text/plain; charset=utf-8`
        /// - Byte arrays: `application/octet-stream`
        /// - JSON (with `json` feature): `application/json`
        ///
        /// # Examples
        ///
        /// ```
        /// use fastedge::body::Body;
        ///
        /// let body = Body::from("Hello");
        /// assert_eq!(body.content_type(), "text/plain; charset=utf-8");
        /// ```
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
