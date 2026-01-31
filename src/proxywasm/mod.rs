//! ProxyWasm compatibility layer for FastEdge.
//!
//! This module provides a ProxyWasm-compatible API for applications that need to run
//! in ProxyWasm environments such as Envoy proxy. It exposes the same functionality
//! as the Component Model API but using FFI (Foreign Function Interface) calls.
//!
//! # Usage
//!
//! Enable the `proxywasm` feature in your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! fastedge = { version = "0.3", features = ["proxywasm"] }
//! ```
//!
//! # Modules
//!
//! - [`key_value`]: Key-value storage operations
//! - [`secret`]: Secret management
//! - [`dictionary`]: Dictionary lookups
//! - [`utils`]: Utility functions
//!
//! # Safety
//!
//! This module uses `unsafe` FFI calls to interact with the ProxyWasm host.
//! The public API is designed to be safe, but incorrect use of internal functions
//! may lead to undefined behavior.

pub mod key_value;
pub mod secret;
pub mod dictionary;
pub mod utils;

extern "C" {
    fn proxy_secret_get(
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_secret_get_effective_at(
        key_data: *const u8,
        key_size: usize,
        at: u32,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_dictionary_get(
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_kv_store_open(key_data: *const u8, key_size: usize, return_handle: *mut u32) -> u32;

    fn proxy_kv_store_get(
        handle: u32,
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_kv_store_zrange_by_score(
        handle: u32,
        key_data: *const u8,
        key_size: usize,
        min: f64,
        max: f64,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_kv_store_scan(
        handle: u32,
        pattern_data: *const u8,
        pattern_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_kv_store_zscan(
        handle: u32,
        key_data: *const u8,
        key_size: usize,
        pattern_data: *const u8,
        pattern_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_kv_store_bf_exists(
        handle: u32,
        key_data: *const u8,
        key_size: usize,
        item_data: *const u8,
        item_size: usize,
        return_handle: *mut u32,
    ) -> u32;

    fn stats_set_user_diag(
        value_data: *const u8,
        value_size: usize,
    ) -> u32;
}
