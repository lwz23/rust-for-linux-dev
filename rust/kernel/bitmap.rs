// SPDX-License-Identifier: GPL-2.0

// Copyright (C) 2025 Google LLC.

//! Rust API for bitmap.
//!
//! C headers: [`include/linux/bitmap.h`](srctree/include/linux/bitmap.h).

use crate::alloc::{AllocError, Flags};
#[cfg_attr(miri, allow(unused_imports))]
use crate::bindings;
#[cfg(all(not(CONFIG_RUST_BITMAP_HARDENED), not(miri)))]
use crate::pr_err;
#[cfg(miri)]
extern crate alloc as rust_alloc;
#[cfg(miri)]
extern crate c2rust_find_bit;
#[cfg(miri)]
extern crate c2rust_bitmap_helpers;
#[cfg(miri)]
use self::rust_alloc::{boxed::Box, vec};
use core::ptr::NonNull;

/// Represents a C bitmap. Wraps underlying C bitmap API.
///
/// # Invariants
///
/// Must reference a `[c_ulong]` long enough to fit `data.len()` bits.
#[cfg_attr(CONFIG_64BIT, repr(align(8)))]
#[cfg_attr(not(CONFIG_64BIT), repr(align(4)))]
pub struct Bitmap {
    data: [()],
}

impl Bitmap {
    /// Borrows a C bitmap.
    ///
    /// # Safety
    ///
    /// * `ptr` holds a non-null address of an initialized array of `unsigned long`
    ///   that is large enough to hold `nbits` bits.
    /// * the array must not be freed for the lifetime of this [`Bitmap`]
    /// * concurrent access only happens through atomic operations
    pub unsafe fn from_raw<'a>(ptr: *const usize, nbits: usize) -> &'a Bitmap {
        let data: *const [()] = core::ptr::slice_from_raw_parts(ptr.cast(), nbits);
        // INVARIANT: `data` references an initialized array that can hold `nbits` bits.
        // SAFETY:
        // The caller guarantees that `data` (derived from `ptr` and `nbits`)
        // points to a valid, initialized, and appropriately sized memory region
        // that will not be freed for the lifetime 'a.
        // We are casting `*const [()]` to `*const Bitmap`. The `Bitmap`
        // struct is a ZST with a `data: [()]` field. This means its layout
        // is compatible with a slice of `()`, and effectively it's a "thin pointer"
        // (its size is 0 and alignment is 1). The `slice_from_raw_parts`
        // function correctly encodes the length (number of bits, not elements)
        // into the metadata of the fat pointer. Therefore, dereferencing this
        // pointer as `&Bitmap` is safe given the caller's guarantees.
        unsafe { &*(data as *const Bitmap) }
    }

    /// Borrows a C bitmap exclusively.
    ///
    /// # Safety
    ///
    /// * `ptr` holds a non-null address of an initialized array of `unsigned long`
    ///   that is large enough to hold `nbits` bits.
    /// * the array must not be freed for the lifetime of this [`Bitmap`]
    /// * no concurrent access may happen.
    pub unsafe fn from_raw_mut<'a>(ptr: *mut usize, nbits: usize) -> &'a mut Bitmap {
        let data: *mut [()] = core::ptr::slice_from_raw_parts_mut(ptr.cast(), nbits);
        // INVARIANT: `data` references an initialized array that can hold `nbits` bits.
        // SAFETY:
        // The caller guarantees that `data` (derived from `ptr` and `nbits`)
        // points to a valid, initialized, and appropriately sized memory region
        // that will not be freed for the lifetime 'a.
        // Furthermore, the caller guarantees no concurrent access will happen,
        // which upholds the exclusivity requirement for a mutable reference.
        // Similar to `from_raw`, casting `*mut [()]` to `*mut Bitmap` is
        // safe because `Bitmap` is a ZST with a `data: [()]` field,
        // making its layout compatible with a slice of `()`.
        unsafe { &mut *(data as *mut Bitmap) }
    }

    /// Returns a raw pointer to the backing [`Bitmap`].
    pub fn as_ptr(&self) -> *const usize {
        core::ptr::from_ref::<Bitmap>(self).cast::<usize>()
    }

