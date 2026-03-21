// SPDX-License-Identifier: GPL-2.0

use super::{build_live_device, DeviceConfig, IrqMode, LiveDevice};
use core::fmt::{self, Write};
use core::str;
use kernel::{
    bindings, c_str, configfs as kconfigfs, new_mutex, prelude::*, str::CString, sync::Mutex,
};

pub(crate) fn new_subsystem() -> impl PinInit<kconfigfs::Subsystem<Root>, Error> {
    kconfigfs::Subsystem::new(c_str!("rnull"), root_item_type(), Root::new())
}

#[pin_data]
pub(crate) struct Root {}

impl Root {
    fn new() -> impl PinInit<Self, Error> {
        try_pin_init!(Self {})
    }
}

#[pin_data]
pub(crate) struct RNullDevice {
    name: CString,
    #[pin]
    config: Mutex<DeviceConfig>,
    #[pin]
    live: Mutex<Option<LiveDevice>>,
}

impl RNullDevice {
    fn new(name: CString) -> impl PinInit<Self, Error> {
        try_pin_init!(Self {
            name,
            config <- new_mutex!(DeviceConfig::default(), "RNullDevice::config"),
            live <- new_mutex!(None::<LiveDevice>, "RNullDevice::live"),
        })
    }

    fn is_powered(&self) -> bool {
        self.live.lock().is_some()
    }

    fn with_config<R>(&self, f: impl FnOnce(DeviceConfig) -> R) -> R {
        let config = *self.config.lock();
        f(config)
    }

    fn update_config(&self, update: impl FnOnce(&mut DeviceConfig) -> Result) -> Result {
        let live = self.live.lock();
        if live.is_some() {
            return Err(EBUSY);
        }
        drop(live);

        let mut config = self.config.lock();
        update(&mut config)
    }

    fn set_power(&self, new_power: bool) -> Result {
        let mut live = self.live.lock();
        if new_power {
            if live.is_some() {
                return Ok(());
            }

            let config = *self.config.lock();
            *live = Some(build_live_device(config, &self.name)?);
        } else {
            live.take();
        }

        Ok(())
    }
}

#[vtable]
impl kconfigfs::GroupOperations for Root {
    type Child = RNullDevice;

    fn make_group(
        &self,
        name: &kernel::str::CStr,
    ) -> Result<impl PinInit<kconfigfs::Group<Self::Child>, Error>> {
        let group_name = CString::try_from(name).map_err(|_| ENOMEM)?;
        let device_name = CString::try_from(name).map_err(|_| ENOMEM)?;
        Ok(kconfigfs::Group::new(
            group_name,
            device_item_type(),
            RNullDevice::new(device_name),
        ))
    }

    fn drop_item(&self, child: &kconfigfs::Group<Self::Child>) {
        let _ = child.data().set_power(false);
    }
}

struct PageWriter<'a> {
    page: &'a mut [u8; bindings::PAGE_SIZE as usize],
    len: usize,
}

impl<'a> PageWriter<'a> {
    fn new(page: &'a mut [u8; bindings::PAGE_SIZE as usize]) -> Self {
        Self { page, len: 0 }
    }

    fn finish(self) -> usize {
        self.len
    }
}

impl Write for PageWriter<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let remaining = self.page.len().saturating_sub(self.len);
        if s.len() > remaining {
            return Err(fmt::Error);
        }

        let end = self.len + s.len();
        self.page[self.len..end].copy_from_slice(s.as_bytes());
        self.len = end;
        Ok(())
    }
}

fn write_page(
    page: &mut [u8; bindings::PAGE_SIZE as usize],
    args: fmt::Arguments<'_>,
) -> Result<usize> {
    let mut writer = PageWriter::new(page);
    writer.write_fmt(args).map_err(|_| EINVAL)?;
    Ok(writer.finish())
}

fn parse_input(page: &[u8]) -> Result<&str> {
    str::from_utf8(page).map_err(|_| EINVAL).map(str::trim)
}

fn parse_u32(page: &[u8]) -> Result<u32> {
    parse_input(page)?.parse::<u32>().map_err(|_| EINVAL)
}

fn parse_u64(page: &[u8]) -> Result<u64> {
    parse_input(page)?.parse::<u64>().map_err(|_| EINVAL)
}

fn parse_bool(page: &[u8]) -> Result<bool> {
    match parse_input(page)? {
        "1" | "y" | "Y" | "yes" | "on" | "true" => Ok(true),
        "0" | "n" | "N" | "no" | "off" | "false" => Ok(false),
        _ => Err(EINVAL),
    }
}

fn validate_block_size(block_size: u32) -> Result {
    if !(512..=bindings::PAGE_SIZE as u32).contains(&block_size) || !block_size.is_power_of_two() {
        return Err(EINVAL);
    }

    Ok(())
}

#[vtable]
impl kconfigfs::AttributeOperations<0> for Root {
    type Data = Root;

    fn show(_data: &Root, page: &mut [u8; bindings::PAGE_SIZE as usize]) -> Result<usize> {
        write_page(
            page,
            format_args!("blocksize,irqmode,power,rotational,size\n"),
        )
    }
}

#[vtable]
impl kconfigfs::AttributeOperations<0> for RNullDevice {
    type Data = RNullDevice;

    fn show(data: &RNullDevice, page: &mut [u8; bindings::PAGE_SIZE as usize]) -> Result<usize> {
        data.with_config(|config| write_page(page, format_args!("{}\n", config.size_mib)))
    }

