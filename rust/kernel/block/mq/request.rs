// SPDX-License-Identifier: GPL-2.0

//! This module provides a wrapper for the C `struct request` type.
//!
//! C header: [`include/linux/blk-mq.h`](srctree/include/linux/blk-mq.h)

use crate::{
    bindings,
    block::mq::{BlkStatus, Operations},
    error::{code::EIO, Result},
    types::{ARef, AlwaysRefCounted, ForeignOwnable, Opaque},
};
use core::{
    marker::PhantomData,
    ptr::{addr_of_mut, NonNull},
    sync::atomic::{AtomicU64, Ordering},
};

type ForeignBorrowed<'a, T> = <T as ForeignOwnable>::Borrowed<'a>;

/// Supported request operations exposed to Rust drivers.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum RequestOp {
    /// A read request.
    Read,
    /// A write request.
    Write,
    /// A flush request.
    Flush,
    /// A discard request.
    Discard,
    /// A write-zeroes request.
    WriteZeroes,
    /// An operation value not modeled by this wrapper.
    Unknown(u32),
}

/// Safe wrapper around the blk-mq hardware-context kind.
#[derive(Copy, Clone, Eq, PartialEq)]
pub enum QueueKind {
    /// The default hardware queue map.
    Default,
    /// The dedicated read queue map.
    Read,
    /// The poll queue map.
    Poll,
    /// A queue kind not modeled by this wrapper.
    Unknown(u32),
}

/// A single request segment.
#[derive(Copy, Clone)]
pub struct Segment {
    page: *mut bindings::page,
    len: usize,
    offset: usize,
}

impl Segment {
    /// Returns the segment length in bytes.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns the segment offset within the page.
    pub fn offset(&self) -> usize {
        self.offset
    }

    /// Copies bytes from the segment into `dst`.
    pub fn copy_to_slice(&self, dst: &mut [u8]) -> usize {
        let len = core::cmp::min(dst.len(), self.len);

        // SAFETY: The helper validates and copies exactly `len` bytes from the
        // page region.
        unsafe {
            bindings::kpage_copy_from(
                dst.as_mut_ptr().cast(),
                self.page,
                self.offset,
                len,
            )
        };

        len
    }

    /// Copies bytes from `src` into the segment.
    pub fn copy_from_slice(&self, src: &[u8]) -> usize {
        let len = core::cmp::min(src.len(), self.len);

        // SAFETY: The helper validates and copies exactly `len` bytes into the
        // destination page region.
        unsafe {
            bindings::kpage_copy_to(
                self.page,
                self.offset,
                src.as_ptr().cast(),
                len,
            )
        };

        len
    }

    /// Zeroes the entire segment.
    pub fn zero(&self) {
        // SAFETY: The helper zeroes the page region described by the segment.
        unsafe { bindings::kpage_zero_segment(self.page, self.offset, self.len) };
    }
}

struct SegmentVisit<'a, F> {
    f: &'a mut F,
    result: Result,
}

