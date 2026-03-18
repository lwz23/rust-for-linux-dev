// SPDX-License-Identifier: GPL-2.0

//! RTNL link registration and callback glue for Rust network drivers.
//!
//! C headers: [`include/net/rtnetlink.h`](srctree/include/net/rtnetlink.h),
//! [`include/linux/ethtool.h`](srctree/include/linux/ethtool.h).

use super::{
    netdevice::{DeviceRef, LinkStats64, NetDevice, SetupContext},
    skbuff::SkBuff,
};
use crate::{
    bindings,
    error::{from_result, to_result, Result, VTABLE_DEFAULT_ERROR},
    prelude::*,
};
use core::{cell::UnsafeCell, marker::PhantomData, mem::MaybeUninit};

/// Safe wrapper around RTNL attribute arrays.
pub struct AttrTable<'a> {
    ptr: *mut *mut bindings::nlattr,
    _p: PhantomData<&'a *mut bindings::nlattr>,
}

impl<'a> AttrTable<'a> {
    /// Creates an attribute table wrapper from a raw pointer array.
    ///
    /// # Safety
    ///
    /// `ptr` must be the attribute array provided by the RTNL core for the duration of the
    /// callback.
    unsafe fn from_raw(ptr: *mut *mut bindings::nlattr) -> Self {
        Self {
            ptr,
            _p: PhantomData,
        }
    }

    /// Returns whether the given attribute slot is present.
    pub fn is_present(&self, index: usize) -> bool {
        if self.ptr.is_null() {
            return false;
        }

        // SAFETY: The wrapper is only created from RTNL-managed arrays, and callers are expected
        // to query indices valid for the current callback context.
        unsafe { !(*self.ptr.add(index)).is_null() }
    }
}

/// Safe wrapper for `struct netlink_ext_ack`.
#[repr(transparent)]
pub struct ExtAck(bindings::netlink_ext_ack);

impl ExtAck {
    /// Creates an optional wrapper from a raw pointer.
    ///
    /// # Safety
    ///
    /// `ptr` must either be null or a valid extack pointer supplied by the RTNL core.
    unsafe fn from_raw<'a>(ptr: *mut bindings::netlink_ext_ack) -> Option<&'a mut Self> {
        if ptr.is_null() {
            None
        } else {
            // CAST: `Self` is a `repr(transparent)` wrapper around `bindings::netlink_ext_ack`.
            Some(unsafe { &mut *ptr.cast::<Self>() })
        }
    }
}

/// A transmit status returned by `ndo_start_xmit`.
#[derive(Copy, Clone)]
pub struct TxStatus(bindings::netdev_tx_t);

impl TxStatus {
    /// Packet consumed successfully.
    pub const OK: Self = Self(bindings::netdev_tx_NETDEV_TX_OK);

    pub(crate) const fn as_raw(self) -> bindings::netdev_tx_t {
        self.0
    }
}

/// Driver callbacks for a Rust rtnl-link implementation.
#[vtable]
pub trait Driver: Sized {
    /// Driver private state stored in `netdev_priv()`.
    type Private: Default;

    /// RTNL link kind name.
    const KIND: &'static CStr;

    /// Configures a newly allocated device.
    fn setup(dev: &mut SetupContext<'_, Self>);

    /// Opens the device.
    fn open(dev: &mut NetDevice<Self>) -> Result;

    /// Stops the device.
    fn stop(dev: &mut NetDevice<Self>) -> Result;

    /// Transmits or consumes an skb.
    fn start_xmit(skb: SkBuff, dev: DeviceRef<Self>) -> TxStatus;

    /// Fills 64-bit link statistics.
    fn get_stats64(dev: DeviceRef<Self>, stats: &mut LinkStats64);

    /// Validates link creation parameters.
    fn validate(
        _tb: AttrTable<'_>,
        _data: AttrTable<'_>,
        _extack: Option<&mut ExtAck>,
    ) -> Result {
        build_error!(VTABLE_DEFAULT_ERROR)
    }

    /// Returns the carrier state for ethtool's `get_link`.
    fn get_link(_dev: DeviceRef<Self>) -> u32 {
        build_error!(VTABLE_DEFAULT_ERROR)
    }
}

/// Registers a Rust rtnl-link driver.
pub struct Registration<T: Driver>(KBox<UnsafeCell<bindings::rtnl_link_ops>>, PhantomData<T>);

// SAFETY: Registration only exposes safe construction/destruction; actual registration and
// unregistration are handled by the networking core.
unsafe impl<T: Driver> Send for Registration<T> {}
// SAFETY: Shared references cannot mutate the registration directly.
unsafe impl<T: Driver> Sync for Registration<T> {}

impl<T: Driver> Registration<T> {
    const NET_DEVICE_OPS: bindings::net_device_ops = bindings::net_device_ops {
        ndo_open: Some(Self::open_callback),
        ndo_stop: Some(Self::stop_callback),
        ndo_start_xmit: Some(Self::start_xmit_callback),
        ndo_get_stats64: Some(Self::get_stats64_callback),
        // SAFETY: The rest of the struct is zeroed, which maps the optional callbacks to `None`.
        ..unsafe { MaybeUninit::<bindings::net_device_ops>::zeroed().assume_init() }
    };

