// SPDX-License-Identifier: GPL-2.0

//! Safe wrapper over the kernel badblocks helper.
//!
//! C header: [`include/linux/badblocks.h`](srctree/include/linux/badblocks.h)

use crate::{bindings, error, types::Opaque};
use core::sync::atomic::{AtomicI32, Ordering};

/// Thread-safe wrapper around `struct badblocks`.
///
/// The underlying C helpers provide the required synchronization for lookups
/// and updates, so shared references are sufficient for all supported
/// operations.
#[repr(transparent)]
pub struct BadBlocks(Opaque<bindings::badblocks>);

// SAFETY: The underlying C object is internally synchronized for the supported
// badblocks operations.
unsafe impl Send for BadBlocks {}
unsafe impl Sync for BadBlocks {}

impl BadBlocks {
    /// Creates a new empty badblocks tracker.
    pub fn new() -> error::Result<Self> {
        let inner = Opaque::new(unsafe { core::mem::zeroed::<bindings::badblocks>() });

        // SAFETY: `inner` is a valid zero-initialized allocation for the C
        // object, and the helper initializes it.
        error::to_result(unsafe { bindings::badblocks_init(inner.get(), 0) })?;

        Ok(Self(inner))
    }

    #[inline]
    fn shift(&self) -> &AtomicI32 {
        // SAFETY: `shift` lives for as long as `self`, and the wrapper only
        // exposes shared access patterns that match the C helper usage.
        unsafe { &*core::ptr::addr_of!((*self.0.get()).shift).cast::<AtomicI32>() }
    }

    /// Returns whether badblocks accounting is enabled.
    #[inline]
    pub fn is_enabled(&self) -> bool {
        self.shift().load(Ordering::Acquire) >= 0
    }

    /// Renders the current badblocks contents into `page`.
    pub fn show(&self, page: &mut [u8; bindings::PAGE_SIZE]) -> error::Result<usize> {
        // SAFETY: `self` and `page` are both valid for the C helper.
        let len = unsafe { bindings::badblocks_show(self.0.get(), page.as_mut_ptr().cast(), 0) };
        if len < 0 {
            Err(error::Error::from_errno(len as i32))
        } else {
            Ok(len as usize)
        }
    }

    /// Returns `true` when any bad block overlaps the given range.
    pub fn check(&self, start: u64, len: u32) -> bool {
        // `null_blk` only consults badblocks once the feature has been
        // enabled through configfs. Mirror that guard here so callers cannot
        // accidentally trigger the helper's internal WARN_ON on disabled state.
        if !self.is_enabled() || len == 0 {
            return false;
        }

        let mut first_bad = 0u64;
        let mut bad_len = 0i32;
        let len = len as i32;

        // SAFETY: The arguments follow the C helper contract.
        unsafe {
            bindings::badblocks_check(
                self.0.get(),
                start,
                len,
                &mut first_bad,
                &mut bad_len,
            ) != 0
        }
    }

    /// Marks a range as bad.
    pub fn set(&self, start: u64, len: u32) -> error::Result {
        self.enable();
        let len = len as i32;
        // SAFETY: The arguments follow the C helper contract.
        error::to_result(unsafe { bindings::badblocks_set(self.0.get(), start, len, 1) })
    }

    /// Clears a bad range.
    pub fn clear(&self, start: u64, len: u32) -> error::Result {
        self.enable();
        let len = len as i32;
        // SAFETY: The arguments follow the C helper contract.
        error::to_result(unsafe { bindings::badblocks_clear(self.0.get(), start, len) })
    }

    fn enable(&self) {
        // `badblocks.shift` is initialized to `-1`; switching it to `0`
        // enables the feature exactly like the C driver does.
        let _ = self
            .shift()
            .compare_exchange(-1, 0, Ordering::AcqRel, Ordering::Acquire);
    }
}

impl Drop for BadBlocks {
    fn drop(&mut self) {
        // SAFETY: The object was initialized by `badblocks_init`.
        unsafe { bindings::badblocks_exit(self.0.get()) };
    }
}
