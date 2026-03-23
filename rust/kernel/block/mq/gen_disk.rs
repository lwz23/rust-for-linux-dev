// SPDX-License-Identifier: GPL-2.0

//! Generic disk abstraction.
//!
//! C header: [`include/linux/blkdev.h`](srctree/include/linux/blkdev.h)
//! C header: [`include/linux/blk_mq.h`](srctree/include/linux/blk_mq.h)

use crate::block::mq::{raw_writer::RawWriter, Operations, TagSet};
use crate::error;
use crate::{
    bindings,
    error::from_err_ptr,
    error::Result,
    sync::Arc,
    types::{ForeignOwnable, ScopeGuard},
};
use core::fmt::{self, Write};

/// A builder for [`GenDisk`].
///
/// Use this struct to configure and add new [`GenDisk`] to the VFS.
pub struct GenDiskBuilder {
    rotational: bool,
    logical_block_size: u32,
    physical_block_size: u32,
    capacity_sectors: u64,
    max_hw_sectors: u32,
    max_hw_discard_sectors: u32,
    virt_boundary_mask: u32,
}

impl Default for GenDiskBuilder {
    fn default() -> Self {
        Self {
            rotational: false,
            logical_block_size: bindings::PAGE_SIZE as u32,
            physical_block_size: bindings::PAGE_SIZE as u32,
            capacity_sectors: 0,
            max_hw_sectors: 0,
            max_hw_discard_sectors: 0,
            virt_boundary_mask: 0,
        }
    }
}

impl GenDiskBuilder {
    /// Create a new instance.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the rotational media attribute for the device to be built.
    pub fn rotational(mut self, rotational: bool) -> Self {
        self.rotational = rotational;
        self
    }

    /// Validate block size by verifying that it is between 512 and `PAGE_SIZE`,
    /// and that it is a power of two.
    fn validate_block_size(size: u32) -> Result<()> {
        if !(512..=bindings::PAGE_SIZE as u32).contains(&size) || !size.is_power_of_two() {
            Err(error::code::EINVAL)
        } else {
            Ok(())
        }
    }

    /// Set the logical block size of the device to be built.
    ///
    /// This method will check that block size is a power of two and between 512
    /// and 4096. If not, an error is returned and the block size is not set.
    ///
    /// This is the smallest unit the storage device can address. It is
    /// typically 4096 bytes.
    pub fn logical_block_size(mut self, block_size: u32) -> Result<Self> {
        Self::validate_block_size(block_size)?;
        self.logical_block_size = block_size;
        Ok(self)
    }

    /// Set the physical block size of the device to be built.
    ///
    /// This method will check that block size is a power of two and between 512
    /// and 4096. If not, an error is returned and the block size is not set.
    ///
    /// This is the smallest unit a physical storage device can write
    /// atomically. It is usually the same as the logical block size but may be
    /// bigger. One example is SATA drives with 4096 byte physical block size
    /// that expose a 512 byte logical block size to the operating system.
    pub fn physical_block_size(mut self, block_size: u32) -> Result<Self> {
        Self::validate_block_size(block_size)?;
        self.physical_block_size = block_size;
        Ok(self)
    }

    /// Set the capacity of the device to be built, in sectors (512 bytes).
    pub fn capacity_sectors(mut self, capacity: u64) -> Self {
        self.capacity_sectors = capacity;
        self
    }

    /// Set the queue's maximum hardware sector count.
    pub fn max_hw_sectors(mut self, sectors: u32) -> Self {
        self.max_hw_sectors = sectors;
        self
    }

    /// Set the queue's maximum discard sectors.
    pub fn max_hw_discard_sectors(mut self, sectors: u32) -> Self {
        self.max_hw_discard_sectors = sectors;
        self
    }

    /// Set the queue's virtual boundary mask.
    pub fn virt_boundary_mask(mut self, mask: u32) -> Self {
        self.virt_boundary_mask = mask;
        self
    }

