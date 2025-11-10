//! FastEdge key-value persistent storage
//!
//! This module provides an interface for key-value storage, which is implemented by the host.
//!
//! An example of using FastEdge Key-Value store looks like:
//!
//! ```
//! use fastedge::proxywasm::key_value::Store;
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
//!         let Ok(store) = Store::open("default") else {
//!             return Action::Pause;
//!         };
//!
//!         let Ok(r) = store.get("key-3338664") else {
//!             return Action::Pause;
//!         };
//!
//!         Action::Continue
//!     }
//! }
//! ```
//!

use std::fmt::Display;
use crate::utils;
use std::ptr::null_mut;

/// The set of errors which may be raised by functions in this interface
#[derive(Debug, Clone)]
pub enum Error {
    /// The host does not recognize the store label requested.
    NoSuchStore,
    /// The requesting component does not have access to the specified store
    /// (which may or may not exist).
    AccessDenied,
    /// Some implementation-specific error has occurred (e.g. I/O)
    Other(String),
}

pub struct Store {
    handle: u32,
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::NoSuchStore => write!(f, "no such store"),
            Error::AccessDenied => write!(f, "access denied"),
            Error::Other(msg) => write!(f, "other error: {}", msg),
        }
    }
}

impl Store {
    /// Open the default store.
    pub fn new() -> Result<Self, Error> {
        Self::open("default")
    }

    /// Open the store with the specified name.
    ///
    /// `error::no-such-store` will be raised if the `name` is not recognized.
    pub fn open(name: &str) -> Result<Self, Error> {
        let mut return_handler = 0;
        unsafe {
            match super::proxy_kv_store_open(name.as_ptr(), name.len(), &mut return_handler) {
                0 => Ok(Store {
                    handle: return_handler,
                }),
                1 => Err(Error::NoSuchStore),
                2 => Err(Error::AccessDenied),
                status => Err(Error::Other(format!("unexpected status: {}", status))),
            }
        }
    }

    /// Get the value associated with the specified `key`
    ///
    /// Returns `ok(none)` if the key does not exist.
    pub fn get(&self, key: &str) -> Result<Option<Vec<u8>>, Error> {
        let mut return_data: *mut u8 = null_mut();
        let mut return_size: usize = 0;

        unsafe {
            match super::proxy_kv_store_get(
                self.handle,
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
                status => Err(Error::Other(format!("unexpected status: {}", status))),
            }
        }
    }

    /// Get all the elements with score from the sorted set at `key` with a f64 score between min and max
    /// (including elements with score equal to min or max). The elements are considered to be ordered from low to high
    /// scores.
    ///
    /// Returns empty `Vec` if the key does not exist or min and max are out of score.
    pub fn zrange_by_score(&self, key: &str, min: f64, max: f64) -> Result<Vec<(Vec<u8>, f64)>, Error> {
        let mut return_data: *mut u8 = null_mut();
        let mut return_size: usize = 0;

        unsafe {
            match super::proxy_kv_store_zrange_by_score(
                self.handle,
                key.as_ptr(),
                key.len(),
                min,
                max,
                &mut return_data,
                &mut return_size,
            ) {
                0 => {
                    if !return_data.is_null() {
                        let data = Vec::from_raw_parts(return_data, return_size, return_size);

                        let data: Vec<(Vec<u8>, f64)> = utils::deserialize_list(&data)
                            .into_iter()
                            .map(|v| {
                                let mut value = v.to_vec();
                                let sz = size_of::<f64>();
                                if value.len() > sz {
                                    let npos = value.len() - sz;
                                    let score = value.split_off(npos);
                                    let score = f64::from_le_bytes(
                                        <[u8; 8]>::try_from(&score[0..sz]).expect("Failed to convert score bytes to f64: expected 8 bytes"),
                                    );
                                    (value, score)
                                } else {
                                    // return an empty vector and 0.0 score if deserialization fails
                                    // empty key should never happen
                                    (vec![], 0.0)
                                }
                            })
                            .collect();
                        Ok(data)
                    } else {
                        Ok(vec![])
                    }
                }
                status => Err(Error::Other(format!("unexpected status: {}", status))),
            }
        }
    }

    /// Interface to scan over keys in the store.
    /// It matches glob-style pattern filter on each element from the retrieved collection.
    ///
    /// Returns an array of elements as a list of keys.
    pub fn scan(&self, pattern: &str) -> Result<Vec<String>, Error> {
        let mut return_data: *mut u8 = null_mut();
        let mut return_size: usize = 0;

        unsafe {
            match super::proxy_kv_store_scan(
                self.handle,
                pattern.as_ptr(),
                pattern.len(),
                &mut return_data,
                &mut return_size,
            ) {
                0 => {
                    if !return_data.is_null() {
                        let data = Vec::from_raw_parts(return_data, return_size, return_size);

                        let data: Vec<String> = utils::deserialize_list(&data)
                            .into_iter()
                            .map(|v| String::from_utf8_lossy(v).to_string())
                            .collect();
                        Ok(data)
                    } else {
                        Ok(vec![])
                    }
                }
                status => Err(Error::Other(format!("unexpected status: {}", status))),
            }
        }
    }

    /// Get the values associated with the specified `key` stored in sorted set ordered by f64 score
    ///
    /// Returns empty `Vec` if the key does not exist or min and max are out of index.
    pub fn zscan(&self, key: &str, pattern: &str) -> Result<Vec<(Vec<u8>, f64)>, Error> {
        let mut return_data: *mut u8 = null_mut();
        let mut return_size: usize = 0;

        unsafe {
            match super::proxy_kv_store_zscan(
                self.handle,
                key.as_ptr(),
                key.len(),
                pattern.as_ptr(),
                pattern.len(),
                &mut return_data,
                &mut return_size,
            ) {
                0 => {
                    if !return_data.is_null() {
                        let data = Vec::from_raw_parts(return_data, return_size, return_size);

                        let data: Vec<(Vec<u8>, f64)> = utils::deserialize_list(&data)
                            .into_iter()
                            .map(|v| {
                                let mut value = v.to_vec();
                                let sz = size_of::<f64>();
                                if value.len() > sz {
                                    let npos = value.len() - sz;
                                    let score = value.split_off(npos);
                                    let score = f64::from_le_bytes(
                                        <[u8; 8]>::try_from(&score[0..sz]).expect("Failed to convert score bytes to f64: expected 8 bytes"),
                                    );
                                    (value, score)
                                } else {
                                    // return and empty vector and 0.0 score if deserialization fails
                                    // empty key should never happen
                                    (vec![], 0.0)
                                }
                            })
                            .collect();
                        Ok(data)
                    } else {
                        Ok(vec![])
                    }
                }
                status => Err(Error::Other(format!("unexpected status: {}", status))),
            }
        }
    }

    /// Determines whether a given item was added to a Bloom filter.
    ///
    /// Returns one of these replies: 'true' means that, with high probability, item was already added to the filter,
    /// and 'false' means that key does not exist or that item had not been added to the filter.
    pub fn bf_exists(&self, key: &str, item: &str) -> Result<bool, Error> {
        let mut return_handler: u32 = 0;
        unsafe {
            match super::proxy_kv_store_bf_exists(
                self.handle,
                key.as_ptr(),
                key.len(),
                item.as_ptr(),
                item.len(),
                &mut return_handler,
            ) {
                0 => Ok(return_handler != 0),
                status => Err(Error::Other(format!("unexpected status: {}", status))),
            }
        }
    }
}
