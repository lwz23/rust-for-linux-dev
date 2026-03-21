// SPDX-License-Identifier: GPL-2.0

//! Rust null block driver subset used for blind-write calibration.

mod configfs;

use kernel::{
    alloc::flags,
    block::mq::{
        self,
        gen_disk::{GenDisk, GenDiskBuilder},
        Request, TagSet,
    },
    prelude::*,
    sync::Arc,
    types::ARef,
    workqueue,
};

const DEFAULT_SIZE_MIB: u64 = 250 * 1024;
const DEFAULT_BLOCK_SIZE: u32 = 512;
const DEFAULT_ROTATIONAL: bool = false;
const DEFAULT_QUEUE_DEPTH: u32 = 64;

module! {
    type: RNullModule,
    name: "rnull",
    author: "Codex",
    description: "Rust null block driver subset for calibration rerun",
    license: "GPL",
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IrqMode {
    None,
    Softirq,
    Timer,
}

impl Default for IrqMode {
    fn default() -> Self {
        Self::Softirq
    }
}

impl IrqMode {
    pub(crate) fn as_u32(self) -> u32 {
        match self {
            Self::None => 0,
            Self::Softirq => 1,
            Self::Timer => 2,
        }
    }

    pub(crate) fn try_from_u32(value: u32) -> Result<Self> {
        match value {
            0 => Ok(Self::None),
            1 => Ok(Self::Softirq),
            2 => Ok(Self::Timer),
            _ => Err(EINVAL),
        }
    }
}

#[derive(Clone, Copy)]
pub(crate) struct DeviceConfig {
    pub(crate) size_mib: u64,
    pub(crate) block_size: u32,
    pub(crate) rotational: bool,
    pub(crate) irqmode: IrqMode,
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            size_mib: DEFAULT_SIZE_MIB,
            block_size: DEFAULT_BLOCK_SIZE,
            rotational: DEFAULT_ROTATIONAL,
            irqmode: IrqMode::default(),
        }
    }
}

impl DeviceConfig {
    pub(crate) fn capacity_sectors(self) -> u64 {
        self.size_mib.saturating_mul(1024 * 1024 / 512)
    }
}

pub(crate) struct LiveDevice {
    _disk: GenDisk<RNullOps>,
}

struct QueueData {
    irqmode: IrqMode,
}

struct RNullOps;

#[vtable]
impl mq::Operations for RNullOps {
    type QueueData = Box<QueueData>;

    fn queue_rq(queue_data: &QueueData, rq: ARef<Request<Self>>, _is_last: bool) -> Result {
        match queue_data.irqmode {
            IrqMode::None => {
                let _ = Request::end_ok(rq);
            }
            IrqMode::Softirq => {
                Request::complete(rq);
            }
            IrqMode::Timer => {
                let deferred = rq.clone();
                if workqueue::system()
                    .try_spawn(flags::GFP_ATOMIC, move || Request::complete(deferred))
                    .is_err()
                {
                    Request::complete(rq);
                }
            }
        }

        Ok(())
    }

    fn commit_rqs(_queue_data: &QueueData) {}

    fn complete(rq: ARef<Request<Self>>) {
        let _ = Request::end_ok(rq);
    }
}

pub(crate) fn build_live_device(
    config: DeviceConfig,
    name: &kernel::str::CStr,
) -> Result<LiveDevice> {
    let tagset: Arc<TagSet<RNullOps>> =
        Arc::pin_init(TagSet::new(1, DEFAULT_QUEUE_DEPTH, 1), flags::GFP_KERNEL)?;
    let queue_data = Box::new(
        QueueData {
            irqmode: config.irqmode,
        },
        flags::GFP_KERNEL,
    )?;

    let builder = GenDiskBuilder::new()
        .logical_block_size(config.block_size)?
        .physical_block_size(config.block_size)?
        .rotational(config.rotational)
        .capacity_sectors(config.capacity_sectors());

    Ok(LiveDevice {
        _disk: builder.build(format_args!("{}", name), tagset, queue_data)?,
    })
}

struct RNullModule {
    _subsystem: Pin<Box<kernel::configfs::Subsystem<configfs::Root>>>,
}

impl kernel::Module for RNullModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        Ok(Self {
            _subsystem: Box::pin_init(configfs::new_subsystem(), flags::GFP_KERNEL)?,
        })
    }
}