    /// Returns a mutable raw pointer to the backing [`Bitmap`].
    pub fn as_mut_ptr(&mut self) -> *mut usize {
        core::ptr::from_mut::<Bitmap>(self).cast::<usize>()
    }

    /// Returns length of this [`Bitmap`].
    #[expect(clippy::len_without_is_empty)]
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

/// Holds either a pointer to array of `unsigned long` or a small bitmap.
#[repr(C)]
union BitmapRepr {
    bitmap: usize,
    ptr: NonNull<usize>,
}

macro_rules! bitmap_assert {
    ($cond:expr, $($arg:tt)+) => {
        #[cfg(CONFIG_RUST_BITMAP_HARDENED)]
        assert!($cond, $($arg)*);
    }
}

macro_rules! bitmap_assert_return {
    ($cond:expr, $($arg:tt)+) => {
        #[cfg(CONFIG_RUST_BITMAP_HARDENED)]
        assert!($cond, $($arg)*);

        #[cfg(all(not(CONFIG_RUST_BITMAP_HARDENED), miri))]
        if !($cond) {
            return
        }

        #[cfg(all(not(CONFIG_RUST_BITMAP_HARDENED), not(miri)))]
        if !($cond) {
            pr_err!($($arg)*);
            return
        }
    }
}

/// Represents an owned bitmap.
///
/// Wraps underlying C bitmap API. See [`Bitmap`] for available
/// methods.
///
/// # Examples
///
/// Basic usage
///
/// ```
/// use kernel::alloc::flags::GFP_KERNEL;
/// use kernel::bitmap::BitmapVec;
///
/// let mut b = BitmapVec::new(16, GFP_KERNEL)?;
///
/// assert_eq!(16, b.len());
/// for i in 0..16 {
///     if i % 4 == 0 {
///       b.set_bit(i);
///     }
/// }
/// assert_eq!(Some(0), b.next_bit(0));
/// assert_eq!(Some(1), b.next_zero_bit(0));
/// assert_eq!(Some(4), b.next_bit(1));
/// assert_eq!(Some(5), b.next_zero_bit(4));
/// assert_eq!(Some(12), b.last_bit());
/// # Ok::<(), Error>(())
/// ```
///
/// # Invariants
///
/// * `nbits` is `<= MAX_LEN`.
/// * if `nbits <= MAX_INLINE_LEN`, then `repr` is a `usize`.
/// * otherwise, `repr` holds a non-null pointer to an initialized
///   array of `unsigned long` that is large enough to hold `nbits` bits.
pub struct BitmapVec {
    /// Representation of bitmap.
    repr: BitmapRepr,
    /// Length of this bitmap. Must be `<= MAX_LEN`.
    nbits: usize,
}

impl core::ops::Deref for BitmapVec {
    type Target = Bitmap;

