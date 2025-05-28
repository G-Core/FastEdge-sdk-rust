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
                1 => Err(Error::NoSuchStore),
                2 => Err(Error::AccessDenied),
                status => Err(Error::Other(format!("unexpected status: {}", status))),
            }
        }
    }

    /// Get the values associated with the specified `key` stored in sorted set orderd by u32 index
    ///
    /// Returns empty `Vec` if the key does not exist or min and max are out of index.
    pub fn get_by_range(&self, key: &str, min: u32, max: u32) -> Result<Vec<Vec<u8>>, Error> {
        let mut return_data: *mut u8 = null_mut();
        let mut return_size: usize = 0;

        unsafe {
            match super::proxy_kv_store_get_by_range(
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

                        let data: Vec<Vec<u8>> = utils::deserialize_list(&data)
                            .into_iter()
                            .map(|v| v.to_vec())
                            .collect();
                        Ok(data)
                    } else {
                        Ok(vec![])
                    }
                }
                1 => Err(Error::NoSuchStore),
                2 => Err(Error::AccessDenied),
                status => Err(Error::Other(format!("unexpected status: {}", status))),
            }
        }
    }

    /// Determines whether a given item was added to a Bloom filter.
    ///
    /// Returns one of these replies: 'true' means that, with high probability, item was already added to the filter,
    /// and 'false' means that key does not exist or that item had not been added to the filter.
    pub fn bf_exists(&self, bf: &str, name: &str) -> Result<bool, Error> {
        let mut return_handler: u32 = 0;
        unsafe {
            match super::proxy_kv_store_bf_exists(
                self.handle,
                bf.as_ptr(),
                bf.len(),
                name.as_ptr(),
                name.len(),
                &mut return_handler,
            ) {
                0 => Ok(return_handler != 0),
                1 => Err(Error::NoSuchStore),
                2 => Err(Error::AccessDenied),
                status => Err(Error::Other(format!("unexpected status: {}", status))),
            }
        }
    }
}
