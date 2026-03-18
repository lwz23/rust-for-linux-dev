// SPDX-License-Identifier: GPL-2.0

//! Rust implementation scaffold for the nlmon rtnl-link driver.
//!
//! Reference C implementation: [`drivers/net/nlmon.c`](./nlmon.c)

use kernel::{
    error::code,
    net::{self, device_flags, features, hardware, link_attrs, netlink, priv_flags, stat_type},
    prelude::*,
};

module! {
    type: NlmonModule,
    name: "nlmon_rust",
    authors: ["lwz23"],
    description: "Rust netlink monitoring device",
    license: "GPL v2",
    alias: ["rtnl-link-nlmon"],
}

struct NlmonModule {
    _registration: net::Registration<NlmonDriver>,
}

impl kernel::Module for NlmonModule {
    fn init(module: &'static ThisModule) -> Result<Self> {
        Ok(Self {
            _registration: net::Registration::<NlmonDriver>::new(module)?,
        })
    }
}

#[derive(Default)]
struct NlmonPrivate;

struct NlmonDriver;

#[vtable]
impl net::Driver for NlmonDriver {
    type Private = NlmonPrivate;
    const KIND: &'static CStr = c"nlmon";

    fn setup(dev: &mut net::SetupContext<'_, Self>) {
        dev.set_type(hardware::NETLINK);
        dev.add_private_flags(priv_flags::NO_QUEUE);
        dev.set_lltx(true);
        dev.set_features(features::SG | features::FRAGLIST | features::HIGHDMA);
        dev.set_flags(device_flags::NOARP);
        dev.set_pcpu_stat_type(stat_type::LSTATS);
        dev.set_mtu(netlink::GOODSIZE);
        dev.set_min_mtu(netlink::HEADER_LEN);
    }

    fn open(_dev: &mut net::NetDevice<Self>) -> Result {
        Ok(())
    }

    fn stop(_dev: &mut net::NetDevice<Self>) -> Result {
        Ok(())
    }

    fn start_xmit(_skb: net::SkBuff, _dev: net::DeviceRef<Self>) -> net::TxStatus {
        net::TxStatus::OK
    }

    fn get_stats64(_dev: net::DeviceRef<Self>, _stats: &mut net::LinkStats64) {}

    fn validate(
        tb: net::AttrTable<'_>,
        _data: net::AttrTable<'_>,
        _extack: Option<&mut net::ExtAck>,
    ) -> Result {
        if tb.is_present(link_attrs::ADDRESS) {
            return Err(code::EINVAL);
        }

        Ok(())
    }

    fn get_link(_dev: net::DeviceRef<Self>) -> u32 {
        1
    }
}
