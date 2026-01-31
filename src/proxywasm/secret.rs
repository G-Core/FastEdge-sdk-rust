//! FastEdge secret storage (ProxyWasm API).
//!
//! This module provides secure access to encrypted secrets through the ProxyWasm FFI interface.
//! Secrets are encrypted at rest and support versioned retrieval.
//!
//! # Security
//!
//! - Never log or expose secret values in application output
//! - Secrets are only accessible to authorized applications
//! - Use time-based retrieval for secret rotation scenarios
//!
//! # Examples
//!
//! ```no_run
//! use fastedge::proxywasm::secret;
//!
//! // Get current secret value
//! match secret::get("DATABASE_PASSWORD")? {
//!     Some(password) => {
//!         let pwd = String::from_utf8_lossy(&password);
//!         // Use password securely
//!     }
//!     None => {
//!         eprintln!("Secret not configured");
//!     }
//! }
//! # Ok::<(), u32>(())
//! ```

use std::ptr::null_mut;

/// Retrieves the current value of a secret.
///
/// Returns the secret value that is currently effective. If the secret supports
/// versioning, this returns the latest version.
///
/// # Arguments
///
/// * `key` - The name of the secret to retrieve
///
/// # Returns
///
/// Returns `Ok(Some(value))` if the secret exists, `Ok(None)` if not found,
/// or `Err(status)` on failure.
///
/// # Security
///
/// Never log or expose the returned secret value. Handle it securely and
/// clear it from memory when no longer needed.
///
/// # Examples
///
/// ```no_run
/// use fastedge::proxywasm::secret;
///
/// let api_key = secret::get("THIRD_PARTY_API_KEY")?;
/// if let Some(key) = api_key {
///     // Use the key for API authentication
/// }
/// # Ok::<(), u32>(())
/// ```
pub fn get(key: &str) -> Result<Option<Vec<u8>>, u32> {
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    unsafe {
        match super::proxy_secret_get(key.as_ptr(), key.len(), &mut return_data, &mut return_size) {
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

/// Retrieves a secret value effective at a specific timestamp.
///
/// This function is useful for secret rotation scenarios where you need to retrieve
/// a historical version of a secret that was valid at a particular point in time.
///
/// # Arguments
///
/// * `key` - The name of the secret to retrieve
/// * `at` - Unix timestamp (seconds since epoch) for when the secret should be effective
///
/// # Returns
///
/// Returns `Ok(Some(value))` if a secret was effective at that time,
/// `Ok(None)` if no secret was configured, or `Err(status)` on failure.
///
/// # Examples
///
/// ```no_run
/// use fastedge::proxywasm::secret;
/// use std::time::{SystemTime, UNIX_EPOCH};
///
/// // Get the secret that was valid 1 hour ago
/// let one_hour_ago = SystemTime::now()
///     .duration_since(UNIX_EPOCH)
///     .unwrap()
///     .as_secs() as u32 - 3600;
///
/// if let Some(old_secret) = secret::get_effective_at("API_KEY", one_hour_ago)? {
///     // Use the historical secret value
/// }
/// # Ok::<(), u32>(())
/// ```
pub fn get_effective_at(key: &str, at: u32) -> Result<Option<Vec<u8>>, u32> {
    let mut return_data: *mut u8 = null_mut();
    let mut return_size: usize = 0;
    unsafe {
        match super::proxy_secret_get_effective_at(
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
