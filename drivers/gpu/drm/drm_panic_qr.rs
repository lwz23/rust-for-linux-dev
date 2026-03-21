// SPDX-License-Identifier: GPL-2.0 OR MIT

//! Blind-first Rust QR generator entry points for DRM panic.
//!
//! Reference C component: [`drivers/gpu/drm/drm_panic.c`](./drm_panic.c)

use kernel::{drm::panic_qr, prelude::*};

#[export]
pub extern "C" fn drm_panic_qr_max_data_size(version: u8, url_len: usize) -> usize {
    panic_qr::max_data_size(version, url_len)
}

#[export]
pub extern "C" fn drm_panic_qr_generate(
    url: *const c_char,
    data: *mut u8,
    data_len: usize,
    data_size: usize,
    tmp: *mut u8,
    tmp_size: usize,
) -> u8 {
    panic_qr::generate_from_raw(url, data, data_len, data_size, tmp, tmp_size)
}
