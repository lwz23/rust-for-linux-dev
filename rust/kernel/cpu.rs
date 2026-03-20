// SPDX-License-Identifier: GPL-2.0

//! Generic CPU definitions.
//!
//! C header: [`include/linux/cpu.h`](srctree/include/linux/cpu.h)

use crate::{
    bindings,
    device::Device,
    error::Result,
    prelude::ENODEV,
};

/// Creates a new instance of CPU's device.
///
/// # Safety
///
/// Reference counting is not implemented for the CPU device in the C code. When a CPU is
/// hot-unplugged, the corresponding CPU device is unregistered, but its associated memory
/// is not freed.
///
/// Callers must ensure that the CPU device is not used after it has been unregistered.
/// This can be achieved, for example, by registering a CPU hotplug notifier and removing
/// any references to the CPU device within the notifier's callback.
pub unsafe fn from_cpu(cpu: u32) -> Result<&'static Device> {
    // SAFETY: It is safe to call `get_cpu_device()` for any CPU.
    let ptr = unsafe { bindings::get_cpu_device(cpu) };
    if ptr.is_null() {
        return Err(ENODEV);
    }

    // SAFETY: The pointer returned by `get_cpu_device()`, if not `NULL`, is a valid pointer to
    // a `struct device` and is never freed by the C code.
    Ok(unsafe { Device::as_ref(ptr) })
}

/// Calls `f` with the device backing `cpu`.
///
/// This is a narrow safe wrapper for CPU-device lookups that keeps the borrowed device reference
/// scoped to the callback instead of letting it escape into long-lived state.
#[inline]
pub fn with_device<T>(cpu: u32, f: impl for<'a> FnOnce(&'a Device) -> Result<T>) -> Result<T> {
    // SAFETY: The reference does not escape the callback and `get_cpu_device()` never returns
    // freed memory; this is suitable for short-lived setup operations.
    let dev = unsafe { from_cpu(cpu)? };
    f(dev)
}
