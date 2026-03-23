// SPDX-License-Identifier: GPL-2.0

//! Thin abstractions for the public `cpufreq-dt` C seam.

use crate::{
    bindings,
    cpufreq::{Policy, TableIndex},
    error::{to_result, Result},
    ffi::c_void,
    platform::PlatData,
    prelude::*,
    types::Opaque,
};

/// Immutable view of `struct cpufreq_dt_platform_data`.
#[repr(transparent)]
pub struct PlatformData(Opaque<bindings::cpufreq_dt_platform_data>);

/// Owned copy of the optional callback plumbing carried by `cpufreq_dt_platform_data`.
#[derive(Copy, Clone)]
pub struct PlatformCallbacks {
    have_governor_per_policy: bool,
    get_intermediate: Option<unsafe extern "C" fn(*mut bindings::cpufreq_policy, u32) -> u32>,
    target_intermediate: Option<unsafe extern "C" fn(*mut bindings::cpufreq_policy, u32) -> i32>,
    suspend: Option<unsafe extern "C" fn(*mut bindings::cpufreq_policy) -> i32>,
    resume: Option<unsafe extern "C" fn(*mut bindings::cpufreq_policy) -> i32>,
}

// SAFETY: The wrapped pointer originates from immutable platform data owned by the platform
// device, so borrowing it as `&PlatformData` is sound for the lifetime of the platform device.
unsafe impl PlatData for PlatformData {
    type Borrowed<'a> = &'a Self;

    unsafe fn from_raw<'a>(ptr: *const c_void) -> Option<Self::Borrowed<'a>> {
        if ptr.is_null() {
            None
        } else {
            // SAFETY: Guaranteed by the safety contract of `PlatData::from_raw`.
            Some(unsafe { &*ptr.cast() })
        }
    }
}

impl PlatformData {
    fn as_ref(&self) -> &bindings::cpufreq_dt_platform_data {
        // SAFETY: `PlatformData` is a transparent wrapper around a valid C struct.
        unsafe { &*self.0.get().cast_const() }
    }

    /// Returns whether the driver should advertise per-policy governors.
    pub fn have_governor_per_policy(&self) -> bool {
        self.as_ref().have_governor_per_policy
    }

    /// Copies the callback plumbing into an owned, long-lived configuration object.
    pub fn copy_callbacks(&self) -> PlatformCallbacks {
        PlatformCallbacks {
            have_governor_per_policy: self.have_governor_per_policy(),
            get_intermediate: self.as_ref().get_intermediate,
            target_intermediate: self.as_ref().target_intermediate,
            suspend: self.as_ref().suspend,
            resume: self.as_ref().resume,
        }
    }
}

impl PlatformCallbacks {
    /// Returns whether the driver should advertise per-policy governors.
    pub fn have_governor_per_policy(&self) -> bool {
        self.have_governor_per_policy
    }

    /// Returns the intermediate frequency in kHz, or `0` when no callback is configured.
    pub fn get_intermediate_khz(&self, policy: &mut Policy, index: TableIndex) -> u32 {
        match self.get_intermediate {
            Some(cb) => unsafe { cb(policy.as_raw(), usize::from(index) as u32) },
            None => 0,
        }
    }

    /// Runs the target-intermediate callback when configured.
    pub fn target_intermediate(&self, policy: &mut Policy, index: TableIndex) -> Result {
        match self.target_intermediate {
            Some(cb) => to_result(unsafe { cb(policy.as_raw(), usize::from(index) as u32) }),
            None => Ok(()),
        }
    }

    /// Runs the suspend callback when configured.
    pub fn suspend(&self, policy: &mut Policy) -> Result {
        match self.suspend {
            Some(cb) => to_result(unsafe { cb(policy.as_raw()) }),
            None => policy.generic_suspend(),
        }
    }

    /// Runs the resume callback when configured.
    pub fn resume(&self, policy: &mut Policy) -> Result {
        match self.resume {
            Some(cb) => to_result(unsafe { cb(policy.as_raw()) }),
            None => Ok(()),
        }
    }
}

/// Registers a `cpufreq-dt` platform device under the given raw parent device pointer.
///
/// This mirrors the C helper exported by `cpufreq-dt.c` and keeps the raw-pointer handling out of
/// the driver body.
pub fn register_pdev_from_raw_device(
    parent: *mut bindings::device,
) -> *mut bindings::platform_device {
    let info = bindings::platform_device_info {
        parent,
        name: c"cpufreq-dt".as_ptr(),
        ..unsafe { core::mem::zeroed() }
    };

    unsafe { bindings::platform_device_register_full(&info) }
}

#[cfg(CONFIG_CPUFREQ_DT_RUST)]
#[export]
pub unsafe extern "C" fn cpufreq_dt_pdev_register(
    dev: *mut bindings::device,
) -> *mut bindings::platform_device {
    register_pdev_from_raw_device(dev)
}
