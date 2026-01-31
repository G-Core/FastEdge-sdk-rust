//! FastEdge utility functions (ProxyWasm API).
//!
//! This module provides diagnostic and monitoring utilities through the ProxyWasm FFI interface.
//!
//! # Examples
//!
//! ```no_run
//! use fastedge::proxywasm::utils;
//!
//! // Log diagnostic information
//! utils::set_user_diag("Processing started for user 12345");
//! ```

/// Sets a diagnostic message for debugging and monitoring.
///
/// This function allows you to log custom diagnostic messages that can be viewed
/// in the FastEdge platform logs. Use it for debugging, monitoring, and tracking
/// application behavior.
///
/// # Arguments
///
/// * `value` - The diagnostic message to log
///
/// # Panics
///
/// Panics if the operation fails (non-zero status code from host).
///
/// # Examples
///
/// ```no_run
/// use fastedge::proxywasm::utils;
///
/// utils::set_user_diag("Request processing completed successfully");
///
/// let item_count = 42;
/// utils::set_user_diag(&format!("Processed {} items", item_count));
/// ```
///
/// # Note
///
/// Diagnostic messages should be used judiciously as they may impact performance.
/// Avoid logging sensitive information such as passwords or API keys.
pub fn set_user_diag(value: &str) {
    unsafe {
        let status = super::stats_set_user_diag(value.as_ptr(), value.len());
        if status != 0 {
            panic!("unexpected status: {}", status)
        }
    }
}
