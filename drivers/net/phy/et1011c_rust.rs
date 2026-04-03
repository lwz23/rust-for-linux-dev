// SPDX-License-Identifier: GPL-2.0
// Copyright (C) 2026 Wenzhao Liao

#![forbid(unsafe_code)]

//! Rust LSI ET1011C PHY driver.
//!
//! C version of this driver: [`drivers/net/phy/et1011c.c`](./et1011c.c)

use kernel::{
    net::phy::{self, reg::C22, DeviceId, Driver},
    prelude::*,
    uapi,
};

kernel::module_phy_driver! {
    drivers: [PhyET1011C],
    device_table: [
        DeviceId::new_with_driver::<PhyET1011C>(),
    ],
    name: "et1011c_rust",
    authors: ["Wenzhao Liao <wenzhaoliao@ruc.edu.cn>"],
    description: "LSI ET1011C PHY driver in Rust",
    license: "GPL",
}

const ET1011C_STATUS_REG: C22 = C22::vendor_specific::<0x1a>();
const ET1011C_CONFIG_REG: C22 = C22::vendor_specific::<0x16>();

const ET1011C_SPEED_MASK: u16 = 0x0300;
const ET1011C_GIGABIT_SPEED: u16 = 0x0200;
const ET1011C_TX_FIFO_MASK: u16 = 0x3000;
const ET1011C_TX_FIFO_DEPTH_16: u16 = 0x1000;
const ET1011C_GMII_INTERFACE: u16 = 0x0002;
const ET1011C_SYS_CLK_EN: u16 = 0x0010;

const BMCR_SPEED1000: u16 = uapi::BMCR_SPEED1000 as u16;
const BMCR_FULLDPLX: u16 = uapi::BMCR_FULLDPLX as u16;
const BMCR_ANENABLE: u16 = uapi::BMCR_ANENABLE as u16;
const BMCR_SPEED100: u16 = uapi::BMCR_SPEED100 as u16;
const BMCR_RESET: u16 = uapi::BMCR_RESET as u16;

struct PhyET1011C;

#[vtable]
impl Driver for PhyET1011C {
    const NAME: &'static CStr = c"ET1011C";
    const PHY_DEVICE_ID: DeviceId = DeviceId::new_with_custom_mask(0x0282f014, 0xfffffff0);

    fn config_aneg(dev: &mut phy::Device) -> Result {
        let ctl = dev.read(C22::BMCR)?;
        let ctl = ctl & !(BMCR_FULLDPLX | BMCR_SPEED100 | BMCR_SPEED1000 | BMCR_ANENABLE);

        // Match the C driver reset-before-genphy flow without exposing PHYLIB FFI here.
        dev.write(C22::BMCR, ctl | BMCR_RESET)?;
        dev.genphy_config_aneg()
    }

    fn read_status(dev: &mut phy::Device) -> Result<u16> {
        let ret = dev.genphy_read_status::<C22>()?;
        let status = dev.read(ET1011C_STATUS_REG)?;

        // The C driver caches the previous speed in a global `static int`. The Rust
        // version keeps the logic local to the callback and only rewrites the vendor
        // config register when the hardware currently reports gigabit mode.
        if status & ET1011C_SPEED_MASK == ET1011C_GIGABIT_SPEED {
            let config = dev.read(ET1011C_CONFIG_REG)?;
            let config = (config & !ET1011C_TX_FIFO_MASK)
                | ET1011C_GMII_INTERFACE
                | ET1011C_SYS_CLK_EN
                | ET1011C_TX_FIFO_DEPTH_16;
            dev.write(ET1011C_CONFIG_REG, config)?;
        }

        Ok(ret)
    }
}