    fn store(data: &RNullDevice, page: &[u8]) -> Result {
        let size_mib = parse_u64(page)?;
        data.update_config(|config| {
            config.size_mib = size_mib;
            Ok(())
        })
    }
}

#[vtable]
impl kconfigfs::AttributeOperations<1> for RNullDevice {
    type Data = RNullDevice;

    fn show(data: &RNullDevice, page: &mut [u8; bindings::PAGE_SIZE as usize]) -> Result<usize> {
        data.with_config(|config| write_page(page, format_args!("{}\n", config.block_size)))
    }

    fn store(data: &RNullDevice, page: &[u8]) -> Result {
        let block_size = parse_u32(page)?;
        validate_block_size(block_size)?;
        data.update_config(|config| {
            config.block_size = block_size;
            Ok(())
        })
    }
}

#[vtable]
impl kconfigfs::AttributeOperations<2> for RNullDevice {
    type Data = RNullDevice;

    fn show(data: &RNullDevice, page: &mut [u8; bindings::PAGE_SIZE as usize]) -> Result<usize> {
        data.with_config(|config| {
            write_page(page, format_args!("{}\n", u8::from(config.rotational)))
        })
    }

    fn store(data: &RNullDevice, page: &[u8]) -> Result {
        let rotational = parse_bool(page)?;
        data.update_config(|config| {
            config.rotational = rotational;
            Ok(())
        })
    }
}

#[vtable]
impl kconfigfs::AttributeOperations<3> for RNullDevice {
    type Data = RNullDevice;

    fn show(data: &RNullDevice, page: &mut [u8; bindings::PAGE_SIZE as usize]) -> Result<usize> {
        data.with_config(|config| write_page(page, format_args!("{}\n", config.irqmode.as_u32())))
    }

    fn store(data: &RNullDevice, page: &[u8]) -> Result {
        let irqmode = IrqMode::try_from_u32(parse_u32(page)?)?;
        data.update_config(|config| {
            config.irqmode = irqmode;
            Ok(())
        })
    }
}

#[vtable]
impl kconfigfs::AttributeOperations<4> for RNullDevice {
    type Data = RNullDevice;

    fn show(data: &RNullDevice, page: &mut [u8; bindings::PAGE_SIZE as usize]) -> Result<usize> {
        write_page(page, format_args!("{}\n", u8::from(data.is_powered())))
    }

    fn store(data: &RNullDevice, page: &[u8]) -> Result {
        data.set_power(parse_bool(page)?)
    }
}

static ROOT_FEATURES_ATTR: kconfigfs::Attribute<0, Root, Root> =
    kconfigfs::Attribute::new(c_str!("features"));
static ROOT_ATTRS: kconfigfs::AttributeList<2, Root> =
    kconfigfs::AttributeList::from_raw([ROOT_FEATURES_ATTR.as_ptr(), core::ptr::null_mut()]);
static ROOT_ITEM_TYPE: kconfigfs::ItemType<kconfigfs::Subsystem<Root>, Root> =
    kconfigfs::ItemType::<kconfigfs::Subsystem<Root>, Root>::new_with_child_ctor::<2, RNullDevice>(
        &ROOT_ATTRS,
    );

static DEVICE_SIZE_ATTR: kconfigfs::Attribute<0, RNullDevice, RNullDevice> =
    kconfigfs::Attribute::new(c_str!("size"));
static DEVICE_BLOCKSIZE_ATTR: kconfigfs::Attribute<1, RNullDevice, RNullDevice> =
    kconfigfs::Attribute::new(c_str!("blocksize"));
static DEVICE_ROTATIONAL_ATTR: kconfigfs::Attribute<2, RNullDevice, RNullDevice> =
    kconfigfs::Attribute::new(c_str!("rotational"));
static DEVICE_IRQMODE_ATTR: kconfigfs::Attribute<3, RNullDevice, RNullDevice> =
    kconfigfs::Attribute::new(c_str!("irqmode"));
static DEVICE_POWER_ATTR: kconfigfs::Attribute<4, RNullDevice, RNullDevice> =
    kconfigfs::Attribute::new(c_str!("power"));
static DEVICE_ATTRS: kconfigfs::AttributeList<6, RNullDevice> =
    kconfigfs::AttributeList::from_raw([
        DEVICE_SIZE_ATTR.as_ptr(),
        DEVICE_BLOCKSIZE_ATTR.as_ptr(),
        DEVICE_ROTATIONAL_ATTR.as_ptr(),
        DEVICE_IRQMODE_ATTR.as_ptr(),
        DEVICE_POWER_ATTR.as_ptr(),
        core::ptr::null_mut(),
    ]);
static DEVICE_ITEM_TYPE: kconfigfs::ItemType<kconfigfs::Group<RNullDevice>, RNullDevice> =
    kconfigfs::ItemType::<kconfigfs::Group<RNullDevice>, RNullDevice>::new::<6>(&DEVICE_ATTRS);

fn root_item_type() -> &'static kconfigfs::ItemType<kconfigfs::Subsystem<Root>, Root> {
    &ROOT_ITEM_TYPE
}

fn device_item_type() -> &'static kconfigfs::ItemType<kconfigfs::Group<RNullDevice>, RNullDevice> {
    &DEVICE_ITEM_TYPE
}
