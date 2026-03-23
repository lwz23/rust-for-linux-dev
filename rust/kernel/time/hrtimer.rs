// SPDX-License-Identifier: GPL-2.0

//! Minimal intrusive high resolution timer support.
//!
//! This wrapper intentionally supports the subset needed by the rnull port:
//! CLOCK_MONOTONIC relative timers whose owners are kept alive by `Arc`.
//!
//! C header: [`include/linux/hrtimer.h`](srctree/include/linux/hrtimer.h)

use super::Ktime;
use crate::{bindings, init::PinInit, prelude::*, sync::{Arc, ArcBorrow}, types::Opaque};
use core::{marker::PhantomData, ptr::NonNull};

/// Restart policy returned by timer callbacks.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(u32)]
pub enum HrTimerRestart {
    /// Stop the timer.
    NoRestart = bindings::hrtimer_restart_HRTIMER_NORESTART,
    /// Restart the timer.
    Restart = bindings::hrtimer_restart_HRTIMER_RESTART,
}

impl HrTimerRestart {
    fn to_raw(self) -> bindings::hrtimer_restart {
        self as bindings::hrtimer_restart
    }
}

/// Intrusive timer node embedded in an owner object.
#[pin_data]
#[repr(C)]
pub struct HrTimer<T> {
    #[pin]
    timer: Opaque<bindings::hrtimer>,
    _p: PhantomData<T>,
}

// SAFETY: The C timer core serializes timer operations.
unsafe impl<T> Send for HrTimer<T> {}
unsafe impl<T> Sync for HrTimer<T> {}

impl<T> HrTimer<T>
where
    T: HrTimerCallback,
{
    /// Creates a new stopped timer.
    pub fn new() -> impl PinInit<Self> {
        pin_init!(Self {
            timer <- Opaque::ffi_init(|slot: *mut bindings::hrtimer| {
                // SAFETY: `slot` points to valid uninitialized storage and
                // `hrtimer_init` initializes it in place.
                unsafe {
                    bindings::hrtimer_init(
                        slot,
                        bindings::CLOCK_MONOTONIC as bindings::clockid_t,
                        bindings::hrtimer_mode_HRTIMER_MODE_REL,
                    );
                    (*slot).function = Some(Self::callback);
                }
            }),
            _p: PhantomData,
        })
    }

    /// Returns whether the timer is currently active.
    pub fn is_active(&self) -> bool {
        // SAFETY: `self` always points to an initialized timer.
        unsafe { bindings::hrtimer_active(self.timer.get()) }
    }

    /// Cancels the timer, waiting for a running callback if needed.
    pub fn cancel(&self) -> bool {
        // SAFETY: `self` always points to an initialized timer.
        unsafe { bindings::hrtimer_cancel(self.timer.get()) != 0 }
    }

    unsafe fn raw_get(this: *const Self) -> *mut bindings::hrtimer {
        // SAFETY: The caller guarantees `this` points to a live `HrTimer`.
        unsafe { Opaque::raw_get(core::ptr::addr_of!((*this).timer)) }
    }

    unsafe extern "C" fn callback(timer: *mut bindings::hrtimer) -> bindings::hrtimer_restart {
        let timer = timer.cast::<HrTimer<T>>();

        // SAFETY: The hrtimer subsystem invokes the callback only for a timer
        // previously initialized from this wrapper.
        let owner = unsafe { T::timer_container_of(timer) };
        // SAFETY: The owner is kept alive by an `Arc` held by the driver while
        // the timer is armed.
        let owner = unsafe { ArcBorrow::from_raw(owner) };
        // SAFETY: We are in the timer callback context and `timer` is valid.
        let mut ctx = unsafe { HrTimerCallbackContext::from_raw(timer) };

        T::run(owner, &mut ctx).to_raw()
    }
}

/// Callback privilege available only while inside a timer callback.
pub struct HrTimerCallbackContext<'a, T>(NonNull<HrTimer<T>>, PhantomData<&'a ()>);

