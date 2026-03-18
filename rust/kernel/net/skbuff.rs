// SPDX-License-Identifier: GPL-2.0

//! Network packet buffers.
//!
//! C headers: [`include/linux/skbuff.h`](srctree/include/linux/skbuff.h).

use crate::bindings;
use core::ptr::NonNull;

/// An owned socket buffer.
///
/// # Invariants
///
/// The inner pointer is valid and uniquely owned by this object. Dropping the wrapper releases the
/// skb with `consume_skb()`.
pub struct SkBuff(NonNull<bindings::sk_buff>);

impl SkBuff {
    /// Creates an owned [`SkBuff`] from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null, valid, and represent ownership of an skb that Rust is now
    /// responsible for releasing.
    #[allow(dead_code)]
    pub(crate) unsafe fn from_raw(ptr: *mut bindings::sk_buff) -> Self {
        Self(NonNull::new(ptr).expect("sk_buff pointer must be non-null"))
    }

    /// Returns the packet length.
    pub fn len(&self) -> u32 {
        // SAFETY: The type invariant guarantees that the pointer is valid while `self` is alive.
        unsafe { self.0.as_ref().len }
    }

}

impl Drop for SkBuff {
    fn drop(&mut self) {
        // SAFETY: The type invariant guarantees that `self.0` owns a valid skb.
        unsafe { bindings::consume_skb(self.0.as_ptr()) };
    }
}
