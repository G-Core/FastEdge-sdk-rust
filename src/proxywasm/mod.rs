//! ProxyWasm compatibility layer for FastEdge.
//!
//! This module provides a ProxyWasm-compatible subset of the FastEdge Component Model APIs
//! for applications that need to run in ProxyWasm environment. It
//! currently exposes key-value, cache, secret, dictionary, and related utility operations via
//! FFI (Foreign Function Interface) calls.
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
//! - [`cache`]: Ephemeral cache operations
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
pub mod cache;
pub mod secret;
pub mod dictionary;
pub mod utils;

#[link(wasm_import_module = "env")]
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

    fn proxy_cache_get(
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_cache_set(
        key_data: *const u8,
        key_size: usize,
        value_data: *const u8,
        value_size: usize,
        ttl_ms: u64,
    ) -> u32;

    fn proxy_cache_delete(key_data: *const u8, key_size: usize) -> u32;

    fn proxy_cache_exists(
        key_data: *const u8,
        key_size: usize,
        return_exists: *mut u32,
    ) -> u32;

    fn proxy_cache_incr(
        key_data: *const u8,
        key_size: usize,
        delta: i64,
        return_value: *mut i64,
    ) -> u32;

    fn proxy_cache_expire(
        key_data: *const u8,
        key_size: usize,
        ttl_ms: u64,
        return_updated: *mut u32,
    ) -> u32;

    fn proxy_cache_purge(return_count: *mut u64) -> u32;

    fn proxy_cache_purge_prefix(
        prefix_data: *const u8,
        prefix_size: usize,
        return_count: *mut u64,
    ) -> u32;

    fn stats_set_user_diag(
        value_data: *const u8,
        value_size: usize,
    ) -> u32;
}
