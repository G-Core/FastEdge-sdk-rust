//! FastEdge dictionary storage (ProxyWasm API).
//!
//! This module provides dictionary access through the ProxyWasm FFI interface.
//! Dictionaries provide fast, read-only key-value lookups for configuration data.
//!
//! # Examples
//!
//! ```no_run
//! use fastedge::proxywasm::dictionary;
//!
//! // Get a configuration value
//! if let Some(endpoint) = dictionary::get("api_endpoint") {
//!     println!("API endpoint: {}", endpoint);
//! }
//! ```

use std::ptr::null_mut;

/// Retrieves a string value from the dictionary by key.
///
/// # Arguments
///
/// * `key` - The key to look up in the dictionary
///
/// # Returns
///
/// Returns `Some(value)` if the key exists and the value is valid UTF-8,
/// or `None` if the key doesn't exist or the value is not valid UTF-8.
///
/// # Panics
///
/// Panics if an unexpected status code is returned from the host.
///
/// # Examples
///
/// ```no_run
/// use fastedge::proxywasm::dictionary;
///
/// if let Some(timeout) = dictionary::get("timeout_ms") {
///     println!("Timeout: {} ms", timeout);
/// }
/// ```
pub fn get(key: &str) -> Option<String> {
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    unsafe {
        match super::proxy_dictionary_get(key.as_ptr(), key.len(), &mut return_data, &mut return_size) {
            0 => {
                if !return_data.is_null() {
                    let data = Vec::from_raw_parts(
                        return_data,
                        return_size,
                        return_size,
                    );
                    String::from_utf8(data).ok()
                } else {
                    None
                }
            }
            1 => None,
            status => panic!("unexpected status: {}", status),
        }
    }
}
