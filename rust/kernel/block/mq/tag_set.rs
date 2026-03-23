// SPDX-License-Identifier: GPL-2.0

//! This module provides the `TagSet` struct to wrap the C `struct blk_mq_tag_set`.
//!
//! C header: [`include/linux/blk-mq.h`](srctree/include/linux/blk-mq.h)

use core::pin::Pin;

use crate::{
    bindings,
    block::mq::{operations::OperationsVTable, request::RequestDataWrapper, Operations},
    error,
    init,
    prelude::PinInit,
    try_pin_init,
    types::Opaque,
};
use core::{
    convert::TryInto,
    marker::PhantomData,
    sync::atomic::{AtomicU32, Ordering},
};
use macros::{pin_data, pinned_drop};

struct QueueMappingData {
    submit_queues: AtomicU32,
    poll_queues: AtomicU32,
}

impl QueueMappingData {
    fn new(submit_queues: u32, poll_queues: u32) -> Self {
        Self {
            submit_queues: AtomicU32::new(submit_queues),
            poll_queues: AtomicU32::new(poll_queues),
        }
    }

    fn load(&self) -> (u32, u32) {
        (
            self.submit_queues.load(Ordering::Acquire),
            self.poll_queues.load(Ordering::Acquire),
        )
    }

    fn store(&self, submit_queues: u32, poll_queues: u32) {
        self.submit_queues.store(submit_queues, Ordering::Release);
        self.poll_queues.store(poll_queues, Ordering::Release);
    }
}

/// Tag-set construction parameters.
pub struct TagSetConfig {
    submit_queues: u32,
    poll_queues: u32,
    queue_depth: u32,
    nr_maps: u32,
    timeout: u32,
    numa_node: i32,
    flags: u32,
}

impl TagSetConfig {
    /// Creates a new configuration.
    pub fn new(submit_queues: u32, queue_depth: u32, nr_maps: u32) -> Self {
        Self {
            submit_queues,
            poll_queues: 0,
            queue_depth,
            nr_maps,
            timeout: 0,
            numa_node: bindings::NUMA_NO_NODE,
            flags: bindings::BLK_MQ_F_SHOULD_MERGE,
        }
    }

    /// Sets the number of poll queues and enables the standard 3 blk-mq maps.
    pub fn poll_queues(mut self, poll_queues: u32) -> Self {
        self.poll_queues = poll_queues;
        self.nr_maps = if poll_queues > 0 { 3 } else { self.nr_maps.max(1) };
        self
    }

    /// Sets the blk-mq timeout in jiffies.
    pub fn timeout(mut self, timeout: u32) -> Self {
        self.timeout = timeout;
        self
    }

    /// Sets the NUMA node.
    pub fn numa_node(mut self, numa_node: i32) -> Self {
        self.numa_node = numa_node;
        self
    }

    /// Enables or disables the no-scheduler flag.
    pub fn no_sched(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags |= bindings::BLK_MQ_F_NO_SCHED;
        } else {
            self.flags &= !bindings::BLK_MQ_F_NO_SCHED;
        }
        self
    }

    /// Enables or disables the shared tag-bitmap flag.
    pub fn shared_tag_bitmap(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags |= bindings::BLK_MQ_F_TAG_HCTX_SHARED;
        } else {
            self.flags &= !bindings::BLK_MQ_F_TAG_HCTX_SHARED;
        }
        self
    }

    /// Enables or disables blocking queueing.
    pub fn blocking(mut self, enabled: bool) -> Self {
        if enabled {
            self.flags |= bindings::BLK_MQ_F_BLOCKING;
        } else {
            self.flags &= !bindings::BLK_MQ_F_BLOCKING;
        }
        self
    }
}

/// A wrapper for the C `struct blk_mq_tag_set`.
///
/// `struct blk_mq_tag_set` contains a `struct list_head` and so must be pinned.
///
/// # Invariants
///
/// - `inner` is initialized and valid.
#[pin_data(PinnedDrop)]
pub struct TagSet<T: Operations> {
    #[pin]
    inner: Opaque<bindings::blk_mq_tag_set>,
    mapping: QueueMappingData,
    _p: PhantomData<T>,
}

// SAFETY: A tag set owns its underlying blk-mq object and only exposes
// synchronized shared access after initialization.
unsafe impl<T: Operations + Send> Send for TagSet<T> {}

// SAFETY: The blk-mq core synchronizes concurrent use of the underlying tag
// set and the Rust wrapper only provides shared access.
unsafe impl<T: Operations + Send> Sync for TagSet<T> {}

impl<T: Operations> TagSet<T> {
    /// Tries to create a new tag set.
    pub fn new(
        nr_hw_queues: u32,
        num_tags: u32,
        num_maps: u32,
    ) -> impl PinInit<Self, error::Error> {
        Self::with_config(TagSetConfig::new(nr_hw_queues, num_tags, num_maps))
    }

