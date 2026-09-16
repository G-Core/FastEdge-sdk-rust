//! FastEdge cache storage (ProxyWasm API).
//!
//! This module provides an interface for the ephemeral cache, which is implemented by the host.
//! It is the ProxyWasm counterpart of the Component Model [`fastedge::cache`](crate::cache)
//! module and mirrors the `cache-sync` WIT interface.
//!
//! An example of using the FastEdge cache looks like:
//!
//! ```
//! use fastedge::proxywasm::cache;
//! use proxy_wasm::traits::*;
//! use proxy_wasm::types::*;
//!
//! proxy_wasm::main! {{
//!     proxy_wasm::set_log_level(LogLevel::Trace);
//!     proxy_wasm::set_root_context(|_| -> Box<dyn RootContext> { Box::new(HttpBodyRoot) });
//! }}
//!
//! struct HttpBodyRoot;
//!
//! impl Context for HttpBodyRoot {}
//!
//! impl RootContext for HttpBodyRoot {
//!     fn get_type(&self) -> Option<ContextType> {
//!         Some(ContextType::HttpContext)
//!     }
//!
//!     fn create_http_context(&self, _: u32) -> Option<Box<dyn HttpContext>> {
//!         Some(Box::new(HttpBody))
//!     }
//! }
//!
//! struct HttpBody;
//!
//! impl Context for HttpBody {}
//!
//! impl HttpContext for HttpBody {
//!     fn on_http_response_headers(&mut self, _: usize, _: bool) -> Action {
//!
//!         let Ok(cached) = cache::get("key-3338664") else {
//!             return Action::Pause;
//!         };
//!
//!         if cached.is_none() {
//!             // store the value for 5 minutes
//!             let _ = cache::set("key-3338664", b"value", Some(300_000));
//!         }
//!
//!         Action::Continue
//!     }
//! }
//! ```
//!

use std::fmt::Display;
use std::ptr::null_mut;

/// The set of errors which may be raised by functions in this interface
#[derive(Debug, Clone)]
pub enum Error {
    /// The requesting component does not have access to the specified cache
    /// (which may or may not exist).
    AccessDenied,
    /// An unexpected internal error occurred.
    InternalError,
    /// Some implementation-specific error has occurred (e.g. I/O)
    Other(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::AccessDenied => write!(f, "access denied"),
            Error::InternalError => write!(f, "internal error"),
            Error::Other(msg) => write!(f, "other error: {}", msg),
        }
    }
}

/// Maps a non-success host status code onto an [`Error`].
///
/// Codes follow the ProxyWasm status enum the host reports: `2` (bad argument)
/// carries access denial, as it does for `key_value`, and `10` is an internal
/// failure.
fn error_from_status(status: u32) -> Error {
    match status {
        2 => Error::AccessDenied,
        10 => Error::InternalError,
        status => Error::Other(format!("unexpected status: {}", status)),
    }
}

/// Get the value associated with `key`.
///
/// Returns `Ok(None)` if the key does not exist.
pub fn get(key: &str) -> Result<Option<Vec<u8>>, Error> {
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;

    unsafe {
        match super::proxy_cache_get(
            key.as_ptr(),
            key.len(),
            &mut return_data,
            &mut return_size,
        ) {
            0 => {
                if !return_data.is_null() {
                    Ok(Some(Vec::from_raw_parts(
                        return_data,
                        return_size,
                        return_size,
                    )))
                } else {
                    Ok(None)
                }
            }
            1 => Ok(None),
            status => Err(error_from_status(status)),
        }
    }
}

/// Set the value for `key` with an optional expiry.
///
/// If the key already exists, its current value is overwritten.
/// If the key does not exist, a new key-value pair is created.
///
/// `ttl_ms` is the time-to-live in milliseconds. Pass `None` for no expiry.
pub fn set(key: &str, value: &[u8], ttl_ms: Option<u64>) -> Result<(), Error> {
    unsafe {
        // the host treats a zero time-to-live as "no expiry"
        match super::proxy_cache_set(
            key.as_ptr(),
            key.len(),
            value.as_ptr(),
            value.len(),
            ttl_ms.unwrap_or(0),
        ) {
            0 => Ok(()),
            status => Err(error_from_status(status)),
        }
    }
}

/// Delete the key-value pair associated with `key`.
///
/// If the key does not exist, this operation is a no-op.
pub fn delete(key: &str) -> Result<(), Error> {
    unsafe {
        match super::proxy_cache_delete(key.as_ptr(), key.len()) {
            0 | 1 => Ok(()),
            status => Err(error_from_status(status)),
        }
    }
}

/// Check whether `key` exists in the cache.
///
/// Returns `Ok(true)` if the key exists, `Ok(false)` otherwise.
pub fn exists(key: &str) -> Result<bool, Error> {
    let mut return_exists: u32 = 0;

    unsafe {
        match super::proxy_cache_exists(key.as_ptr(), key.len(), &mut return_exists) {
            0 => Ok(return_exists != 0),
            1 => Ok(false),
            status => Err(error_from_status(status)),
        }
    }
}

/// Increment the integer value stored at `key` by `delta`.
///
/// If the key does not exist, it is initialised to `0` before incrementing.
/// The operation is atomic. `delta` may be negative to decrement.
///
/// Returns the new value after the increment, or an error if the operation fails
/// (for example, if the stored value is not an integer).
pub fn incr(key: &str, delta: i64) -> Result<i64, Error> {
    let mut return_value: i64 = 0;

    unsafe {
        match super::proxy_cache_incr(key.as_ptr(), key.len(), delta, &mut return_value) {
            0 => Ok(return_value),
            status => Err(error_from_status(status)),
        }
    }
}

/// Set or update the expiry of `key` to `ttl_ms` milliseconds from now.
///
/// Returns `Ok(false)` if the key does not exist, `Ok(true)` if the expiry was
/// updated successfully.
pub fn expire(key: &str, ttl_ms: u64) -> Result<bool, Error> {
    let mut return_updated: u32 = 0;

    unsafe {
        match super::proxy_cache_expire(key.as_ptr(), key.len(), ttl_ms, &mut return_updated) {
            0 => Ok(return_updated != 0),
            1 => Ok(false),
            status => Err(error_from_status(status)),
        }
    }
}

/// Purge all cache entries owned by the calling application.
///
/// The host scans the application's key index, deletes every cached key,
/// and then removes the index itself.
///
/// Returns the number of keys that were deleted.
pub fn purge() -> Result<u64, Error> {
    let mut return_count: u64 = 0;

    unsafe {
        match super::proxy_cache_purge(&mut return_count) {
            0 => Ok(return_count),
            status => Err(error_from_status(status)),
        }
    }
}

/// Purge all cache entries whose keys begin with `prefix`.
///
/// The host scans the application's key index for keys that begin with the
/// given prefix, deletes every matched key, and removes the matched entries
/// from the index (the index itself is kept for any remaining keys).
///
/// Returns the number of keys that were deleted.
pub fn purge_prefix(prefix: &str) -> Result<u64, Error> {
    let mut return_count: u64 = 0;

    unsafe {
        match super::proxy_cache_purge_prefix(prefix.as_ptr(), prefix.len(), &mut return_count) {
            0 => Ok(return_count),
            status => Err(error_from_status(status)),
        }
    }
}
