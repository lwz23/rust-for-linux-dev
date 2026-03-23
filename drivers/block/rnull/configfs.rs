// SPDX-License-Identifier: GPL-2.0

//! Configfs glue for the blind-first Rust null block driver.

use kernel::{
    configfs as kconfigfs,
    prelude::*,
    str::CString,
    sync::{Arc, ArcBorrow},
};

use crate::THIS_MODULE;

use super::{
    parse_badblocks, parse_bool, parse_u32, parse_u64, show_bool, show_u32,
    show_u64, Controller, DeviceInner,
};

const ATTR_SIZE: u64 = 0;
const ATTR_COMPLETION_NSEC: u64 = 1;
const ATTR_SUBMIT_QUEUES: u64 = 2;
const ATTR_POLL_QUEUES: u64 = 3;
const ATTR_HOME_NODE: u64 = 4;
const ATTR_QUEUE_MODE: u64 = 5;
const ATTR_BLOCKSIZE: u64 = 6;
const ATTR_MAX_SECTORS: u64 = 7;
const ATTR_IRQMODE: u64 = 8;
const ATTR_HW_QUEUE_DEPTH: u64 = 9;
const ATTR_BLOCKING: u64 = 10;
const ATTR_MEMORY_BACKED: u64 = 11;
const ATTR_DISCARD: u64 = 12;
const ATTR_CACHE_SIZE: u64 = 13;
const ATTR_MBPS: u64 = 14;
const ATTR_NO_SCHED: u64 = 15;
const ATTR_SHARED_TAGS: u64 = 16;
const ATTR_SHARED_TAG_BITMAP: u64 = 17;
const ATTR_FUA: u64 = 18;
const ATTR_VIRT_BOUNDARY: u64 = 19;
const ATTR_POWER: u64 = 20;
const ATTR_BADBLOCKS: u64 = 21;

#[pin_data]
pub(crate) struct Root {
    controller: Arc<Controller>,
}

#[pin_data]
pub(crate) struct Device {
    device: Arc<DeviceInner>,
}

pub(crate) fn create_subsystem(
    controller: Arc<Controller>,
) -> Result<Pin<Box<kconfigfs::Subsystem<Root>>>> {
    Box::pin_init(
        kconfigfs::Subsystem::new(
            kernel::c_str!("nullb"),
            root_type(),
            kernel::static_lock_class!(),
            try_pin_init!(Root { controller }),
        ),
        GFP_KERNEL,
    )
}

fn root_type() -> &'static kconfigfs::ItemType<kconfigfs::Subsystem<Root>, Root> {
    kernel::configfs_attr_list_init!(ROOT_ATTRS, (0, 0, &ROOT_FEATURES_ATTR));
    &ROOT_TYPE
}

fn device_type() -> &'static kconfigfs::ItemType<kconfigfs::Group<Device>, Device> {
    kernel::configfs_attr_list_init!(
        DEVICE_ATTRS,
        (0, 0, &DEVICE_SIZE_ATTR),
        (1, 1, &DEVICE_COMPLETION_NSEC_ATTR),
        (2, 2, &DEVICE_SUBMIT_QUEUES_ATTR),
        (3, 3, &DEVICE_POLL_QUEUES_ATTR),
        (4, 4, &DEVICE_HOME_NODE_ATTR),
        (5, 5, &DEVICE_QUEUE_MODE_ATTR),
        (6, 6, &DEVICE_BLOCKSIZE_ATTR),
        (7, 7, &DEVICE_MAX_SECTORS_ATTR),
        (8, 8, &DEVICE_IRQMODE_ATTR),
        (9, 9, &DEVICE_HW_QUEUE_DEPTH_ATTR),
        (10, 10, &DEVICE_BLOCKING_ATTR),
        (11, 11, &DEVICE_MEMORY_BACKED_ATTR),
        (12, 12, &DEVICE_DISCARD_ATTR),
        (13, 13, &DEVICE_CACHE_SIZE_ATTR),
        (14, 14, &DEVICE_MBPS_ATTR),
        (15, 15, &DEVICE_NO_SCHED_ATTR),
        (16, 16, &DEVICE_SHARED_TAGS_ATTR),
        (17, 17, &DEVICE_SHARED_TAG_BITMAP_ATTR),
        (18, 18, &DEVICE_FUA_ATTR),
        (19, 19, &DEVICE_VIRT_BOUNDARY_ATTR),
        (20, 20, &DEVICE_POWER_ATTR),
        (21, 21, &DEVICE_BADBLOCKS_ATTR),
    );
    &DEVICE_TYPE
}

static ROOT_FEATURES_ATTR: kconfigfs::Attribute<0, Root, Root> =
    kconfigfs::Attribute::new(kernel::c_str!("features"));

static ROOT_ATTRS: kconfigfs::AttributeList<2, Root> = kconfigfs::AttributeList::new();