    fn deref(&self) -> &Bitmap {
        let ptr = if self.nbits <= BitmapVec::MAX_INLINE_LEN {
            // SAFETY: Bitmap is represented inline.
            #[allow(unused_unsafe, reason = "Safe since Rust 1.92.0")]
            unsafe {
                core::ptr::addr_of!(self.repr.bitmap)
            }
        } else {
            // SAFETY: Bitmap is represented as array of `unsigned long`.
            unsafe { self.repr.ptr.as_ptr() }
        };

        // SAFETY: We got the right pointer and invariants of [`Bitmap`] hold.
        // An inline bitmap is treated like an array with single element.
        unsafe { Bitmap::from_raw(ptr, self.nbits) }
    }
}

impl core::ops::DerefMut for BitmapVec {
    fn deref_mut(&mut self) -> &mut Bitmap {
        let ptr = if self.nbits <= BitmapVec::MAX_INLINE_LEN {
            // SAFETY: Bitmap is represented inline.
            #[allow(unused_unsafe, reason = "Safe since Rust 1.92.0")]
            unsafe {
                core::ptr::addr_of_mut!(self.repr.bitmap)
            }
        } else {
            // SAFETY: Bitmap is represented as array of `unsigned long`.
            unsafe { self.repr.ptr.as_ptr() }
        };

        // SAFETY: We got the right pointer and invariants of [`BitmapVec`] hold.
        // An inline bitmap is treated like an array with single element.
        unsafe { Bitmap::from_raw_mut(ptr, self.nbits) }
    }
}

/// Enable ownership transfer to other threads.
///
/// SAFETY: We own the underlying bitmap representation.
unsafe impl Send for BitmapVec {}

/// Enable unsynchronized concurrent access to [`BitmapVec`] through shared references.
///
/// SAFETY: `deref()` will return a reference to a [`Bitmap`]. Its methods
/// take immutable references are either atomic or read-only.
unsafe impl Sync for BitmapVec {}

impl Drop for BitmapVec {
    fn drop(&mut self) {
        if self.nbits <= BitmapVec::MAX_INLINE_LEN {
            return;
        }
        #[cfg(miri)]
        {
            let words = usize::div_ceil(self.nbits, usize::BITS as usize);
            // SAFETY: Under `cfg(miri)`, `repr.ptr` is created from `Box<[usize]>` in `new`.
            // Reconstruct the boxed slice with the same length so it can be dropped.
            unsafe {
                let data = core::ptr::slice_from_raw_parts_mut(self.repr.ptr.as_ptr(), words);
                drop(Box::from_raw(data));
            }
            return;
        }

        #[cfg(not(miri))]
        {
        // SAFETY: `self.ptr` was returned by the C `bitmap_zalloc`.
        //
        // INVARIANT: there is no other use of the `self.ptr` after this
        // call and the value is being dropped so the broken invariant is
        // not observable on function exit.
        unsafe { bindings::bitmap_free(self.repr.ptr.as_ptr()) };
        }
    }
}

impl BitmapVec {
    /// The maximum possible length of a `BitmapVec`.
    pub const MAX_LEN: usize = i32::MAX as usize;

    /// The maximum length that uses the inline representation.
    pub const MAX_INLINE_LEN: usize = usize::BITS as usize;

    /// Construct a longest possible inline [`BitmapVec`].
    #[inline]
    pub fn new_inline() -> Self {
        // INVARIANT: `nbits <= MAX_INLINE_LEN`, so an inline bitmap is the right repr.
        BitmapVec {
            repr: BitmapRepr { bitmap: 0 },
            nbits: BitmapVec::MAX_INLINE_LEN,
        }
    }

    /// Constructs a new [`BitmapVec`].
    ///
    /// Fails with [`AllocError`] when the [`BitmapVec`] could not be allocated. This
    /// includes the case when `nbits` is greater than `MAX_LEN`.
    #[inline]
    pub fn new(nbits: usize, flags: Flags) -> Result<Self, AllocError> {
        if nbits <= BitmapVec::MAX_INLINE_LEN {
            return Ok(BitmapVec {
                repr: BitmapRepr { bitmap: 0 },
                nbits,
            });
        }
        if nbits > Self::MAX_LEN {
            return Err(AllocError);
        }

        #[cfg(miri)]
        {
            let _ = flags;
            let words = usize::div_ceil(nbits, usize::BITS as usize);
            let boxed = vec![0usize; words].into_boxed_slice();
            let ptr = NonNull::new(Box::into_raw(boxed).cast::<usize>()).ok_or(AllocError)?;
            return Ok(BitmapVec {
                repr: BitmapRepr { ptr },
                nbits,
            });
        }

        #[cfg(not(miri))]
        {
        let nbits_u32 = u32::try_from(nbits).unwrap();
        // SAFETY: `MAX_INLINE_LEN < nbits` and `nbits <= MAX_LEN`.
        let ptr = unsafe { bindings::bitmap_zalloc(nbits_u32, flags.as_raw()) };
        let ptr = NonNull::new(ptr).ok_or(AllocError)?;
        // INVARIANT: `ptr` returned by C `bitmap_zalloc` and `nbits` checked.
        Ok(BitmapVec {
            repr: BitmapRepr { ptr },
            nbits,
        })
        }
    }

    /// Returns length of this [`Bitmap`].
    #[allow(clippy::len_without_is_empty)]
    #[inline]
    pub fn len(&self) -> usize {
        self.nbits
    }

