// SPDX-License-Identifier: GPL-2.0

//! Network device helpers used by Rust rtnl-link drivers.
//!
//! C headers: [`include/linux/netdevice.h`](srctree/include/linux/netdevice.h),
//! [`include/linux/netlink.h`](srctree/include/linux/netlink.h).

use super::rtnl::Driver;
use crate::{
    bindings,
    build_assert,
    error::{code, to_result, Result},
    types::Opaque,
};
use core::{marker::PhantomData, ptr::NonNull};

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

    /// Highest valid top-level link attribute index.
    pub const MAX: usize = bindings::__IFLA_MAX as usize - 1;
}

/// A shared reference to a network device.
///
/// # Invariants
///
/// The inner pointer is non-null and valid for the duration of the wrapper's use.
#[derive(Copy, Clone)]
pub struct DeviceRef<'a, T: Driver> {
    ptr: NonNull<bindings::net_device>,
    _p: PhantomData<&'a T>,
}

impl<'a, T: Driver> DeviceRef<'a, T> {
    /// Creates a device reference from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be non-null and point to a live `struct net_device`.
    pub(crate) unsafe fn from_raw(ptr: *mut bindings::net_device) -> Self {
        Self {
            ptr: NonNull::new(ptr).expect("net_device pointer must be non-null"),
            _p: PhantomData,
        }
    }

    pub(crate) fn as_raw(self) -> *mut bindings::net_device {
        self.ptr.as_ptr()
    }

    fn as_ref(&self) -> &bindings::net_device {
        // SAFETY: The type invariant guarantees that `self.ptr` remains valid for `'a`.
        unsafe { self.ptr.as_ref() }
    }

    /// Returns the lightweight-statistics capability if the device has been configured for it.
    pub fn lstats(self) -> Option<LStatsHandle<'a, T>> {
        if self.as_ref().pcpu_stat_type() == stat_type::LSTATS {
            Some(LStatsHandle { dev: self })
        } else {
            None
        }
    }
}

/// A mutable network device during serialized callbacks such as `setup`, `open`, or `stop`.
///
/// # Invariants
///
/// The inner `net_device` pointer is valid for the duration of the callback and its private area
/// is initialized as `T::Private` before any safe accessors are used.
#[repr(transparent)]
pub struct NetDevice<T: Driver> {
    inner: Opaque<bindings::net_device>,
    _p: PhantomData<T>,
}

impl<T: Driver> NetDevice<T> {
    /// Creates a mutable device wrapper from a raw `net_device` pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must be a valid `net_device` and the caller must be in a context where unique access
    /// to the wrapped object is permitted for the returned lifetime.
    pub(super) unsafe fn from_raw<'a>(ptr: *mut bindings::net_device) -> &'a mut Self {
        // CAST: `Self` is a `repr(transparent)` wrapper around `bindings::net_device`.
        unsafe { &mut *ptr.cast::<Self>() }
    }

    pub(super) const fn as_raw(&self) -> *mut bindings::net_device {
        self.inner.get()
    }

    fn as_mut_ref(&mut self) -> &mut bindings::net_device {
        // SAFETY: The wrapper invariant guarantees unique access.
        unsafe { &mut *self.as_raw() }
    }

    fn private_ptr(&self) -> *mut T::Private {
        // SAFETY: The callback invariants guarantee a valid device pointer. The helper returns the
        // private allocation associated with this device.
        unsafe { bindings::netdev_priv(self.as_raw()) }.cast::<T::Private>()
    }

    pub(super) fn init_private(&mut self) {
        build_assert!(
            core::mem::align_of::<T::Private>() <= 32,
            "net_device private data alignment exceeds NETDEV_ALIGN"
        );

        // SAFETY: `setup` is the first callback to access the private area for a freshly
        // allocated device, so the memory is valid for initialization and not yet initialized.
        unsafe { self.private_ptr().write(T::Private::default()) };
    }

    pub(super) unsafe fn drop_private(&mut self) {
        // SAFETY: The caller guarantees the private area was initialized exactly once and this is
        // the matching teardown path.
        unsafe { core::ptr::drop_in_place(self.private_ptr()) };
    }

    pub(super) fn install_ops(
        &mut self,
        netdev_ops: &'static bindings::net_device_ops,
        ethtool_ops: &'static bindings::ethtool_ops,
        priv_destructor: Option<unsafe extern "C" fn(*mut bindings::net_device)>,
    ) {
        let dev = self.as_mut_ref();
        dev.netdev_ops = netdev_ops;
        dev.ethtool_ops = ethtool_ops;
        dev.needs_free_netdev = true;
        dev.priv_destructor = priv_destructor;
    }