static ROOT_TYPE: kconfigfs::ItemType<kconfigfs::Subsystem<Root>, Root> =
    kconfigfs::ItemType::<kconfigfs::Subsystem<Root>, Root>::new_with_child_ctor::<2, Device>(
        &THIS_MODULE,
        &ROOT_ATTRS,
    );

static DEVICE_SIZE_ATTR: kconfigfs::Attribute<0, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("size"));
static DEVICE_COMPLETION_NSEC_ATTR: kconfigfs::Attribute<1, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("completion_nsec"));
static DEVICE_SUBMIT_QUEUES_ATTR: kconfigfs::Attribute<2, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("submit_queues"));
static DEVICE_POLL_QUEUES_ATTR: kconfigfs::Attribute<3, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("poll_queues"));
static DEVICE_HOME_NODE_ATTR: kconfigfs::Attribute<4, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("home_node"));
static DEVICE_QUEUE_MODE_ATTR: kconfigfs::Attribute<5, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("queue_mode"));
static DEVICE_BLOCKSIZE_ATTR: kconfigfs::Attribute<6, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("blocksize"));
static DEVICE_MAX_SECTORS_ATTR: kconfigfs::Attribute<7, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("max_sectors"));
static DEVICE_IRQMODE_ATTR: kconfigfs::Attribute<8, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("irqmode"));
static DEVICE_HW_QUEUE_DEPTH_ATTR: kconfigfs::Attribute<9, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("hw_queue_depth"));
static DEVICE_BLOCKING_ATTR: kconfigfs::Attribute<10, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("blocking"));
static DEVICE_MEMORY_BACKED_ATTR: kconfigfs::Attribute<11, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("memory_backed"));
static DEVICE_DISCARD_ATTR: kconfigfs::Attribute<12, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("discard"));
static DEVICE_CACHE_SIZE_ATTR: kconfigfs::Attribute<13, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("cache_size"));
static DEVICE_MBPS_ATTR: kconfigfs::Attribute<14, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("mbps"));
static DEVICE_NO_SCHED_ATTR: kconfigfs::Attribute<15, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("no_sched"));
static DEVICE_SHARED_TAGS_ATTR: kconfigfs::Attribute<16, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("shared_tags"));
static DEVICE_SHARED_TAG_BITMAP_ATTR: kconfigfs::Attribute<17, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("shared_tag_bitmap"));
static DEVICE_FUA_ATTR: kconfigfs::Attribute<18, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("fua"));
static DEVICE_VIRT_BOUNDARY_ATTR: kconfigfs::Attribute<19, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("virt_boundary"));
static DEVICE_POWER_ATTR: kconfigfs::Attribute<20, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("power"));
static DEVICE_BADBLOCKS_ATTR: kconfigfs::Attribute<21, Device, Device> =
    kconfigfs::Attribute::new(kernel::c_str!("badblocks"));

static DEVICE_ATTRS: kconfigfs::AttributeList<23, Device> = kconfigfs::AttributeList::new();

static DEVICE_TYPE: kconfigfs::ItemType<kconfigfs::Group<Device>, Device> =
    kconfigfs::ItemType::<kconfigfs::Group<Device>, Device>::new::<23>(
        &THIS_MODULE,
        &DEVICE_ATTRS,
    );

#[vtable]
impl kconfigfs::AttributeOperations<0> for Root {
    type Data = Root;

    fn show(_data: &Self::Data, page: &mut [u8; kernel::bindings::PAGE_SIZE]) -> Result<usize> {
        super::write_page(
            page,
            format_args!(
                "memory_backed,cache_size,fua,mbps,badblocks,submit_queues,poll_queues,irqmode,shared_tags,shared_tag_bitmap\n"
            ),
        )
    }
}

#[vtable]
impl kconfigfs::GroupOperations for Root {
    type Child = Device;

    fn make_group(
        &self,
        name: &CStr,
    ) -> Result<impl PinInit<kconfigfs::Group<Self::Child>, Error>> {
        let group_name = name.to_cstring()?;
        let device_name = CString::try_from(&*group_name)?;
        let device = self.controller.create_configfs_device(device_name)?;

        Ok(kconfigfs::Group::new(
            group_name,
            device_type(),
            try_pin_init!(Device { device }),
        ))
    }

    fn drop_item(&self, child: ArcBorrow<'_, kconfigfs::Group<Self::Child>>) {
        let device = child.data().device.clone();
        self.controller.drop_device(&device);
    }
}

macro_rules! attr_rw_u64 {
    ($id:expr, $field:ident) => {
        #[vtable]
        impl kconfigfs::AttributeOperations<{ $id }> for Device {
            type Data = Device;

            fn show(data: &Self::Data, page: &mut [u8; kernel::bindings::PAGE_SIZE]) -> Result<usize> {
                show_u64(page, data.device.config().$field)
            }

            fn store(data: &Self::Data, page: &[u8]) -> Result {
                let value = parse_u64(page)?;
                data.device.set_when_powered_off(|config| config.$field = value)
            }
        }
    };
}

