// SPDX-License-Identifier: GPL-2.0

//! Generic CPU definitions.
//!
//! C header: [`include/linux/cpu.h`](srctree/include/linux/cpu.h)

use crate::{bindings, device::Device, error::Result, prelude::ENODEV};

/// Returns the maximum number of possible CPUs in the current system configuration.
#[inline]
pub fn nr_cpu_ids() -> u32 {
    #[cfg(any(NR_CPUS_1, CONFIG_FORCE_NR_CPUS))]
    {
        bindings::NR_CPUS
    }

    #[cfg(not(any(NR_CPUS_1, CONFIG_FORCE_NR_CPUS)))]
    // SAFETY: `nr_cpu_ids` is a valid global provided by the kernel.
    unsafe {
        bindings::nr_cpu_ids
    }
}

/// The CPU ID.
///
/// Represents a CPU identifier as a wrapper around an [`u32`].
///
/// # Invariants
///
/// The CPU ID lies within the range `[0, nr_cpu_ids())`.
///
/// # Examples
///
/// ```
/// use kernel::cpu::CpuId;
///
/// let cpu = 0;
///
/// // SAFETY: 0 is always a valid CPU number.
/// let id = unsafe { CpuId::from_u32_unchecked(cpu) };
///
/// assert_eq!(id.as_u32(), cpu);
/// assert!(CpuId::from_i32(0).is_some());
/// assert!(CpuId::from_i32(-1).is_none());
/// ```
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub struct CpuId(u32);

impl CpuId {
    /// Creates a new [`CpuId`] from the given `id` without checking bounds.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `id` is a valid CPU ID (i.e., `0 <= id < nr_cpu_ids()`).
    #[inline]
    pub unsafe fn from_i32_unchecked(id: i32) -> Self {
        debug_assert!(id >= 0);
        debug_assert!((id as u32) < nr_cpu_ids());

        // INVARIANT: The function safety guarantees `id` is a valid CPU id.
        Self(id as u32)
    }

    /// Creates a new [`CpuId`] from the given `id`, checking that it is valid.
    pub fn from_i32(id: i32) -> Option<Self> {
        if id < 0 || id as u32 >= nr_cpu_ids() {
            None
        } else {
            // INVARIANT: `id` has just been checked as a valid CPU ID.
            Some(Self(id as u32))
        }
    }

    /// Creates a new [`CpuId`] from the given `id` without checking bounds.
    ///
    /// # Safety
    ///
    /// The caller must ensure that `id` is a valid CPU ID (i.e., `0 <= id < nr_cpu_ids()`).
    #[inline]
    pub unsafe fn from_u32_unchecked(id: u32) -> Self {
        debug_assert!(id < nr_cpu_ids());

        // Ensure the `id` fits in an [`i32`] as it's also representable that way.
        debug_assert!(id <= i32::MAX as u32);

        // INVARIANT: The function safety guarantees `id` is a valid CPU id.
        Self(id)
    }

    /// Creates a new [`CpuId`] from the given `id`, checking that it is valid.
    pub fn from_u32(id: u32) -> Option<Self> {
        if id >= nr_cpu_ids() {
            None
        } else {
            // INVARIANT: `id` has just been checked as a valid CPU ID.
            Some(Self(id))
        }
    }

    /// Returns CPU number.
    #[inline]
    pub fn as_u32(&self) -> u32 {
        self.0
    }

    /// Returns the ID of the CPU the code is currently running on.
    ///
    /// The returned value is considered unstable because it may change
    /// unexpectedly due to preemption or CPU migration. It should only be
    /// used when the context ensures that the task remains on the same CPU
    /// or the users could use a stale (yet valid) CPU ID.
    #[inline]
    pub fn current() -> Self {
        // SAFETY: raw_smp_processor_id() always returns a valid CPU ID.
        unsafe { Self::from_u32_unchecked(bindings::raw_smp_processor_id()) }
    }
}

impl From<CpuId> for u32 {
    fn from(id: CpuId) -> Self {
        id.as_u32()
    }
}

impl From<CpuId> for i32 {
    fn from(id: CpuId) -> Self {
        id.as_u32() as i32
    }
}

/// Iterator over CPUs currently marked present by the kernel.
pub struct PresentCpus {
    next: u32,
}

impl Iterator for PresentCpus {
    type Item = CpuId;

    fn next(&mut self) -> Option<Self::Item> {
        while self.next < nr_cpu_ids() {
            let cpu = self.next;
            self.next += 1;

            // SAFETY: `cpu` is in range by construction and `__cpu_present_mask` is a valid
            // global cpumask provided by the kernel.
            let present = unsafe {
                bindings::cpumask_test_cpu(
                    cpu as i32,
                    (&raw const bindings::__cpu_present_mask).cast_mut(),
                )
            };

            if present {
                // SAFETY: `cpu` has just been verified to be in `[0, nr_cpu_ids())`.
                return Some(unsafe { CpuId::from_u32_unchecked(cpu) });
            }
        }

        None
    }
}

/// Returns an iterator over the CPUs that are currently marked present.
#[inline]
pub fn present_cpus() -> PresentCpus {
    PresentCpus { next: 0 }
}

/// Executes a closure for each present CPU.
pub fn for_each_present(mut f: impl FnMut(CpuId) -> Result) -> Result {
    for cpu in present_cpus() {
        f(cpu)?;
    }

    Ok(())
}

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
pub unsafe fn from_cpu(cpu: CpuId) -> Result<&'static Device> {
    // SAFETY: It is safe to call `get_cpu_device()` for any CPU.
    let ptr = unsafe { bindings::get_cpu_device(u32::from(cpu)) };
    if ptr.is_null() {
        return Err(ENODEV);
    }

    // SAFETY: The pointer returned by `get_cpu_device()`, if not `NULL`, is a valid pointer to
    // a `struct device` and is never freed by the C code.
    Ok(unsafe { Device::from_raw(ptr) })
}

/// Safely borrows a CPU device for the duration of the provided closure.
///
/// This avoids exposing the raw lifetime caveat from [`from_cpu`] to driver code. The returned
/// value may not retain references to the CPU device beyond the callback's scope.
pub fn with_device<R>(cpu: CpuId, f: impl FnOnce(&Device) -> Result<R>) -> Result<R> {
    // SAFETY: The returned reference is only exposed within the callback scope.
    let dev = unsafe { from_cpu(cpu)? };
    f(dev)
}