    const ETHTOOL_OPS: bindings::ethtool_ops = bindings::ethtool_ops {
        get_link: if T::HAS_GET_LINK {
            Some(Self::get_link_callback)
        } else {
            None
        },
        // SAFETY: The rest of the struct is zeroed, which maps the optional callbacks to `None`.
        ..unsafe { MaybeUninit::<bindings::ethtool_ops>::zeroed().assume_init() }
    };

    const VTABLE: bindings::rtnl_link_ops = bindings::rtnl_link_ops {
        kind: crate::str::as_char_ptr_in_const_context(T::KIND),
        priv_size: core::mem::size_of::<T::Private>(),
        setup: Some(Self::setup_callback),
        validate: if T::HAS_VALIDATE {
            Some(Self::validate_callback)
        } else {
            None
        },
        // SAFETY: C drivers use the same zero-initialised tail for unneeded fields.
        ..unsafe { MaybeUninit::<bindings::rtnl_link_ops>::zeroed().assume_init() }
    };

    /// Registers the driver with the RTNL core.
    pub fn new(_module: &'static ThisModule) -> Result<Self> {
        let mut ops = KBox::new(UnsafeCell::new(Self::VTABLE), GFP_KERNEL)?;

        // SAFETY: `ops` is heap allocated and remains valid for the lifetime of the registration.
        to_result(unsafe { bindings::rtnl_link_register(ops.get_mut()) })?;

        Ok(Self(ops, PhantomData))
    }

    unsafe extern "C" fn priv_destructor_callback(dev: *mut bindings::net_device) {
        // SAFETY: The networking core invokes the destructor with the exact device previously set
        // up by `setup_callback`, so its private area is initialized as `T::Private`.
        let dev = unsafe { NetDevice::<T>::from_raw(dev) };
        // SAFETY: The core calls the destructor exactly once for each device instance.
        unsafe { dev.drop_private() };
    }

    unsafe extern "C" fn setup_callback(dev: *mut bindings::net_device) {
        // SAFETY: The RTNL core invokes `setup` for a freshly allocated device and grants unique
        // access during the callback.
        let dev = unsafe { NetDevice::<T>::from_raw(dev) };
        dev.install_ops(
            &Self::NET_DEVICE_OPS,
            &Self::ETHTOOL_OPS,
            Some(Self::priv_destructor_callback),
        );
        dev.init_private();
        T::setup(&mut SetupContext::new(dev));
    }

    unsafe extern "C" fn validate_callback(
        tb: *mut *mut bindings::nlattr,
        data: *mut *mut bindings::nlattr,
        extack: *mut bindings::netlink_ext_ack,
    ) -> c_int {
        from_result(|| {
            // SAFETY: The RTNL core supplies valid attribute tables and extack pointers for the
            // duration of the callback.
            let tb = unsafe { AttrTable::from_raw(tb) };
            let data = unsafe { AttrTable::from_raw(data) };
            let extack = unsafe { ExtAck::from_raw(extack) };
            T::validate(tb, data, extack)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn open_callback(dev: *mut bindings::net_device) -> c_int {
        from_result(|| {
            // SAFETY: `ndo_open` receives a valid device with serialized mutable access.
            let dev = unsafe { NetDevice::<T>::from_raw(dev) };
            T::open(dev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn stop_callback(dev: *mut bindings::net_device) -> c_int {
        from_result(|| {
            // SAFETY: `ndo_stop` receives a valid device with serialized mutable access.
            let dev = unsafe { NetDevice::<T>::from_raw(dev) };
            T::stop(dev)?;
            Ok(0)
        })
    }

    unsafe extern "C" fn start_xmit_callback(
        skb: *mut bindings::sk_buff,
        dev: *mut bindings::net_device,
    ) -> bindings::netdev_tx_t {
        // SAFETY: `ndo_start_xmit` transfers ownership of `skb` to the driver callback for the
        // duration of the call.
        let skb = unsafe { SkBuff::from_raw(skb) };
        // SAFETY: The networking core passes a valid device pointer for the duration of the
        // callback.
        let dev = unsafe { DeviceRef::from_raw(dev) };
        T::start_xmit(skb, dev).as_raw()
    }

    unsafe extern "C" fn get_stats64_callback(
        dev: *mut bindings::net_device,
        storage: *mut bindings::rtnl_link_stats64,
    ) {
        // SAFETY: The networking core passes valid pointers for the duration of the callback.
        let dev = unsafe { DeviceRef::from_raw(dev) };
        let stats = unsafe { LinkStats64::from_raw(storage) };
        T::get_stats64(dev, stats);
    }

    unsafe extern "C" fn get_link_callback(dev: *mut bindings::net_device) -> u32 {
        // SAFETY: The networking core passes a valid device pointer for the duration of the call.
        let dev = unsafe { DeviceRef::from_raw(dev) };
        T::get_link(dev)
    }
}

impl<T: Driver> Drop for Registration<T> {
    fn drop(&mut self) {
        // SAFETY: The existence of `self` guarantees that the registration previously succeeded
        // and that the backing `rtnl_link_ops` storage remains valid.
        unsafe { bindings::rtnl_link_unregister(self.0.get_mut()) };
    }
}
