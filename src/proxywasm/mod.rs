pub mod key_value;
pub mod secret;

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

    fn proxy_kv_store_open(key_data: *const u8, key_size: usize, return_handle: *mut u32) -> u32;

    fn proxy_kv_store_get(
        handle: u32,
        key_data: *const u8,
        key_size: usize,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_kv_store_get_by_range(
        handle: u32,
        key_data: *const u8,
        key_size: usize,
        min: u32,
        max: u32,
        return_value_data: *mut *mut u8,
        return_value_size: *mut usize,
    ) -> u32;

    fn proxy_kv_store_bf_exists(
        handle: u32,
        bf_data: *const u8,
        bf_size: usize,
        key_data: *const u8,
        key_size: usize,
        return_handle: *mut u32
    ) -> u32;
}