    /// Executes a closure with mutable private data and a callback-scoped device reference.
    pub fn with_private<R>(
        &mut self,
        f: impl FnOnce(&mut T::Private, DeviceRef<'_, T>) -> R,
    ) -> R {
        let raw = self.as_raw();
        let private = self.private_mut();
        // SAFETY: `raw` originates from `self`, which remains valid for the duration of this
        // method. `DeviceRef` no longer exposes access to private data, so pairing it with
        // `private` does not create an aliasing hole in safe code.
        let dev = unsafe { DeviceRef::from_raw(raw) };
        f(private, dev)
    }

    /// Returns the driver's private data.
    pub fn private(&self) -> &T::Private {
        // SAFETY: The wrapper invariant guarantees that the private area has been initialized.
        unsafe { &*self.private_ptr() }
    }

    /// Returns the driver's private data mutably.
    pub fn private_mut(&mut self) -> &mut T::Private {
        // SAFETY: The wrapper invariant guarantees that the private area has been initialized and
        // that `&mut self` gives unique access for the duration of the borrow.
        unsafe { &mut *self.private_ptr() }
    }
}

/// Device setup context.
pub struct SetupContext<'a, T: Driver> {
    dev: &'a mut NetDevice<T>,
}

impl<'a, T: Driver> SetupContext<'a, T> {
    pub(super) fn new(dev: &'a mut NetDevice<T>) -> Self {
        Self { dev }
    }

    /// Sets the device hardware type.
    pub fn set_type(&mut self, type_: u16) {
        self.dev.as_mut_ref().type_ = type_;
    }

    /// Adds private flags.
    pub fn add_private_flags(&mut self, flags: u32) {
        let bits = unsafe { &mut self.dev.as_mut_ref().__bindgen_anon_1.__bindgen_anon_1 };
        let current = bits.priv_flags() as u32;
        bits.set_priv_flags((current | flags) as _);
    }

    /// Sets the lockless transmit flag.
    pub fn set_lltx(&mut self, enabled: bool) {
        let bits = unsafe { &mut self.dev.as_mut_ref().__bindgen_anon_1.__bindgen_anon_1 };
        bits.set_lltx(enabled.into());
    }

    /// Sets the device feature mask.
    pub fn set_features(&mut self, features: bindings::netdev_features_t) {
        self.dev.as_mut_ref().features = features;
    }

    /// Sets the device flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.dev.as_mut_ref().flags = flags;
    }

    /// Enables per-cpu lightweight statistics.
    pub fn enable_lstats(&mut self) {
        self.dev.as_mut_ref().set_pcpu_stat_type(stat_type::LSTATS);
    }

    /// Sets the MTU.
    pub fn set_mtu(&mut self, mtu: u32) {
        self.dev.as_mut_ref().mtu = mtu;
    }

    /// Sets the minimum MTU.
    pub fn set_min_mtu(&mut self, mtu: u32) {
        self.dev.as_mut_ref().min_mtu = mtu;
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
    pub fn add<T: Driver>(&mut self, dev: DeviceRef<'_, T>) -> Result {
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
    pub(super) unsafe fn from_raw<'a>(ptr: *mut bindings::rtnl_link_stats64) -> &'a mut Self {
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

/// A callback-scoped capability for devices configured with `NETDEV_PCPU_STAT_LSTATS`.
#[derive(Copy, Clone)]
pub struct LStatsHandle<'a, T: Driver> {
    dev: DeviceRef<'a, T>,
}

impl<'a, T: Driver> LStatsHandle<'a, T> {
    /// Adds one received packet with the given length.
    pub fn add(self, len: u32) {
        // SAFETY: `dev` points to a valid `net_device` and `len` is passed directly to the kernel
        // helper.
        unsafe { bindings::dev_lstats_add(self.dev.as_raw(), len) };
    }

    /// Reads the current receive packet and byte counters.
    pub fn read(self, stats: &mut LinkStats64) {
        let mut packets = 0;
        let mut bytes = 0;

        // SAFETY: `dev` and the output pointers are valid for the duration of the call.
        unsafe { bindings::dev_lstats_read(self.dev.as_raw(), &mut packets, &mut bytes) };

        stats.set_rx(packets, bytes);
    }
}
