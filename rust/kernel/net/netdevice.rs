// SPDX-License-Identifier: GPL-2.0

//! Network device helpers used by Rust rtnl-link drivers.
//!
//! C headers: [`include/linux/netdevice.h`](srctree/include/linux/netdevice.h),
//! [`include/linux/netlink.h`](srctree/include/linux/netlink.h).

use crate::{
    bindings,
    error::{code, to_result, Result},
    types::Opaque,
};
use core::ptr::NonNull;

/// Network device hardware types.
pub mod hardware {
    use crate::bindings;

    /// Netlink monitoring device hardware type.
    pub const NETLINK: u16 = bindings::ARPHRD_NETLINK as u16;
}

/// Net device private flags.
pub mod priv_flags {
    use crate::bindings;

    /// Device can operate without a qdisc.
    pub const NO_QUEUE: u32 = bindings::netdev_priv_flags_IFF_NO_QUEUE as u32;
}

/// Net device flags.
pub mod device_flags {
    use crate::bindings;

    /// Interface has no ARP.
    pub const NOARP: u32 = bindings::net_device_flags_IFF_NOARP as u32;
}

/// Net device feature masks.
pub mod features {
    use crate::bindings;

    /// Scatter-gather.
    pub const SG: bindings::netdev_features_t = bindings::NETIF_F_SG;
    /// Fragment list handling.
    pub const FRAGLIST: bindings::netdev_features_t = bindings::NETIF_F_FRAGLIST;
    /// High DMA support.
    pub const HIGHDMA: bindings::netdev_features_t = bindings::NETIF_F_HIGHDMA;
}

/// Net device per-cpu statistics kinds.
pub mod stat_type {
    use crate::bindings;

    /// `struct pcpu_lstats`.
    pub const LSTATS: bindings::netdev_stat_type =
        bindings::netdev_stat_type_NETDEV_PCPU_STAT_LSTATS;
}

/// Netlink-related constants used by `nlmon`.
pub mod netlink {
    use crate::bindings;

    /// Suggested MTU-sized payload for netlink monitoring devices.
    pub const GOODSIZE: u32 = bindings::NLMSG_GOODSIZE;
    /// The minimum nlmon MTU expected by the C driver.
    pub const HEADER_LEN: u32 = core::mem::size_of::<bindings::nlmsghdr>() as u32;
}

/// RTNL link attribute indices.
pub mod link_attrs {
    use crate::bindings;

    /// Interface address attribute.
    pub const ADDRESS: usize = bindings::IFLA_ADDRESS as usize;
}

/// A shared reference to a network device.
///
/// # Invariants
///
/// The inner pointer is non-null and valid for the duration of the wrapper's use.
#[derive(Copy, Clone)]
pub struct DeviceRef(NonNull<bindings::net_device>);

impl DeviceRef {
    /// Creates a device reference from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and point to a live `struct net_device`.
    #[allow(dead_code)]
    pub(crate) unsafe fn from_raw(ptr: *mut bindings::net_device) -> Self {
        Self(NonNull::new(ptr).expect("net_device pointer must be non-null"))
    }

    pub(crate) fn as_raw(self) -> *mut bindings::net_device {
        self.0.as_ptr()
    }
}

/// Safe wrapper for `struct netlink_tap`.
#[derive(Default)]
pub struct NetlinkTapHandle {
    inner: bindings::netlink_tap,
    registered: bool,
}

impl NetlinkTapHandle {
    fn current_module_ptr() -> *mut bindings::module {
        #[cfg(MODULE)]
        {
            // SAFETY: `__this_module` is constructed by the kernel when the module is loaded and
            // remains alive until the module is unloaded.
            unsafe { core::ptr::from_ref(&bindings::__this_module).cast_mut() }
        }

        #[cfg(not(MODULE))]
        {
            core::ptr::null_mut()
        }
    }

    /// Registers the tap for the given device.
    pub fn add(&mut self, dev: DeviceRef) -> Result {
        if self.registered {
            return Err(code::EBUSY);
        }

        self.inner.dev = dev.as_raw();
        self.inner.module = Self::current_module_ptr();

        // SAFETY: `self.inner` is a valid tap object and the device/module pointers stored above
        // remain valid for the duration of the registration.
        to_result(unsafe { bindings::netlink_add_tap(&mut self.inner) })?;
        self.registered = true;
        Ok(())
    }

    /// Unregisters the tap if it is currently active.
    pub fn remove(&mut self) -> Result {
        if !self.registered {
            return Ok(());
        }

        // SAFETY: `self.inner` is currently registered, so removing it is valid.
        to_result(unsafe { bindings::netlink_remove_tap(&mut self.inner) })?;
        self.registered = false;
        Ok(())
    }
}

impl Drop for NetlinkTapHandle {
    fn drop(&mut self) {
        if self.registered {
            // SAFETY: Best-effort cleanup of a tap that was previously registered by this object.
            let _ = unsafe { bindings::netlink_remove_tap(&mut self.inner) };
            self.registered = false;
        }
    }
}

/// Link statistics wrapper.
#[repr(transparent)]
pub struct LinkStats64(Opaque<bindings::rtnl_link_stats64>);

impl LinkStats64 {
    /// Creates a stats wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid pointer supplied by the networking core for the duration of the
    /// callback.
    #[allow(dead_code)]
    pub(crate) unsafe fn from_raw<'a>(ptr: *mut bindings::rtnl_link_stats64) -> &'a mut Self {
        // CAST: `Self` is a `repr(transparent)` wrapper around `bindings::rtnl_link_stats64`.
        unsafe { &mut *ptr.cast::<Self>() }
    }

    fn as_raw(&mut self) -> *mut bindings::rtnl_link_stats64 {
        self.0.get()
    }

    /// Stores receive packet and byte counters.
    pub fn set_rx(&mut self, packets: u64, bytes: u64) {
        let stats = self.as_raw();
        // SAFETY: The wrapper invariant guarantees that `stats` is valid for write.
        unsafe {
            (*stats).rx_packets = packets;
            (*stats).rx_bytes = bytes;
        }
    }
}

/// Helpers for per-cpu lightweight statistics.
pub struct LStats;

impl LStats {
    /// Adds one received packet with the given length.
    pub fn add(dev: DeviceRef, len: u32) {
        // SAFETY: `dev` points to a valid `net_device` and `len` is passed directly to the kernel
        // helper.
        unsafe { bindings::dev_lstats_add(dev.as_raw(), len) };
    }

    /// Reads the current receive packet and byte counters.
    pub fn read(dev: DeviceRef, stats: &mut LinkStats64) {
        let mut packets = 0;
        let mut bytes = 0;

        // SAFETY: `dev` and the output pointers are valid for the duration of the call.
        unsafe { bindings::dev_lstats_read(dev.as_raw(), &mut packets, &mut bytes) };

        stats.set_rx(packets, bytes);
    }
}