unsafe extern "C" fn segment_trampoline<F>(
    page: *mut bindings::page,
    len: usize,
    offset: usize,
    data: *mut core::ffi::c_void,
) -> core::ffi::c_int
where
    F: FnMut(Segment) -> Result,
{
    // SAFETY: `data` is a valid `SegmentVisit<F>` for the duration of the
    // helper call.
    let ctx = unsafe { &mut *data.cast::<SegmentVisit<'_, F>>() };

    match (ctx.f)(Segment { page, len, offset }) {
        Ok(()) => 0,
        Err(err) => {
            ctx.result = Err(err);
            1
        }
    }
}

/// A wrapper around a blk-mq `struct request`. This represents an IO request.
///
/// # Implementation details
///
/// There are four states for a request that the Rust bindings care about:
///
/// A) Request is owned by block layer (refcount 0)
/// B) Request is owned by driver but with zero `ARef`s in existence
///    (refcount 1)
/// C) Request is owned by driver with exactly one `ARef` in existence
///    (refcount 2)
/// D) Request is owned by driver with more than one `ARef` in existence
///    (refcount > 2)
///
///
/// We need to track A and B to ensure we fail tag to request conversions for
/// requests that are not owned by the driver.
///
/// We need to track C and D to ensure that it is safe to end the request and hand
/// back ownership to the block layer.
///
/// The states are tracked through the private `refcount` field of
/// `RequestDataWrapper`. This structure lives in the private data area of the C
/// `struct request`.
///
/// # Invariants
///
/// * `self.0` is a valid `struct request` created by the C portion of the kernel.
/// * The private data area associated with this request must be an initialized
///   and valid `RequestDataWrapper<T>`.
/// * `self` is reference counted by atomic modification of
///   self.wrapper_ref().refcount().
///
#[repr(transparent)]
pub struct Request<T: Operations>(Opaque<bindings::request>, PhantomData<T>);

impl<T: Operations> Request<T> {
    /// Create an `ARef<Request>` from a `struct request` pointer.
    ///
    /// # Safety
    ///
    /// * The caller must own a refcount on `ptr` that is transferred to the
    ///   returned `ARef`.
    /// * The type invariants for `Request` must hold for the pointee of `ptr`.
    pub(crate) unsafe fn aref_from_raw(ptr: *mut bindings::request) -> ARef<Self> {
        // INVARIANT: By the safety requirements of this function, invariants are upheld.
        // SAFETY: By the safety requirement of this function, we own a
        // reference count that we can pass to `ARef`.
        unsafe { ARef::from_raw(NonNull::new_unchecked(ptr as *const Self as *mut Self)) }
    }

    /// Notify the block layer that a request is going to be processed now.
    ///
    /// The block layer uses this hook to do proper initializations such as
    /// starting the timeout timer. It is a requirement that block device
    /// drivers call this function when starting to process a request.
    ///
    /// # Safety
    ///
    /// The caller must have exclusive ownership of `self`, that is
    /// `self.wrapper_ref().refcount() == 2`.
    pub(crate) unsafe fn start_unchecked(this: &ARef<Self>) {
        // SAFETY: By type invariant, `self.0` is a valid `struct request` and
        // we have exclusive access.
        unsafe { bindings::blk_mq_start_request(this.0.get()) };
    }

    /// Try to take exclusive ownership of `this` by dropping the refcount to 0.
    /// This fails if `this` is not the only `ARef` pointing to the underlying
    /// `Request`.
    ///
    /// If the operation is successful, `Ok` is returned with a pointer to the
    /// C `struct request`. If the operation fails, `this` is returned in the
    /// `Err` variant.
    fn try_set_end(this: ARef<Self>) -> Result<*mut bindings::request, ARef<Self>> {
        // We can race with `TagSet::tag_to_rq`
        if let Err(_old) = this.wrapper_ref().refcount().compare_exchange(
            2,
            0,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            return Err(this);
        }

        let request_ptr = this.0.get();
        core::mem::forget(this);

        Ok(request_ptr)
    }

    /// Notify the block layer that the request has been completed without errors.
    ///
    /// This function will return `Err` if `this` is not the only `ARef`
    /// referencing the request.
    pub fn end_ok(this: ARef<Self>) -> Result<(), ARef<Self>> {
        Self::end(this, BlkStatus::OK)
    }

    /// Notify the block layer that the request has completed with the given
    /// blk status.
    pub fn end(this: ARef<Self>, status: BlkStatus) -> Result<(), ARef<Self>> {
        let request_ptr = Self::try_set_end(this)?;

        // SAFETY: By type invariant, `this.0` was a valid `struct request`. The
        // success of the call to `try_set_end` guarantees that there are no
        // `ARef`s pointing to this request. Therefore it is safe to hand it
        // back to the block layer.
        unsafe { bindings::blk_mq_end_request(request_ptr, status.to_raw()) };

        Ok(())
    }

    /// Schedule the request for completion through the driver's
    /// `Operations::complete` callback.
    pub fn complete(this: ARef<Self>) {
        let request_ptr = this.0.get();
        core::mem::forget(this);

        // SAFETY: `request_ptr` is a live request owned by the driver. If the
        // completion cannot be remote, we reclaim the leaked reference below.
        if !unsafe { bindings::blk_mq_complete_request_remote(request_ptr) } {
            // SAFETY: We deliberately leaked one request reference above and
            // are reclaiming it here for the local completion path.
            let rq = unsafe { Self::aref_from_raw(request_ptr) };
            T::complete(rq);
        }
    }

    /// Return a pointer to the `RequestDataWrapper` stored in the private area
    /// of the request structure.
    ///
    /// # Safety
    ///
    /// - `this` must point to a valid allocation of size at least size of
    ///   `Self` plus size of `RequestDataWrapper`.
    pub(crate) unsafe fn wrapper_ptr(this: *mut Self) -> NonNull<RequestDataWrapper> {
        let request_ptr = this.cast::<bindings::request>();
        // SAFETY: By safety requirements for this function, `this` is a
        // valid allocation.
        let wrapper_ptr =
            unsafe { bindings::blk_mq_rq_to_pdu(request_ptr).cast::<RequestDataWrapper>() };
        // SAFETY: By C API contract, wrapper_ptr points to a valid allocation
        // and is not null.
        unsafe { NonNull::new_unchecked(wrapper_ptr) }
    }

    /// Return a reference to the `RequestDataWrapper` stored in the private
    /// area of the request structure.
    pub(crate) fn wrapper_ref(&self) -> &RequestDataWrapper {
        // SAFETY: By type invariant, `self.0` is a valid allocation. Further,
        // the private data associated with this request is initialized and
        // valid. The existence of `&self` guarantees that the private data is
        // valid as a shared reference.
        unsafe { Self::wrapper_ptr(self as *const Self as *mut Self).as_ref() }
    }

    /// Creates a temporary `ARef` from a live in-flight request.
    ///
    /// # Safety
    ///
    /// `ptr` must point to a request currently owned by the driver.
    pub(crate) unsafe fn borrow_inflight(ptr: *mut bindings::request) -> ARef<Self> {
        let request = unsafe { &*ptr.cast::<Self>() };
        request.wrapper_ref().refcount().fetch_add(1, Ordering::Relaxed);
        unsafe { Self::aref_from_raw(ptr) }
    }

    /// Returns the request operation.
    pub fn op(&self) -> RequestOp {
        match unsafe { bindings::req_op(self.0.get()) } as u32 {
            bindings::req_op_REQ_OP_READ => RequestOp::Read,
            bindings::req_op_REQ_OP_WRITE => RequestOp::Write,
            bindings::req_op_REQ_OP_FLUSH => RequestOp::Flush,
            bindings::req_op_REQ_OP_DISCARD => RequestOp::Discard,
            bindings::req_op_REQ_OP_WRITE_ZEROES => RequestOp::WriteZeroes,
            other => RequestOp::Unknown(other),
        }
    }

    /// Returns the starting sector for the request.
    pub fn sector(&self) -> u64 {
        unsafe { bindings::blk_rq_pos(self.0.get()) }
    }

    /// Returns the number of 512-byte sectors in the request.
    pub fn sectors(&self) -> u32 {
        unsafe { bindings::blk_rq_sectors(self.0.get()) }
    }

    /// Returns the total request size in bytes.
    pub fn bytes(&self) -> u32 {
        unsafe { bindings::blk_rq_bytes(self.0.get()) }
    }

    /// Returns whether the request is marked FUA.
    pub fn is_fua(&self) -> bool {
        let fua = 1u64 << bindings::req_flag_bits___REQ_FUA;
        let flags = unsafe { (*self.0.get()).cmd_flags as u64 };
        (flags & fua) != 0
    }

    /// Returns the hardware-context queue type.
    pub fn queue_kind(&self) -> QueueKind {
        let ty = unsafe { (*(*self.0.get()).mq_hctx).type_ as u32 };

        match ty {
            bindings::hctx_type_HCTX_TYPE_DEFAULT => QueueKind::Default,
            bindings::hctx_type_HCTX_TYPE_READ => QueueKind::Read,
            bindings::hctx_type_HCTX_TYPE_POLL => QueueKind::Poll,
            other => QueueKind::Unknown(other),
        }
    }

    /// Returns the hardware-context queue index.
    pub fn hctx_index(&self) -> u32 {
        unsafe { (*(*self.0.get()).mq_hctx).queue_num }
    }

    /// Returns whether the block layer requested a fake timeout.
    pub fn should_fake_timeout(&self) -> bool {
        unsafe { bindings::blk_should_fake_timeout(self.0.get()) }
    }

    /// Requeues the request through blk-mq.
    pub fn requeue(&self, kick_requeue_list: bool) {
        // SAFETY: The request is live and owned by the driver while this
        // method is used.
        unsafe { bindings::blk_mq_requeue_request(self.0.get(), kick_requeue_list) };
    }

    /// Returns the queue-owned data associated with this request.
    pub fn queue_data(&self) -> ForeignBorrowed<'_, T::QueueData> {
        let queuedata = unsafe { (*(*self.0.get()).q).queuedata };

        // SAFETY: `queuedata` originates from `ForeignOwnable::into_foreign`
        // in `GenDiskBuilder::build` and is owned by the queue.
        unsafe { T::QueueData::borrow(queuedata) }
    }

    /// Returns a stable identifier for this request pointer.
    pub fn opaque_id(&self) -> usize {
        self.0.get() as usize
    }

    /// Visits all request segments.
    pub fn for_each_segment<F>(&self, mut f: F) -> Result
    where
        F: FnMut(Segment) -> Result,
    {
        let mut ctx = SegmentVisit {
            f: &mut f,
            result: Ok(()),
        };

        // SAFETY: `ctx` lives until the helper returns, and the callback is
        // invoked synchronously.
        let ret = unsafe {
            bindings::rq_for_each_segment(
                self.0.get(),
                (&mut ctx as *mut SegmentVisit<'_, F>).cast(),
                Some(segment_trampoline::<F>),
            )
        };

        if let Err(err) = ctx.result {
            Err(err)
        } else if ret != 0 {
            Err(EIO)
        } else {
            Ok(())
        }
    }
}

/// A wrapper around data stored in the private area of the C `struct request`.
pub(crate) struct RequestDataWrapper {
    /// The Rust request refcount has the following states:
    ///
    /// - 0: The request is owned by C block layer.
    /// - 1: The request is owned by Rust abstractions but there are no ARef references to it.
    /// - 2+: There are `ARef` references to the request.
    refcount: AtomicU64,
}

impl RequestDataWrapper {
    /// Return a reference to the refcount of the request that is embedding
    /// `self`.
    pub(crate) fn refcount(&self) -> &AtomicU64 {
        &self.refcount
    }

    /// Return a pointer to the refcount of the request that is embedding the
    /// pointee of `this`.
    ///
    /// # Safety
    ///
    /// - `this` must point to a live allocation of at least the size of `Self`.
    pub(crate) unsafe fn refcount_ptr(this: *mut Self) -> *mut AtomicU64 {
        // SAFETY: Because of the safety requirements of this function, the
        // field projection is safe.
        unsafe { addr_of_mut!((*this).refcount) }
    }
}

// SAFETY: Exclusive access is thread-safe for `Request`. `Request` has no `&mut
// self` methods and `&self` methods that mutate `self` are internally
// synchronized.
unsafe impl<T: Operations> Send for Request<T> {}

// SAFETY: Shared access is thread-safe for `Request`. `&self` methods that
// mutate `self` are internally synchronized`
unsafe impl<T: Operations> Sync for Request<T> {}

/// Store the result of `op(target.load())` in target, returning new value of
/// target.
fn atomic_relaxed_op_return(target: &AtomicU64, op: impl Fn(u64) -> u64) -> u64 {
    let old = target.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| Some(op(x)));

    // SAFETY: Because the operation passed to `fetch_update` above always
    // return `Some`, `old` will always be `Ok`.
    let old = unsafe { old.unwrap_unchecked() };

    op(old)
}

