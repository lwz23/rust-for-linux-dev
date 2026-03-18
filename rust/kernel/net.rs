// SPDX-License-Identifier: GPL-2.0

//! Networking.

pub mod netdevice;
#[cfg(CONFIG_RUST_PHYLIB_ABSTRACTIONS)]
pub mod phy;
pub mod rtnl;
pub mod skbuff;

pub use netdevice::{
    device_flags, features, hardware, link_attrs, netlink, priv_flags, stat_type, DeviceRef,
    LStatsHandle, LinkStats64, NetDevice, NetlinkTapHandle, SetupContext,
};
pub use rtnl::{AttrTable, Driver, ExtAck, Registration, TxStatus};
pub use skbuff::SkBuff;