    /// Build a new `GenDisk` and add it to the VFS.
    pub fn build<T: Operations>(
        self,
        name: fmt::Arguments<'_>,
        tagset: Arc<TagSet<T>>,
        queue_data: T::QueueData,
    ) -> Result<GenDisk<T>> {
        let queue_data = queue_data.into_foreign();
        let recover_queue_data = ScopeGuard::new(|| {
            // SAFETY: `queue_data` was produced by `into_foreign` above and
            // has not been handed back on this error path.
            drop(unsafe { T::QueueData::from_foreign(queue_data) });
        });

        let lock_class_key = crate::sync::LockClassKey::new();
        let mut limits: bindings::queue_limits = unsafe { core::mem::zeroed() };

        limits.logical_block_size = self.logical_block_size;
        limits.physical_block_size = self.physical_block_size;
        limits.max_hw_sectors = self.max_hw_sectors;
        limits.max_hw_discard_sectors = self.max_hw_discard_sectors;
        limits.virt_boundary_mask = self.virt_boundary_mask as u64;

        // SAFETY: `tagset.raw_tag_set()` points to a valid and initialized tag set
        let gendisk = from_err_ptr(unsafe {
            bindings::__blk_mq_alloc_disk(
                tagset.raw_tag_set(),
                &mut limits,
                queue_data.cast_mut(),
                lock_class_key.as_ptr(),
            )
        })?;

        const TABLE: bindings::block_device_operations = bindings::block_device_operations {
            submit_bio: None,
            open: None,
            release: None,
            ioctl: None,
            compat_ioctl: None,
            check_events: None,
            unlock_native_capacity: None,
            getgeo: None,
            set_read_only: None,
            swap_slot_free_notify: None,
            report_zones: None,
            devnode: None,
            alternative_gpt_sector: None,
            get_unique_id: None,
            // TODO: Set to THIS_MODULE. Waiting for const_refs_to_static feature to
            // be merged (unstable in rustc 1.78 which is staged for linux 6.10)
            // https://github.com/rust-lang/rust/issues/119618
            owner: core::ptr::null_mut(),
            pr_ops: core::ptr::null_mut(),
            free_disk: None,
            poll_bio: None,
        };

        // SAFETY: `gendisk` is a valid pointer as we initialized it above
        unsafe { (*gendisk).fops = &TABLE };

        let mut raw_writer = RawWriter::from_array(
            // SAFETY: `gendisk` points to a valid and initialized instance. We
            // have exclusive access, since the disk is not added to the VFS
            // yet.
            unsafe { &mut (*gendisk).disk_name },
        )?;
        raw_writer.write_fmt(name)?;
        raw_writer.write_char('\0')?;

        // SAFETY: `gendisk` points to a valid and initialized instance of
        // `struct gendisk`. We have exclusive access, so we cannot race.
        unsafe {
            bindings::blk_queue_logical_block_size((*gendisk).queue, self.logical_block_size)
        };

        // SAFETY: `gendisk` points to a valid and initialized instance of
        // `struct gendisk`. We have exclusive access, so we cannot race.
        unsafe {
            bindings::blk_queue_physical_block_size((*gendisk).queue, self.physical_block_size)
        };

        // SAFETY: `gendisk` points to a valid and initialized instance of
        // `struct gendisk`. `set_capacity` takes a lock to synchronize this
        // operation, so we will not race.
        unsafe { bindings::set_capacity(gendisk, self.capacity_sectors) };

        if !self.rotational {
            // SAFETY: `gendisk` points to a valid and initialized instance of
            // `struct gendisk`. This operation uses a relaxed atomic bit flip
            // operation, so there is no race on this field.
            unsafe { bindings::blk_queue_flag_set(bindings::QUEUE_FLAG_NONROT, (*gendisk).queue) };
        } else {
            // SAFETY: `gendisk` points to a valid and initialized instance of
            // `struct gendisk`. This operation uses a relaxed atomic bit flip
            // operation, so there is no race on this field.
            unsafe {
                bindings::blk_queue_flag_clear(bindings::QUEUE_FLAG_NONROT, (*gendisk).queue)
            };
        }

        crate::error::to_result(
            // SAFETY: `gendisk` points to a valid and initialized instance of
            // `struct gendisk`.
            unsafe {
                bindings::device_add_disk(core::ptr::null_mut(), gendisk, core::ptr::null_mut())
            },
        )?;

        recover_queue_data.dismiss();

        // INVARIANT: `gendisk` was initialized above.
        // INVARIANT: `gendisk` was added to the VFS via `device_add_disk` above.
        Ok(GenDisk {
            _tagset: tagset,
            gendisk,
        })
    }
}

/// A generic block device.
///
/// # Invariants
///
///  - `gendisk` must always point to an initialized and valid `struct gendisk`.
///  - `gendisk` was added to the VFS through a call to
///     `bindings::device_add_disk`.
pub struct GenDisk<T: Operations> {
    _tagset: Arc<TagSet<T>>,
    gendisk: *mut bindings::gendisk,
}

// SAFETY: `GenDisk` is an owned pointer to a `struct gendisk` and an `Arc` to a
// `TagSet` It is safe to send this to other threads as long as T is Send.
unsafe impl<T: Operations + Send> Send for GenDisk<T> {}

impl<T: Operations> GenDisk<T> {
    fn raw_queue(&self) -> *mut bindings::request_queue {
        // SAFETY: By type invariant, `self.gendisk` always points to a live
        // `gendisk`.
        unsafe { (*self.gendisk).queue }
    }

    /// Enables or disables the queue write-cache setting.
    pub fn set_write_cache(&self, enabled: bool, fua: bool) {
        // SAFETY: The queue belongs to this live gendisk.
        unsafe { bindings::blk_queue_write_cache(self.raw_queue(), enabled, fua) };
    }

    /// Stops all hardware queues.
    pub fn stop_hw_queues(&self) {
        // SAFETY: The queue belongs to this live gendisk.
        unsafe { bindings::blk_mq_stop_hw_queues(self.raw_queue()) };
    }

    /// Restarts stopped hardware queues.
    pub fn start_stopped_hw_queues(&self, asynchronous: bool) {
        // SAFETY: The queue belongs to this live gendisk.
        unsafe { bindings::blk_mq_start_stopped_hw_queues(self.raw_queue(), asynchronous) };
    }
}

impl<T: Operations> Drop for GenDisk<T> {
    fn drop(&mut self) {
        // SAFETY: `queuedata` was initialised from `ForeignOwnable::into_foreign`
        // in `GenDiskBuilder::build` and remains owned by this queue.
        let queue_data = unsafe { (*(*self.gendisk).queue).queuedata };

        // SAFETY: By type invariant, `self.gendisk` points to a valid and
        // initialized instance of `struct gendisk`, and it was previously added
        // to the VFS.
        unsafe { bindings::del_gendisk(self.gendisk) };

        // SAFETY: Ownership of `queue_data` is transferred back exactly once
        // during `GenDisk` teardown.
        drop(unsafe { T::QueueData::from_foreign(queue_data) });
    }
}