/// Store the result of `op(target.load)` in `target` if `target.load() !=
/// pred`, returning true if the target was updated.
fn atomic_relaxed_op_unless(target: &AtomicU64, op: impl Fn(u64) -> u64, pred: u64) -> bool {
    target
        .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |x| {
            if x == pred {
                None
            } else {
                Some(op(x))
            }
        })
        .is_ok()
}

// SAFETY: All instances of `Request<T>` are reference counted. This
// implementation of `AlwaysRefCounted` ensure that increments to the ref count
// keeps the object alive in memory at least until a matching reference count
// decrement is executed.
unsafe impl<T: Operations> AlwaysRefCounted for Request<T> {
    fn inc_ref(&self) {
        let refcount = &self.wrapper_ref().refcount();

        #[cfg_attr(not(CONFIG_DEBUG_MISC), allow(unused_variables))]
        let updated = atomic_relaxed_op_unless(refcount, |x| x + 1, 0);

        #[cfg(CONFIG_DEBUG_MISC)]
        if !updated {
            panic!("Request refcount zero on clone")
        }
    }

    unsafe fn dec_ref(obj: core::ptr::NonNull<Self>) {
        // SAFETY: The type invariants of `ARef` guarantee that `obj` is valid
        // for read.
        let wrapper_ptr = unsafe { Self::wrapper_ptr(obj.as_ptr()).as_ptr() };
        // SAFETY: The type invariant of `Request` guarantees that the private
        // data area is initialized and valid.
        let refcount = unsafe { &*RequestDataWrapper::refcount_ptr(wrapper_ptr) };

        #[cfg_attr(not(CONFIG_DEBUG_MISC), allow(unused_variables))]
        let new_refcount = atomic_relaxed_op_return(refcount, |x| x - 1);

        #[cfg(CONFIG_DEBUG_MISC)]
        if new_refcount == 0 {
            panic!("Request reached refcount zero in Rust abstractions");
        }
    }
}
