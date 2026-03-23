// SPDX-License-Identifier: GPL-2.0

//! Block status wrapper.

use crate::bindings;

/// Safe wrapper around `blk_status_t`.
#[derive(Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct BlkStatus(bindings::blk_status_t);

impl BlkStatus {
    /// Successful completion.
    pub const OK: Self = Self(bindings::BLK_STS_OK as bindings::blk_status_t);
    /// Operation is not supported.
    pub const NOTSUPP: Self = Self(1);
    /// Request timed out.
    pub const TIMEOUT: Self = Self(2);
    /// The queue is temporarily resource constrained.
    pub const RESOURCE: Self = Self(9);
    /// The device is temporarily resource constrained.
    pub const DEV_RESOURCE: Self = Self(13);
    /// I/O failure.
    pub const IOERR: Self = Self(10);

    /// Creates a wrapper from a raw `blk_status_t`.
    pub const fn from_raw(raw: bindings::blk_status_t) -> Self {
        Self(raw)
    }

    /// Returns the raw `blk_status_t`.
    pub const fn to_raw(self) -> bindings::blk_status_t {
        self.0
    }
}