    /// Fills this `Bitmap` with random bits.
    #[cfg(CONFIG_FIND_BIT_BENCHMARK_RUST)]
    pub fn fill_random(&mut self) {
        // SAFETY: `self.as_mut_ptr` points to either an array of the
        // appropriate length or one usize.
        unsafe {
            bindings::get_random_bytes(
                self.as_mut_ptr().cast::<ffi::c_void>(),
                usize::div_ceil(self.nbits, bindings::BITS_PER_LONG as usize)
                    * bindings::BITS_PER_LONG as usize
                    / 8,
            );
        }
    }
}

impl Bitmap {
    /// Set bit with index `index`.
    ///
    /// ATTENTION: `set_bit` is non-atomic, which differs from the naming
    /// convention in C code. The corresponding C function is `__set_bit`.
    ///
    /// If CONFIG_RUST_BITMAP_HARDENED is not enabled and `index` is greater than
    /// or equal to `self.nbits`, does nothing.
    ///
    /// # Panics
    ///
    /// Panics if CONFIG_RUST_BITMAP_HARDENED is enabled and `index` is greater than
    /// or equal to `self.nbits`.
    #[inline]
    pub fn set_bit(&mut self, index: usize) {
        bitmap_assert_return!(
            index < self.len(),
            "Bit `index` must be < {}, was {}",
            self.len(),
            index
        );
        

        
        // SAFETY: Bit `index` is within bounds.
        unsafe { bindings::__set_bit(index, self.as_mut_ptr()) };
        
    }

    /// Set bit with index `index`, atomically.
    ///
    /// This is a relaxed atomic operation (no implied memory barriers).
    ///
    /// ATTENTION: The naming convention differs from C, where the corresponding
    /// function is called `set_bit`.
    ///
    /// If CONFIG_RUST_BITMAP_HARDENED is not enabled and `index` is greater than
    /// or equal to `self.len()`, does nothing.
    ///
    /// # Panics
    ///
    /// Panics if CONFIG_RUST_BITMAP_HARDENED is enabled and `index` is greater than
    /// or equal to `self.len()`.
    #[inline]
    pub fn set_bit_atomic(&self, index: usize) {
        bitmap_assert_return!(
            index < self.len(),
            "Bit `index` must be < {}, was {}",
            self.len(),
            index
        );
        #[cfg(miri)]
        {
            let word = index / usize::BITS as usize;
            let bit = index % usize::BITS as usize;
            let words = usize::div_ceil(self.len(), usize::BITS as usize);
            // SAFETY: this is a miri-only emulation; tests use it single-threaded.
            unsafe {
                let data = core::slice::from_raw_parts_mut(self.as_ptr().cast_mut(), words);
                data[word] |= 1usize << bit;
            }
            return;
        }

        #[cfg(not(miri))]
        {
        // SAFETY: `index` is within bounds and the caller has ensured that
        // there is no mix of non-atomic and atomic operations.
        unsafe { bindings::set_bit(index, self.as_ptr().cast_mut()) };
        }
    }

    /// Clear `index` bit.
    ///
    /// ATTENTION: `clear_bit` is non-atomic, which differs from the naming
    /// convention in C code. The corresponding C function is `__clear_bit`.
    ///
    /// If CONFIG_RUST_BITMAP_HARDENED is not enabled and `index` is greater than
    /// or equal to `self.len()`, does nothing.
    ///
    /// # Panics
    ///
    /// Panics if CONFIG_RUST_BITMAP_HARDENED is enabled and `index` is greater than
    /// or equal to `self.len()`.
    #[inline]
    pub fn clear_bit(&mut self, index: usize) {
        bitmap_assert_return!(
            index < self.len(),
            "Bit `index` must be < {}, was {}",
            self.len(),
            index
        );
        

        
        // SAFETY: `index` is within bounds.
        unsafe { bindings::__clear_bit(index, self.as_mut_ptr()) };
        
    }