impl<'a, T> HrTimerCallbackContext<'a, T>
where
    T: HrTimerCallback,
{
    unsafe fn from_raw(timer: *mut HrTimer<T>) -> Self {
        // SAFETY: The caller guarantees `timer` is valid for the callback
        // duration.
        Self(unsafe { NonNull::new_unchecked(timer) }, PhantomData)
    }

    /// Forwards the timer from the current time.
    pub fn forward_now(&mut self, interval: Ktime) -> u64 {
        // SAFETY: By type invariant we are currently running inside this timer
        // callback and therefore may forward it.
        unsafe { bindings::hrtimer_forward_now(HrTimer::<T>::raw_get(self.0.as_ptr()), interval.to_ns()) }
    }
}

/// Trait implemented by objects embedding an [`HrTimer`].
pub unsafe trait HasHrTimer: Sized {
    /// Returns the embedded timer field.
    ///
    /// # Safety
    ///
    /// `this` must point to a live owner object.
    unsafe fn raw_get_timer(this: *const Self) -> *const HrTimer<Self>;

    /// Recovers the owner pointer from an embedded timer pointer.
    ///
    /// # Safety
    ///
    /// `timer` must be the timer field embedded inside `Self`.
    unsafe fn timer_container_of(timer: *mut HrTimer<Self>) -> *const Self;
}

/// Timer callback implemented by owners embedding an [`HrTimer`].
///
/// The owner must be reference counted by `Arc` while the timer is armed.
pub trait HrTimerCallback: HasHrTimer + Send + Sync + Sized + 'static {

    /// Executes when the timer fires.
    fn run(this: ArcBorrow<'_, Self>, ctx: &mut HrTimerCallbackContext<'_, Self>) -> HrTimerRestart;

    /// Starts the timer relative to now.
    fn start(this: &Arc<Self>, expires: Ktime) {
        let this_ptr = &**this as *const Self;
        // SAFETY: `this_ptr` comes from a live `Arc<Self>`, which the caller
        // keeps alive while the timer is armed.
        let timer = unsafe { <Self as HasHrTimer>::raw_get_timer(this_ptr) };
        // SAFETY: `timer` points to an initialized hrtimer embedded in `Self`.
        unsafe {
            bindings::hrtimer_start(
                HrTimer::<Self>::raw_get(timer),
                expires.to_ns(),
                bindings::hrtimer_mode_HRTIMER_MODE_REL,
            )
        };
    }

    /// Cancels the timer.
    fn cancel(this: &Self) -> bool {
        // SAFETY: `this` is a live owner object.
        let timer = unsafe { <Self as HasHrTimer>::raw_get_timer(this) };
        // SAFETY: `timer` points to an initialized hrtimer embedded in `Self`.
        unsafe { bindings::hrtimer_cancel(HrTimer::<Self>::raw_get(timer)) != 0 }
    }

    /// Returns whether the timer is active.
    fn is_active(this: &Self) -> bool {
        // SAFETY: `this` is a live owner object.
        let timer = unsafe { <Self as HasHrTimer>::raw_get_timer(this) };
        // SAFETY: `timer` points to an initialized hrtimer embedded in `Self`.
        unsafe { bindings::hrtimer_active(HrTimer::<Self>::raw_get(timer)) }
    }
}

/// Implements [`HrTimerCallback`] field access for the common `self.timer`
/// embedding pattern.
#[macro_export]
macro_rules! impl_has_hr_timer {
    (
        impl$(<$($generics:tt)*>)?
            HrTimerCallback
            for $self:ty
        {
            field: self.$field:ident $(,)?
        }
    ) => {
        // SAFETY: The generated field projections and `container_of!` call all
        // refer to the same embedded `HrTimer`.
        unsafe impl$(<$($generics)*>)? $crate::time::hrtimer::HasHrTimer for $self {
            #[inline]
            unsafe fn raw_get_timer(
                this: *const Self,
            ) -> *const $crate::time::hrtimer::HrTimer<Self> {
                // SAFETY: The caller guarantees `this` is valid.
                unsafe { ::core::ptr::addr_of!((*this).$field) }
            }

            #[inline]
            unsafe fn timer_container_of(
                timer: *mut $crate::time::hrtimer::HrTimer<Self>,
            ) -> *const Self {
                // SAFETY: The caller guarantees `timer` is embedded in `Self`.
                unsafe { $crate::container_of!(timer, Self, $field) }
            }
        }
    };
}
