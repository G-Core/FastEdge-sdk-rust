//! This module provides an interface for FastEdge specific handlers, such as setting user diagnostics.
//!

/// Save statistics user diagnostic message.
pub fn set_user_diag(value: &str) {
    unsafe {
        let status = super::stats_set_user_diag(value.as_ptr(), value.len());
        if status != 0 {
            panic!("unexpected status: {}", status)
        }
    }
}
