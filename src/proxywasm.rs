extern "C" {
    fn proxy_get_secret(
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_get_effective_at_secret(
        key_data: *const u8,
        key_size: usize,
        at: u32,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;
}

/// ProxyWasm secret interface
pub mod secret {
    use crate::proxywasm::{proxy_get_secret, proxy_get_effective_at_secret};
    use std::ptr::null_mut;

    /// Returns a secret value to the corresponding key effective now.
    /// If the value does not exist returns `None`.
    pub fn get(key: &str) -> Result<Option<Vec<u8>>, u32> {
        let mut return_data: *mut u8 = null_mut();
        let mut return_size: usize = 0;
        unsafe {
            match proxy_get_secret(
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
                status => panic!("unexpected status: {}", status),
            }
        }
    }

    /// Returns a secret value to the corresponding key effective at given timestamp (in sec).
    /// If the value does not exist returns `None`.
    pub fn get_effective_at(key: &str, at: u32) -> Result<Option<Vec<u8>>, u32> {
        let mut return_data: *mut u8 = null_mut();
        let mut return_size: usize = 0;
        unsafe {
            match proxy_get_effective_at_secret(
                key.as_ptr(),
                key.len(),
                at,
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
                status => panic!("unexpected status: {}", status),
            }
        }
    }
}