    /// Clear `index` bit, atomically.
    ///
    /// This is a relaxed atomic operation (no implied memory barriers).
    ///
    /// ATTENTION: The naming convention differs from C, where the corresponding
    /// function is called `clear_bit`.
    ///
    /// If CONFIG_RUST_BITMAP_HARDENED is not enabled and `index` is greater than
    /// or equal to `self.len()`, does nothing.
    ///
    /// # Panics
    ///
    /// Panics if CONFIG_RUST_BITMAP_HARDENED is enabled and `index` is greater than
    /// or equal to `self.len()`.
    #[inline]
    pub fn clear_bit_atomic(&self, index: usize) {
        bitmap_assert_return!(
            index < self.len(),
            "Bit `index` must be < {}, was {}",
            self.len(),
            index
        );
        #[cfg(miri)]
        {
            let word = index / usize::BITS as usize;
            let bit = index % usize::BITS as usize;
            let words = usize::div_ceil(self.len(), usize::BITS as usize);
            // SAFETY: this is a miri-only emulation; tests use it single-threaded.
            unsafe {
                let data = core::slice::from_raw_parts_mut(self.as_ptr().cast_mut(), words);
                data[word] &= !(1usize << bit);
            }
            return;
        }

        #[cfg(not(miri))]
        {
        // SAFETY: `index` is within bounds and the caller has ensured that
        // there is no mix of non-atomic and atomic operations.
        unsafe { bindings::clear_bit(index, self.as_ptr().cast_mut()) };
        }
    }

    /// Copy `src` into this [`Bitmap`] and set any remaining bits to zero.
    ///
    /// # Examples
    ///
    /// ```
    /// use kernel::alloc::{AllocError, flags::GFP_KERNEL};
    /// use kernel::bitmap::BitmapVec;
    ///
    /// let mut long_bitmap = BitmapVec::new(256, GFP_KERNEL)?;
    ///
    /// assert_eq!(None, long_bitmap.last_bit());
    ///
    /// let mut short_bitmap = BitmapVec::new(16, GFP_KERNEL)?;
    ///
    /// short_bitmap.set_bit(7);
    /// long_bitmap.copy_and_extend(&short_bitmap);
    /// assert_eq!(Some(7), long_bitmap.last_bit());
    ///
    /// # Ok::<(), AllocError>(())
    /// ```
    #[inline]
    pub fn copy_and_extend(&mut self, src: &Bitmap) {
        let len = core::cmp::min(src.len(), self.len());
        

        
        // SAFETY: access to `self` and `src` is within bounds.
        unsafe {
            bindings::bitmap_copy_and_extend(
                self.as_mut_ptr(),
                src.as_ptr(),
                len as u32,
                self.len() as u32,
            )
        };
        
    }

    /// Finds last set bit.
    ///
    /// # Examples
    ///
    /// ```
    /// use kernel::alloc::{AllocError, flags::GFP_KERNEL};
    /// use kernel::bitmap::BitmapVec;
    ///
    /// let bitmap = BitmapVec::new(64, GFP_KERNEL)?;
    ///
    /// match bitmap.last_bit() {
    ///     Some(idx) => {
    ///         pr_info!("The last bit has index {idx}.\n");
    ///     }
    ///     None => {
    ///         pr_info!("All bits in this bitmap are 0.\n");
    ///     }
    /// }
    /// # Ok::<(), AllocError>(())
    /// ```
    #[inline]
    pub fn last_bit(&self) -> Option<usize> {
        

        
        // SAFETY: `_find_next_bit` access is within bounds due to invariant.
        let index = unsafe { bindings::_find_last_bit(self.as_ptr(), self.len()) };
        if index >= self.len() {
            None
        } else {
            Some(index)
        }
        
    }

    /// Finds next set bit, starting from `start`.
    ///
    /// Returns `None` if `start` is greater or equal to `self.nbits`.
    #[inline]
    pub fn next_bit(&self, start: usize) -> Option<usize> {
        bitmap_assert!(
            start < self.len(),
            "`start` must be < {} was {}",
            self.len(),
            start
        );
        // SAFETY: `_find_next_bit` tolerates out-of-bounds arguments and returns a
        // value larger than or equal to `self.len()` in that case.
        // Under `cfg(miri)` the symbol is the C2Rust translation of `lib/find_bit.c`,
        // linked from crate `c2rust_find_bit`.
        let index = unsafe { bindings::_find_next_bit(self.as_ptr(), self.len(), start) };
        if index >= self.len() {
            None
        } else {
            Some(index)
        }
    }

