// SPDX-License-Identifier: GPL-2.0

//! Network device helpers used by Rust rtnl-link drivers.
//!
//! C headers: [`include/linux/netdevice.h`](srctree/include/linux/netdevice.h),
//! [`include/linux/netlink.h`](srctree/include/linux/netlink.h).

use super::rtnl::Driver;
use crate::{
    bindings,
    error::{code, to_result, Result},
    prelude::*,
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

/// A callback-scoped capability that refers to the current device instance.
///
/// Unlike [`DeviceRef`], this capability can only be created from the mutable callback context of
/// the current device, so it is suitable for APIs that must not accept arbitrary devices.
pub struct CurrentDevice<'a, T: Driver> {
    ptr: NonNull<bindings::net_device>,
    _p: PhantomData<Pin<&'a mut NetDevice<T>>>,
}

impl<'a, T: Driver> CurrentDevice<'a, T> {
    /// Creates a current-device capability from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must point to the exact `net_device` currently borrowed mutably for the callback
    /// lifetime `'a`.
    pub(crate) unsafe fn from_raw(ptr: *mut bindings::net_device) -> Self {
        Self {
            ptr: NonNull::new(ptr).expect("net_device pointer must be non-null"),
            _p: PhantomData,
        }
    }

    fn as_raw(&self) -> *mut bindings::net_device {
        self.ptr.as_ptr()
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
    pub(super) unsafe fn from_raw<'a>(ptr: *mut bindings::net_device) -> Pin<&'a mut Self> {
        // CAST: `Self` is a `repr(transparent)` wrapper around `bindings::net_device`.
        unsafe { Pin::new_unchecked(&mut *ptr.cast::<Self>()) }
    }

    pub(super) const fn as_raw(&self) -> *mut bindings::net_device {
        self.inner.get()
    }

    fn as_mut_ref(self: Pin<&mut Self>) -> &mut bindings::net_device {
        // SAFETY: The wrapper invariant guarantees unique access.
        unsafe { &mut *self.get_unchecked_mut().as_raw() }
    }

    fn private_ptr(self: Pin<&Self>) -> *mut T::Private {
        // SAFETY: The callback invariants guarantee a valid device pointer. The helper returns the
        // private allocation associated with this device.
        unsafe { bindings::netdev_priv(self.as_raw()) }.cast::<T::Private>()
    }

    pub(super) fn init_private(self: Pin<&mut Self>, init: impl PinInit<T::Private>) {
        build_assert!(
            core::mem::align_of::<T::Private>() <= 32,
            "net_device private data alignment exceeds NETDEV_ALIGN"
        );

        let private_ptr = self.as_ref().private_ptr();

        // SAFETY:
        // - `setup` is the first callback to access the private area for a freshly allocated
        //   device, so the slot is uninitialized and valid for write.
        // - `netdev_priv()` returns storage inside the allocated `net_device`, so the private
        //   object occupies a stable memory location for the lifetime of the device.
        match unsafe { init.__pinned_init(private_ptr) } {
            Ok(()) => {}
            Err(e) => match e {},
        }
    }

    pub(super) unsafe fn drop_private(self: Pin<&mut Self>) {
        // SAFETY: The caller guarantees the private area was initialized exactly once and this is
        // the matching teardown path.
        unsafe { core::ptr::drop_in_place(self.as_ref().private_ptr()) };
    }

    pub(super) fn install_ops(
        self: Pin<&mut Self>,
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

    /// Executes a closure with pinned private data and a capability for the current device.
    pub fn with_private<R>(
        self: Pin<&mut Self>,
        f: impl FnOnce(Pin<&mut T::Private>, CurrentDevice<'_, T>) -> R,
    ) -> R {
        let raw = self.as_ref().get_ref().as_raw();
        let private_ptr = self.as_ref().private_ptr();

        // SAFETY:
        // - `private_ptr` refers to the already initialized private area of the current device.
        // - The private storage lives inside the `net_device`, which is pinned for the duration of
        //   the callback, so the pointee is stable.
        let private = unsafe { Pin::new_unchecked(&mut *private_ptr) };
        // SAFETY: `raw` comes from the current mutable callback borrow of `self`.
        let dev = unsafe { CurrentDevice::from_raw(raw) };
        f(private, dev)
    }

    /// Returns a shared reference to the driver's private state.
    pub fn private(self: Pin<&Self>) -> &T::Private {
        // SAFETY: The wrapper invariant guarantees that the private area has been initialized.
        unsafe { &*self.private_ptr() }
    }
}

/// Device setup context.
pub struct SetupContext<'a, T: Driver> {
    dev: Pin<&'a mut NetDevice<T>>,
}

impl<'a, T: Driver> SetupContext<'a, T> {
    pub(super) fn new(dev: Pin<&'a mut NetDevice<T>>) -> Self {
        Self { dev }
    }

    /// Sets the device hardware type.
    pub fn set_type(&mut self, type_: u16) {
        self.dev.as_mut().as_mut_ref().type_ = type_;
    }

    /// Adds private flags.
    pub fn add_private_flags(&mut self, flags: u32) {
        let bits = unsafe { &mut self.dev.as_mut().as_mut_ref().__bindgen_anon_1.__bindgen_anon_1 };
        let current = bits.priv_flags() as u32;
        bits.set_priv_flags((current | flags) as _);
    }

    /// Sets the lockless transmit flag.
    pub fn set_lltx(&mut self, enabled: bool) {
        let bits = unsafe { &mut self.dev.as_mut().as_mut_ref().__bindgen_anon_1.__bindgen_anon_1 };
        bits.set_lltx(enabled.into());
    }

    /// Sets the device feature mask.
    pub fn set_features(&mut self, features: bindings::netdev_features_t) {
        self.dev.as_mut().as_mut_ref().features = features;
    }

    /// Sets the device flags.
    pub fn set_flags(&mut self, flags: u32) {
        self.dev.as_mut().as_mut_ref().flags = flags;
    }

    /// Enables per-cpu lightweight statistics.
    pub fn enable_lstats(&mut self) {
        self.dev.as_mut().as_mut_ref().set_pcpu_stat_type(stat_type::LSTATS);
    }

    /// Sets the MTU.
    pub fn set_mtu(&mut self, mtu: u32) {
        self.dev.as_mut().as_mut_ref().mtu = mtu;
    }

    /// Sets the minimum MTU.
    pub fn set_min_mtu(&mut self, mtu: u32) {
        self.dev.as_mut().as_mut_ref().min_mtu = mtu;
    }
}

/// Safe wrapper for `struct netlink_tap`.
#[pin_data(PinnedDrop)]
pub struct NetlinkTapHandle {
    #[pin]
    inner: Opaque<bindings::netlink_tap>,
    registered: bool,
}

impl NetlinkTapHandle {
    /// Creates a new unregistered tap handle.
    pub fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            inner <- Opaque::ffi_init(|slot| unsafe { core::ptr::write_bytes(slot, 0, 1) }),
            registered: false,
        })
    }

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
    pub fn add<T: Driver>(mut self: Pin<&mut Self>, dev: CurrentDevice<'_, T>) -> Result {
        let raw = self.as_ref().get_ref().inner.get();
        let this = self.as_mut().project();

        if *this.registered {
            return Err(code::EBUSY);
        }

        // SAFETY: `raw` points to the pinned tap object managed by `self`.
        unsafe {
            (*raw).dev = dev.as_raw();
            (*raw).module = Self::current_module_ptr();
        }

        // SAFETY: `raw` is a valid tap object and the device/module pointers stored above
        // remain valid for the duration of the registration.
        to_result(unsafe { bindings::netlink_add_tap(raw) })?;
        *this.registered = true;
        Ok(())
    }

    /// Unregisters the tap if it is currently active.
    pub fn remove(mut self: Pin<&mut Self>) -> Result {
        let raw = self.as_ref().get_ref().inner.get();
        let this = self.as_mut().project();

        if !*this.registered {
            return Ok(());
        }

        // SAFETY: `raw` is currently registered, so removing it is valid.
        to_result(unsafe { bindings::netlink_remove_tap(raw) })?;
        *this.registered = false;
        Ok(())
    }
}

#[pinned_drop]
impl PinnedDrop for NetlinkTapHandle {
    fn drop(mut self: Pin<&mut Self>) {
        let raw = self.as_ref().get_ref().inner.get();
        let this = self.as_mut().project();

        if *this.registered {
            // SAFETY: Best-effort cleanup of a tap that was previously registered by this object.
            let _ = unsafe { bindings::netlink_remove_tap(raw) };
            *this.registered = false;
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
