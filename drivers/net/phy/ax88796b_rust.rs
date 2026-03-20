// SPDX-License-Identifier: GPL-2.0

//! Rust blind-write benchmark for the Asix PHY driver.

use kernel::bindings;
use kernel::net::phy::{
    self,
    reg::C22,
    Driver,
};
use kernel::prelude::*;

const PHY_ID_ASIX_AX88772A: u32 = 0x003b1861;
const PHY_ID_ASIX_AX88772C: u32 = 0x003b1881;
const PHY_ID_ASIX_AX88796B: u32 = 0x003b1841;

kernel::module_phy_driver! {
    drivers: [AsixAx88772a, AsixAx88772c, AsixAx88796b],
    device_table: [
        phy::DeviceId::new_with_driver::<AsixAx88772a>(),
        phy::DeviceId::new_with_driver::<AsixAx88772c>(),
        phy::DeviceId::new_with_driver::<AsixAx88796b>(),
    ],
    name: "ax88796b_rust",
    authors: ["lwz"],
    description: "Rust blind-write Asix PHY driver",
    license: "GPL",
}

struct AsixAx88772a;
struct AsixAx88772c;
struct AsixAx88796b;

fn asix_soft_reset(dev: &mut phy::Device) -> Result {
    dev.write(C22::BMCR, 0)?;
    dev.genphy_soft_reset()
}

fn asix_ax88772a_read_status(dev: &mut phy::Device) -> Result<u16> {
    dev.genphy_update_link()?;
    if !dev.is_link_up() {
        return Ok(0);
    }

    let val = dev.read(C22::BMCR)?;
    if (val & bindings::BMCR_SPEED100 as u16) != 0 {
        dev.set_speed(bindings::SPEED_100);
    } else {
        dev.set_speed(bindings::SPEED_10);
    }

    if (val & bindings::BMCR_FULLDPLX as u16) != 0 {
        dev.set_duplex(phy::DuplexMode::Full);
    } else {
        dev.set_duplex(phy::DuplexMode::Half);
    }

    dev.genphy_read_lpa()?;
    if dev.is_autoneg_enabled() && dev.is_autoneg_completed() {
        dev.resolve_aneg_linkmode();
    }

    Ok(0)
}

fn asix_ax88772a_link_change_notify(dev: &mut phy::Device) {
    if dev.state() == phy::DeviceState::NoLink {
        let _ = dev.init_hw();
        let _ = dev.start_aneg();
    }
}

#[vtable]
impl Driver for AsixAx88772a {
    const NAME: &'static CStr = c"Asix Electronics AX88772A";
    const PHY_DEVICE_ID: phy::DeviceId = phy::DeviceId::new_with_exact_mask(PHY_ID_ASIX_AX88772A);
    const FLAGS: u32 = bindings::PHY_IS_INTERNAL;

    fn soft_reset(dev: &mut phy::Device) -> Result {
        asix_soft_reset(dev)
    }

    fn suspend(dev: &mut phy::Device) -> Result {
        dev.genphy_suspend()
    }

    fn resume(dev: &mut phy::Device) -> Result {
        dev.genphy_resume()
    }

    fn read_status(dev: &mut phy::Device) -> Result<u16> {
        asix_ax88772a_read_status(dev)
    }

    fn link_change_notify(dev: &mut phy::Device) {
        asix_ax88772a_link_change_notify(dev)
    }
}

#[vtable]
impl Driver for AsixAx88772c {
    const NAME: &'static CStr = c"Asix Electronics AX88772C";
    const PHY_DEVICE_ID: phy::DeviceId = phy::DeviceId::new_with_exact_mask(PHY_ID_ASIX_AX88772C);
    const FLAGS: u32 = bindings::PHY_IS_INTERNAL;

    fn soft_reset(dev: &mut phy::Device) -> Result {
        asix_soft_reset(dev)
    }

    fn suspend(dev: &mut phy::Device) -> Result {
        dev.genphy_suspend()
    }

    fn resume(dev: &mut phy::Device) -> Result {
        dev.genphy_resume()
    }
}

#[vtable]
impl Driver for AsixAx88796b {
    const NAME: &'static CStr = c"Asix Electronics AX88796B";
    const PHY_DEVICE_ID: phy::DeviceId = phy::DeviceId::new_with_model_mask(PHY_ID_ASIX_AX88796B);

    fn soft_reset(dev: &mut phy::Device) -> Result {
        asix_soft_reset(dev)
    }
}