    /// Finds next zero bit, starting from `start`.
    /// Returns `None` if `start` is greater than or equal to `self.len()`.
    #[inline]
    pub fn next_zero_bit(&self, start: usize) -> Option<usize> {
        bitmap_assert!(
            start < self.len(),
            "`start` must be < {} was {}",
            self.len(),
            start
        );
        

        
        // SAFETY: `_find_next_zero_bit` tolerates out-of-bounds arguments and returns a
        // value larger than or equal to `self.len()` in that case.
        let index = unsafe { bindings::_find_next_zero_bit(self.as_ptr(), self.len(), start) };
        if index >= self.len() {
            None
        } else {
            Some(index)
        }
        
    }
}

#[cfg(any(CONFIG_RUST_BITMAP_KUNIT_TEST, miri))]
#[cfg_attr(not(miri), macros::kunit_tests(rust_kernel_bitmap))]
mod tests {
    use super::*;
    use kernel::alloc::flags::GFP_KERNEL;

    #[test]
    fn bitmap_borrow() {
        let fake_bitmap: [usize; 2] = [0, 0];
        let fake_bitmap_len = 2 * usize::BITS as usize;
        // SAFETY: `fake_c_bitmap` is an array of expected length.
        let b = unsafe { Bitmap::from_raw(fake_bitmap.as_ptr(), fake_bitmap_len) };
        assert_eq!(fake_bitmap_len, b.len());
        assert_eq!(None, b.next_bit(0));
    }

    #[test]
    fn bitmap_copy() {
        let fake_bitmap: usize = 0xFF;
        // SAFETY: `fake_c_bitmap` can be used as one-element array of expected length.
        let b = unsafe { Bitmap::from_raw(core::ptr::addr_of!(fake_bitmap), 8) };
        assert_eq!(8, b.len());
        assert_eq!(None, b.next_zero_bit(0));
    }

    #[test]
    fn bitmap_vec_new() -> Result<(), AllocError> {
        let b = BitmapVec::new(0, GFP_KERNEL)?;
        assert_eq!(0, b.len());

        let b = BitmapVec::new(3, GFP_KERNEL)?;
        assert_eq!(3, b.len());

        let b = BitmapVec::new(1024, GFP_KERNEL)?;
        assert_eq!(1024, b.len());

        // Requesting too large values results in [`AllocError`].
        let res = BitmapVec::new(1 << 31, GFP_KERNEL);
        assert!(res.is_err());
        Ok(())
    }

    #[test]
    fn bitmap_set_clear_find() -> Result<(), AllocError> {
        let mut b = BitmapVec::new(128, GFP_KERNEL)?;

        // Zero-initialized
        assert_eq!(None, b.next_bit(0));
        assert_eq!(Some(0), b.next_zero_bit(0));
        assert_eq!(None, b.last_bit());

        b.set_bit(17);

        assert_eq!(Some(17), b.next_bit(0));
        assert_eq!(Some(17), b.next_bit(17));
        assert_eq!(None, b.next_bit(18));
        assert_eq!(Some(17), b.last_bit());

        b.set_bit(107);

        assert_eq!(Some(17), b.next_bit(0));
        assert_eq!(Some(17), b.next_bit(17));
        assert_eq!(Some(107), b.next_bit(18));
        assert_eq!(Some(107), b.last_bit());

        b.clear_bit(17);

        assert_eq!(Some(107), b.next_bit(0));
        assert_eq!(Some(107), b.last_bit());
        Ok(())
    }

    #[test]
    fn owned_bitmap_out_of_bounds() -> Result<(), AllocError> {
        // TODO: Kunit #[test]s do not support `cfg` yet,
        // so we add it here in the body.
        #[cfg(not(CONFIG_RUST_BITMAP_HARDENED))]
        {
            let mut b = BitmapVec::new(128, GFP_KERNEL)?;
            b.set_bit(2048);
            b.set_bit_atomic(2048);
            b.clear_bit(2048);
            b.clear_bit_atomic(2048);
            assert_eq!(None, b.next_bit(2048));
            assert_eq!(None, b.next_zero_bit(2048));
            assert_eq!(None, b.last_bit());
        }
        Ok(())
    }

    // TODO: uncomment once kunit supports [should_panic] and `cfg`.
    // #[cfg(CONFIG_RUST_BITMAP_HARDENED)]
    // #[test]
    // #[should_panic]
    // fn owned_bitmap_out_of_bounds() -> Result<(), AllocError> {
    //     let mut b = BitmapVec::new(128, GFP_KERNEL)?;
    //
    //     b.set_bit(2048);
    // }