    /// Tries to create a new tag set from an explicit configuration.
    pub fn with_config(config: TagSetConfig) -> impl PinInit<Self, error::Error> {
        // SAFETY: `blk_mq_tag_set` only contains integers and pointers, which
        // all are allowed to be 0.
        let tag_set: bindings::blk_mq_tag_set = unsafe { core::mem::zeroed() };
        let tag_set: error::Result<bindings::blk_mq_tag_set> =
            core::mem::size_of::<RequestDataWrapper>()
            .try_into()
            .map(|cmd_size| {
                bindings::blk_mq_tag_set {
                    ops: OperationsVTable::<T>::build(),
                    nr_hw_queues: config.submit_queues + config.poll_queues,
                    timeout: config.timeout,
                    numa_node: config.numa_node,
                    queue_depth: config.queue_depth,
                    cmd_size,
                    flags: config.flags,
                    driver_data: core::ptr::null_mut(),
                    nr_maps: config.nr_maps,
                    ..tag_set
                }
            })
            .map_err(Into::into);

        try_pin_init!(TagSet {
            inner <- unsafe {
                init::pin_init_from_closure(|place: *mut Opaque<bindings::blk_mq_tag_set>| {
                    core::ptr::write(Opaque::raw_get(place), tag_set?);
                    Ok::<(), error::Error>(())
                })
            },
            mapping: QueueMappingData::new(config.submit_queues, config.poll_queues),
            _p: PhantomData,
        })
        .pin_chain(|this| {
            // SAFETY: The object is pinned and fully initialized here, so the
            // address of `mapping` is stable for the lifetime of the tag set.
            unsafe {
                (*this.inner.get()).driver_data =
                    core::ptr::addr_of!(this.mapping).cast_mut().cast();
            }

            // SAFETY: `this.inner` is initialized and ready for blk-mq.
            error::to_result(unsafe { bindings::blk_mq_alloc_tag_set(this.inner.get()) })
        })
    }

    /// Returns the wrapped `struct blk_mq_tag_set`.
    pub(crate) fn raw_tag_set(&self) -> *mut bindings::blk_mq_tag_set {
        self.inner.get()
    }

    /// Updates the submit/poll queue counts for this tag set.
    pub fn update_queue_counts(&self, submit_queues: u32, poll_queues: u32) -> error::Result {
        let old = self.mapping.load();
        self.mapping.store(submit_queues, poll_queues);

        let total = submit_queues
            .checked_add(poll_queues)
            .ok_or(error::code::EINVAL)? as core::ffi::c_int;

        // SAFETY: `self.inner` is a live tag set.
        unsafe { bindings::blk_mq_update_nr_hw_queues(self.inner.get(), total) };

        // SAFETY: `self.inner` remains valid after the update attempt.
        let success = unsafe { (*self.inner.get()).nr_hw_queues } == total as u32;

        if success {
            Ok(())
        } else {
            self.mapping.store(old.0, old.1);
            Err(error::code::ENOMEM)
        }
    }

    pub(crate) unsafe fn map_queues(set: *mut bindings::blk_mq_tag_set) {
        let driver_data = unsafe { (*set).driver_data };
        let (submit_queues, poll_queues) = if driver_data.is_null() {
            (unsafe { (*set).nr_hw_queues }, 0)
        } else {
            // SAFETY: `driver_data` points at the pinned `mapping` field.
            unsafe { &*driver_data.cast::<QueueMappingData>() }.load()
        };

        let mut queue_offset = 0;
        let nr_maps = unsafe { (*set).nr_maps } as usize;

        for index in 0..nr_maps {
            // SAFETY: blk-mq always provides `nr_maps` entries in `set->map`.
            let map = unsafe { &mut (*set).map[index] };

            match index as u32 {
                bindings::hctx_type_HCTX_TYPE_DEFAULT => {
                    map.nr_queues = submit_queues;
                }
                bindings::hctx_type_HCTX_TYPE_READ => {
                    map.nr_queues = 0;
                    continue;
                }
                bindings::hctx_type_HCTX_TYPE_POLL => {
                    map.nr_queues = poll_queues;
                }
                _ => {
                    map.nr_queues = 0;
                    continue;
                }
            }

            map.queue_offset = queue_offset;
            queue_offset += map.nr_queues;

            // SAFETY: blk-mq provided a valid queue-map entry.
            unsafe { bindings::blk_mq_map_queues(map) };
        }
    }
}

#[pinned_drop]
impl<T: Operations> PinnedDrop for TagSet<T> {
    fn drop(self: Pin<&mut Self>) {
        // SAFETY: By type invariant `inner` is valid and has been properly
        // initialized during construction.
        unsafe { bindings::blk_mq_free_tag_set(self.inner.get()) };
    }
}