macro_rules! attr_rw_u32 {
    ($id:expr, $field:ident) => {
        #[vtable]
        impl kconfigfs::AttributeOperations<{ $id }> for Device {
            type Data = Device;

            fn show(data: &Self::Data, page: &mut [u8; kernel::bindings::PAGE_SIZE]) -> Result<usize> {
                show_u32(page, data.device.config().$field)
            }

            fn store(data: &Self::Data, page: &[u8]) -> Result {
                let value = parse_u32(page)?;
                data.device.set_when_powered_off(|config| config.$field = value)
            }
        }
    };
}

macro_rules! attr_rw_bool {
    ($id:expr, $field:ident) => {
        #[vtable]
        impl kconfigfs::AttributeOperations<{ $id }> for Device {
            type Data = Device;

            fn show(data: &Self::Data, page: &mut [u8; kernel::bindings::PAGE_SIZE]) -> Result<usize> {
                show_bool(page, data.device.config().$field)
            }

            fn store(data: &Self::Data, page: &[u8]) -> Result {
                let value = parse_bool(page)?;
                data.device.set_when_powered_off(|config| config.$field = value)
            }
        }
    };
}

attr_rw_u64!(ATTR_SIZE, size_mib);
attr_rw_u64!(ATTR_COMPLETION_NSEC, completion_nsec);
attr_rw_u32!(ATTR_QUEUE_MODE, queue_mode);
attr_rw_u32!(ATTR_BLOCKSIZE, blocksize);
attr_rw_u32!(ATTR_MAX_SECTORS, max_sectors);
attr_rw_u32!(ATTR_IRQMODE, irqmode);
attr_rw_u32!(ATTR_HW_QUEUE_DEPTH, hw_queue_depth);
attr_rw_bool!(ATTR_BLOCKING, blocking);
attr_rw_bool!(ATTR_MEMORY_BACKED, memory_backed);
attr_rw_bool!(ATTR_DISCARD, discard);
attr_rw_u64!(ATTR_CACHE_SIZE, cache_size_mib);
attr_rw_u32!(ATTR_MBPS, mbps);
attr_rw_bool!(ATTR_NO_SCHED, no_sched);
attr_rw_bool!(ATTR_SHARED_TAGS, shared_tags);
attr_rw_bool!(ATTR_SHARED_TAG_BITMAP, shared_tag_bitmap);
attr_rw_bool!(ATTR_FUA, fua);
attr_rw_bool!(ATTR_VIRT_BOUNDARY, virt_boundary);

#[vtable]
impl kconfigfs::AttributeOperations<{ ATTR_HOME_NODE }> for Device {
    type Data = Device;

    fn show(data: &Self::Data, page: &mut [u8; kernel::bindings::PAGE_SIZE]) -> Result<usize> {
        show_u32(page, data.device.config().home_node as u32)
    }

    fn store(data: &Self::Data, page: &[u8]) -> Result {
        let value = parse_u32(page)?;
        data.device.set_when_powered_off(|config| config.home_node = value as i32)
    }
}

#[vtable]
impl kconfigfs::AttributeOperations<{ ATTR_SUBMIT_QUEUES }> for Device {
    type Data = Device;

    fn show(data: &Self::Data, page: &mut [u8; kernel::bindings::PAGE_SIZE]) -> Result<usize> {
        show_u32(page, data.device.config().submit_queues)
    }

    fn store(data: &Self::Data, page: &[u8]) -> Result {
        data.device.update_submit_queues(parse_u32(page)?)
    }
}

#[vtable]
impl kconfigfs::AttributeOperations<{ ATTR_POLL_QUEUES }> for Device {
    type Data = Device;

    fn show(data: &Self::Data, page: &mut [u8; kernel::bindings::PAGE_SIZE]) -> Result<usize> {
        show_u32(page, data.device.config().poll_queues)
    }

    fn store(data: &Self::Data, page: &[u8]) -> Result {
        data.device.update_poll_queues(parse_u32(page)?)
    }
}

#[vtable]
impl kconfigfs::AttributeOperations<{ ATTR_POWER }> for Device {
    type Data = Device;

    fn show(data: &Self::Data, page: &mut [u8; kernel::bindings::PAGE_SIZE]) -> Result<usize> {
        show_bool(page, data.device.config().power)
    }

    fn store(data: &Self::Data, page: &[u8]) -> Result {
        data.device.set_power(parse_bool(page)?)
    }
}

#[vtable]
impl kconfigfs::AttributeOperations<{ ATTR_BADBLOCKS }> for Device {
    type Data = Device;

    fn show(data: &Self::Data, page: &mut [u8; kernel::bindings::PAGE_SIZE]) -> Result<usize> {
        data.device.backing.badblocks.show(page)
    }

    fn store(data: &Self::Data, page: &[u8]) -> Result {
        data.device.update_badblocks(parse_badblocks(page)?)
    }
}
