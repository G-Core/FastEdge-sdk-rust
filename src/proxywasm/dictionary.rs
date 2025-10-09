//! FastEdge dictionary storage
//!
//! This module provides an interface for dictionary storage, which is implemented by the host.

use std::ptr::null_mut;

/// Returns a dictionary value of the given key.
/// If the value does not exist returns `None`.
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