    #[test]
    fn bitmap_copy_and_extend() -> Result<(), AllocError> {
        let mut long_bitmap = BitmapVec::new(256, GFP_KERNEL)?;

        long_bitmap.set_bit(3);
        long_bitmap.set_bit(200);

        let mut short_bitmap = BitmapVec::new(32, GFP_KERNEL)?;

        short_bitmap.set_bit(17);

        long_bitmap.copy_and_extend(&short_bitmap);

        // Previous bits have been cleared.
        assert_eq!(Some(17), long_bitmap.next_bit(0));
        assert_eq!(Some(17), long_bitmap.last_bit());
        Ok(())
    }

    #[test]
    fn bitmap_find_next_matrix() {
        use crate::bindings;
        // One input set for the native UML C function and the C2Rust function.
        // `next_bit` is used only when `start < nbits`; the C function is also
        // called directly for `start >= nbits` and `nbits == 0`.
        const SEED: u32 = 0x00C2_F1B1;
        const EXPECTED_CALLS: u64 = 303;
        const EXPECTED_SUM: u64 = 11143140604619718100;
        let mut calls = 0u64;
        let mut sum = 0u64;
        let mut unchanged = true;
        let mut rng = SEED;

        let mut mix = |nbits: usize, start: usize, got: usize| {
            calls += 1;
            sum = sum
                .wrapping_mul(1_315_423_911)
                .wrapping_add(got as u64)
                .wrapping_add((nbits as u64) << 20)
                .wrapping_add(start as u64);
        };

        let mut run = |words: &mut [usize; 4], nbits: usize, start: usize, api: bool| {
            let before = *words;
            let got = if api {
                let bitmap = unsafe { Bitmap::from_raw(words.as_ptr(), nbits) };
                match bitmap.next_bit(start) {
                    Some(index) => index,
                    None => nbits,
                }
            } else {
                unsafe { bindings::_find_next_bit(words.as_ptr(), nbits, start) }
            };
            if *words != before {
                unchanged = false;
            }
            mix(nbits, start, got);
        };

        let fixed = [
            [0usize, 0, 0, 0],
            [usize::MAX, usize::MAX, usize::MAX, usize::MAX],
            [1, 0, 0, 0],
            [1usize << 63, 0, 0, 0],
            [0, 1, 0, 0],
            [0, 1usize << 63, 0, 0],
            [
                0xAAAA_AAAA_AAAA_AAAA,
                0x5555_5555_5555_5555,
                0xAAAA_AAAA_AAAA_AAAA,
                0x5555_5555_5555_5555,
            ],
        ];
        let lengths = [64usize, 65, 128];
        for pattern in fixed {
            for nbits in lengths {
                let mut words = pattern;
                let mut starts = [0usize, 1, 63, 64, 0, 0, 0];
                starts[4] = nbits - 1;
                starts[5] = nbits;
                starts[6] = nbits + 1;
                for start in starts {
                    if start < nbits {
                        run(&mut words, nbits, start, true);
                    }
                    run(&mut words, nbits, start, false);
                }
            }
        }

        let mut empty = [0usize; 4];
        run(&mut empty, 0, 0, false);
        run(&mut empty, 0, 1, false);

        for _ in 0..4 {
            let mut words = [0usize; 4];
            for slot in &mut words {
                rng ^= rng << 13;
                rng ^= rng >> 17;
                rng ^= rng << 5;
                *slot = rng as usize | ((rng as usize) << 32);
                rng ^= rng << 13;
                rng ^= rng >> 17;
                rng ^= rng << 5;
                *slot ^= (rng as usize) << 17;
            }
            for nbits in [64usize, 128] {
                for start in [0usize, 17, 64, 100] {
                    if start < nbits {
                        run(&mut words, nbits, start, true);
                    }
                    run(&mut words, nbits, start, false);
                }
            }
        }

        #[cfg(not(miri))]
        pr_err!("MATRIX calls={} sum={} unchanged={}\n", calls, sum, unchanged as u8);
        assert!(unchanged);
        assert_eq!(calls, EXPECTED_CALLS);
        assert_eq!(sum, EXPECTED_SUM);
    }

}
