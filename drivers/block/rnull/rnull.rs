// SPDX-License-Identifier: GPL-2.0

//! Blind-first Rust null block driver candidate.

#![allow(stable_features)]

use alloc::vec::Vec;
use core::{
    cmp,
    fmt,
    sync::atomic::{AtomicU32, Ordering},
};

use kernel::{
    bindings,
    block::{
        badblocks::BadBlocks,
        mq::{
            self,
            gen_disk::{GenDisk, GenDiskBuilder},
            BlkStatus, QueueKind, QueueResult, Request, RequestOp, TagSet, TagSetConfig,
            TimeoutResult,
        },
    },
    new_mutex, new_spinlock,
    prelude::*,
    str::CString,
    sync::{Arc, ArcBorrow, Mutex, SpinLock},
    time::{
        self,
        hrtimer::{HrTimer, HrTimerCallback, HrTimerCallbackContext, HrTimerRestart},
        Ktime,
    },
    types::ARef,
};

mod configfs;

use self::configfs::{create_subsystem, Root};

module! {
    type: RnullModule,
    name: "rnull",
    author: "OpenAI",
    description: "Blind-first Rust null block driver validation candidate",
    license: "GPL",
}

const PAGE_BYTES: usize = bindings::PAGE_SIZE;
const SECTOR_SHIFT: u32 = 9;
const MIB: u64 = 1024 * 1024;
const DEFAULT_NR_DEVICES: u32 = 1;
const DEFAULT_SIZE_MIB: u64 = 250 * 1024;
const DEFAULT_COMPLETION_NSEC: u64 = 10_000;
const DEFAULT_BLOCK_SIZE: u32 = 512;
const DEFAULT_SUBMIT_QUEUES: u32 = 1;
const DEFAULT_POLL_QUEUES: u32 = 1;
const DEFAULT_HW_QUEUE_DEPTH: u32 = 64;
const DEFAULT_HOME_NODE: i32 = bindings::NUMA_NO_NODE;
const MAX_TRACKED_HW_QUEUES: u32 = 128;
const MAX_MBPS: u32 = 40 * 1024;
const TIMEOUT_MSECS: u32 = 5_000;

const QUEUE_MODE_BIO: u32 = 0;
const QUEUE_MODE_RQ: u32 = 1;
const QUEUE_MODE_MQ: u32 = 2;

const IRQ_MODE_NONE: u32 = 0;
const IRQ_MODE_SOFTIRQ: u32 = 1;
const IRQ_MODE_TIMER: u32 = 2;

struct RnullModule {
    controller: Arc<Controller>,
    _subsystem: Pin<Box<kernel::configfs::Subsystem<Root>>>,
}

impl kernel::Module for RnullModule {
    fn init(_module: &'static ThisModule) -> Result<Self> {
        let controller = Controller::new()?;
        let subsystem = create_subsystem(controller.clone())?;

        controller.create_default_devices(DEFAULT_NR_DEVICES)?;

        Ok(Self {
            controller,
            _subsystem: subsystem,
        })
    }
}

impl Drop for RnullModule {
    fn drop(&mut self) {
        self.controller.shutdown_all();
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
enum IrqMode {
    None,
    Softirq,
    Timer,
}

impl IrqMode {
    fn from_raw(raw: u32) -> Self {
        match raw {
            IRQ_MODE_NONE => Self::None,
            IRQ_MODE_TIMER => Self::Timer,
            _ => Self::Softirq,
        }
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct DeviceConfig {
    size_mib: u64,
    completion_nsec: u64,
    submit_queues: u32,
    poll_queues: u32,
    home_node: i32,
    queue_mode: u32,
    blocksize: u32,
    max_sectors: u32,
    irqmode: u32,
    hw_queue_depth: u32,
    blocking: bool,
    memory_backed: bool,
    discard: bool,
    cache_size_mib: u64,
    mbps: u32,
    no_sched: bool,
    shared_tags: bool,
    shared_tag_bitmap: bool,
    fua: bool,
    virt_boundary: bool,
    power: bool,
}

impl DeviceConfig {
    fn defaults() -> Self {
        Self {
            size_mib: DEFAULT_SIZE_MIB,
            completion_nsec: DEFAULT_COMPLETION_NSEC,
            submit_queues: DEFAULT_SUBMIT_QUEUES,
            poll_queues: DEFAULT_POLL_QUEUES,
            home_node: DEFAULT_HOME_NODE,
            queue_mode: QUEUE_MODE_MQ,
            blocksize: DEFAULT_BLOCK_SIZE,
            max_sectors: 0,
            irqmode: IRQ_MODE_SOFTIRQ,
            hw_queue_depth: DEFAULT_HW_QUEUE_DEPTH,
            blocking: false,
            memory_backed: false,
            discard: false,
            cache_size_mib: 0,
            mbps: 0,
            no_sched: false,
            shared_tags: false,
            shared_tag_bitmap: false,
            fua: true,
            virt_boundary: false,
            power: false,
        }
    }

    fn capacity_sectors(self) -> u64 {
        (self.size_mib.saturating_mul(MIB)) >> SECTOR_SHIFT
    }

    fn total_hw_queues(self) -> u32 {
        self.submit_queues.saturating_add(self.poll_queues)
    }

    fn validate(&mut self) -> Result {
        match self.queue_mode {
            QUEUE_MODE_RQ => return Err(EINVAL),
            QUEUE_MODE_BIO => self.queue_mode = QUEUE_MODE_MQ,
            QUEUE_MODE_MQ => {}
            _ => return Err(EINVAL),
        }

        if self.hw_queue_depth == 0 {
            return Err(EINVAL);
        }

        let builder = GenDiskBuilder::new().logical_block_size(self.blocksize)?;
        let _ = builder.physical_block_size(self.blocksize)?;

        if self.submit_queues == 0 {
            self.submit_queues = 1;
        }
        if self.submit_queues > MAX_TRACKED_HW_QUEUES {
            self.submit_queues = MAX_TRACKED_HW_QUEUES;
        }
        if self.poll_queues > MAX_TRACKED_HW_QUEUES {
            self.poll_queues = MAX_TRACKED_HW_QUEUES;
        }
        if self.total_hw_queues() == 0 || self.total_hw_queues() > MAX_TRACKED_HW_QUEUES {
            return Err(EINVAL);
        }

        self.irqmode = self.irqmode.min(IRQ_MODE_TIMER);

        if self.memory_backed {
            self.blocking = true;
        } else {
            self.cache_size_mib = 0;
            self.discard = false;
        }

        self.mbps = self.mbps.min(MAX_MBPS);

        Ok(())
    }
}

#[derive(Clone, Copy, Eq, PartialEq)]
struct SharedTagConfig {
    submit_queues: u32,
    poll_queues: u32,
    hw_queue_depth: u32,
    home_node: i32,
    no_sched: bool,
    shared_tag_bitmap: bool,
    blocking: bool,
}

impl From<DeviceConfig> for SharedTagConfig {
    fn from(config: DeviceConfig) -> Self {
        Self {
            submit_queues: config.submit_queues,
            poll_queues: config.poll_queues,
            hw_queue_depth: config.hw_queue_depth,
            home_node: config.home_node,
            no_sched: config.no_sched,
            shared_tag_bitmap: config.shared_tag_bitmap,
            blocking: config.blocking,
        }
    }
}

struct SharedTagPool {
    config: SharedTagConfig,
    tagset: Arc<TagSet<RnullQueue>>,
    users: usize,
}

struct Controller {
    lifecycle: Arc<Mutex<()>>,
    devices: Arc<Mutex<Vec<Arc<DeviceInner>>>>,
    shared_tagset: Arc<Mutex<Option<SharedTagPool>>>,
    next_default_index: AtomicU32,
}

impl Controller {
    fn new() -> Result<Arc<Self>> {
        Ok(Arc::new(
            Self {
                lifecycle: Arc::pin_init(new_mutex!(()), GFP_KERNEL)?,
                devices: Arc::pin_init(new_mutex!(Vec::new()), GFP_KERNEL)?,
                shared_tagset: Arc::pin_init(new_mutex!(None::<SharedTagPool>), GFP_KERNEL)?,
                next_default_index: AtomicU32::new(0),
            },
            GFP_KERNEL,
        )?)
    }

    fn create_default_devices(self: &Arc<Self>, count: u32) -> Result {
        let mut created = Vec::with_capacity(count as usize, GFP_KERNEL)?;

        for _ in 0..count {
            let index = self.next_default_index.fetch_add(1, Ordering::Relaxed);
            let name = CString::try_from_fmt(format_args!("nullb{}", index))?;
            let device = self.new_device(name)?;
            device.power_on()?;
            created.push(device, GFP_KERNEL)?;
        }

        Ok(())
    }

    fn create_configfs_device(self: &Arc<Self>, name: CString) -> Result<Arc<DeviceInner>> {
        self.new_device(name)
    }

    fn new_device(self: &Arc<Self>, name: CString) -> Result<Arc<DeviceInner>> {
        let _guard = self.lifecycle.lock();
        let device = DeviceInner::new(self.clone(), name)?;
        self.devices.lock().push(device.clone(), GFP_KERNEL)?;
        Ok(device)
    }

    fn remove_device_locked(&self, device: &Arc<DeviceInner>) {
        let mut devices = self.devices.lock();
        if let Some(index) = devices.iter().position(|entry| Arc::ptr_eq(entry, device)) {
            devices.swap_remove(index);
        }
    }

    fn drop_device(&self, device: &Arc<DeviceInner>) {
        let _guard = self.lifecycle.lock();
        device.power_off_locked();
        self.remove_device_locked(device);
    }

    fn shutdown_all(&self) {
        let _guard = self.lifecycle.lock();
        let devices = self.devices.lock();
        for device in devices.iter() {
            device.power_off_locked();
        }
    }

    fn acquire_tagset(&self, config: DeviceConfig) -> Result<(Arc<TagSet<RnullQueue>>, bool)> {
        if !config.shared_tags {
            return Ok((Self::new_tagset(config)?, false));
        }

        let mut shared = self.shared_tagset.lock();
        let wanted = SharedTagConfig::from(config);

        if let Some(pool) = shared.as_mut() {
            if pool.config != wanted {
                return Err(EINVAL);
            }
            pool.users += 1;
            return Ok((pool.tagset.clone(), true));
        }

        let tagset = Self::new_tagset(config)?;
        *shared = Some(SharedTagPool {
            config: wanted,
            tagset: tagset.clone(),
            users: 1,
        });

        Ok((tagset, true))
    }

    fn release_tagset(&self, shared_tagset: bool, tagset: &Arc<TagSet<RnullQueue>>) {
        if !shared_tagset {
            return;
        }

        let mut shared = self.shared_tagset.lock();
        let Some(pool) = shared.as_mut() else {
            return;
        };

        if !Arc::ptr_eq(&pool.tagset, tagset) {
            return;
        }

        if pool.users > 1 {
            pool.users -= 1;
        } else {
            *shared = None;
        }
    }

    fn update_shared_queue_counts(
        &self,
        tagset: &Arc<TagSet<RnullQueue>>,
        submit_queues: u32,
        poll_queues: u32,
    ) {
        let mut shared = self.shared_tagset.lock();
        let Some(pool) = shared.as_mut() else {
            return;
        };

        if !Arc::ptr_eq(&pool.tagset, tagset) {
            return;
        }

        pool.config.submit_queues = submit_queues;
        pool.config.poll_queues = poll_queues;
    }

    fn new_tagset(config: DeviceConfig) -> Result<Arc<TagSet<RnullQueue>>> {
        let tagset = TagSetConfig::new(
            config.submit_queues,
            config.hw_queue_depth,
            if config.poll_queues > 0 { 3 } else { 1 },
        )
        .poll_queues(config.poll_queues)
        .timeout(time::msecs_to_jiffies(TIMEOUT_MSECS) as u32)
        .numa_node(config.home_node)
        .no_sched(config.no_sched)
        .shared_tag_bitmap(config.shared_tag_bitmap)
        .blocking(config.blocking);

        Arc::pin_init(TagSet::with_config(tagset), GFP_KERNEL)
    }
}

struct DeviceBacking {
    storage: Arc<Mutex<StorageState>>,
    badblocks: Arc<BadBlocks>,
}

impl DeviceBacking {
    fn new() -> Result<Arc<Self>> {
        Ok(Arc::new(
            Self {
                storage: Arc::pin_init(new_mutex!(StorageState::new()), GFP_KERNEL)?,
                badblocks: Arc::new(BadBlocks::new()?, GFP_KERNEL)?,
            },
            GFP_KERNEL,
        )?)
    }
}

struct DeviceRuntime {
    _state: Arc<RuntimeState>,
    tagset: Arc<TagSet<RnullQueue>>,
    _disk: GenDisk<RnullQueue>,
    shared_tagset: bool,
}

impl DeviceRuntime {
    fn new(device: &DeviceInner, config: DeviceConfig) -> Result<Self> {
        let (tagset, shared_tagset) = device.controller.acquire_tagset(config)?;
        let state = Arc::pin_init(RuntimeState::new(config, device.backing.clone()), GFP_KERNEL)?;

        let mut builder = GenDiskBuilder::new();
        builder = builder.logical_block_size(config.blocksize)?;
        builder = builder.physical_block_size(config.blocksize)?;
        builder = builder.capacity_sectors(config.capacity_sectors());
        builder = builder.max_hw_sectors(config.max_sectors);

        if config.discard {
            builder = builder.max_hw_discard_sectors(u32::MAX >> SECTOR_SHIFT);
        }
        if config.virt_boundary {
            builder = builder.virt_boundary_mask((PAGE_BYTES - 1) as u32);
        }

        let name = device.name.to_str()?;
        let disk = match builder.build(format_args!("{}", name), tagset.clone(), state.clone()) {
            Ok(disk) => disk,
            Err(err) => {
                device.controller.release_tagset(shared_tagset, &tagset);
                return Err(err);
            }
        };

        if config.cache_size_mib > 0 {
            disk.set_write_cache(true, config.fua);
        }

        Ok(Self {
            _state: state,
            tagset,
            _disk: disk,
            shared_tagset,
        })
    }
}

struct DeviceInner {
    controller: Arc<Controller>,
    name: CString,
    backing: Arc<DeviceBacking>,
    config: Arc<Mutex<DeviceConfig>>,
    runtime: Arc<Mutex<Option<DeviceRuntime>>>,
}

impl DeviceInner {
    fn new(controller: Arc<Controller>, name: CString) -> Result<Arc<Self>> {
        Ok(Arc::new(
            Self {
                controller,
                name,
                backing: DeviceBacking::new()?,
                config: Arc::pin_init(new_mutex!(DeviceConfig::defaults()), GFP_KERNEL)?,
                runtime: Arc::pin_init(new_mutex!(None::<DeviceRuntime>), GFP_KERNEL)?,
            },
            GFP_KERNEL,
        )?)
    }

    fn config(&self) -> DeviceConfig {
        *self.config.lock()
    }

    fn power_on(&self) -> Result {
        let _guard = self.controller.lifecycle.lock();
        self.power_on_locked()
    }

    fn power_on_locked(&self) -> Result {
        let mut config = self.config.lock();
        if config.power {
            return Ok(());
        }

        let mut validated = *config;
        validated.validate()?;

        let runtime = DeviceRuntime::new(self, validated)?;
        let mut runtime_slot = self.runtime.lock();
        if runtime_slot.is_some() {
            self.controller
                .release_tagset(runtime.shared_tagset, &runtime.tagset);
            return Err(EBUSY);
        }

        validated.power = true;
        *config = validated;
        *runtime_slot = Some(runtime);
        Ok(())
    }

    fn power_off(&self) {
        let _guard = self.controller.lifecycle.lock();
        self.power_off_locked();
    }

    fn power_off_locked(&self) {
        let runtime = {
            let mut runtime = self.runtime.lock();
            runtime.take()
        };

        if let Some(runtime) = runtime {
            self.controller
                .release_tagset(runtime.shared_tagset, &runtime.tagset);
            drop(runtime);
        }

        self.config.lock().power = false;
    }

    fn set_when_powered_off(&self, update: impl FnOnce(&mut DeviceConfig)) -> Result {
        let _guard = self.controller.lifecycle.lock();
        self.set_when_powered_off_locked(update)
    }

    fn set_when_powered_off_locked(&self, update: impl FnOnce(&mut DeviceConfig)) -> Result {
        let mut config = self.config.lock();
        if config.power {
            return Err(EBUSY);
        }
        update(&mut config);
        Ok(())
    }

    fn update_submit_queues(&self, submit_queues: u32) -> Result {
        let _guard = self.controller.lifecycle.lock();
        self.update_submit_queues_locked(submit_queues)
    }

    fn update_submit_queues_locked(&self, submit_queues: u32) -> Result {
        if submit_queues == 0 || submit_queues > MAX_TRACKED_HW_QUEUES {
            return Err(EINVAL);
        }

        let mut config = self.config.lock();
        if config.power {
            let runtime = self.runtime.lock();
            let Some(runtime) = runtime.as_ref() else {
                return Err(EINVAL);
            };

            if submit_queues.saturating_add(config.poll_queues) > MAX_TRACKED_HW_QUEUES {
                return Err(EINVAL);
            }

            runtime
                .tagset
                .update_queue_counts(submit_queues, config.poll_queues)?;
            if config.shared_tags {
                self.controller.update_shared_queue_counts(
                    &runtime.tagset,
                    submit_queues,
                    config.poll_queues,
                );
            }
        }

        config.submit_queues = submit_queues;
        Ok(())
    }

    fn update_poll_queues(&self, poll_queues: u32) -> Result {
        let _guard = self.controller.lifecycle.lock();
        self.update_poll_queues_locked(poll_queues)
    }

    fn update_poll_queues_locked(&self, poll_queues: u32) -> Result {
        if poll_queues > MAX_TRACKED_HW_QUEUES {
            return Err(EINVAL);
        }

        let mut config = self.config.lock();
        if config.submit_queues.saturating_add(poll_queues) > MAX_TRACKED_HW_QUEUES {
            return Err(EINVAL);
        }

        if config.power {
            let runtime = self.runtime.lock();
            let Some(runtime) = runtime.as_ref() else {
                return Err(EINVAL);
            };
            runtime
                .tagset
                .update_queue_counts(config.submit_queues, poll_queues)?;
            if config.shared_tags {
                self.controller.update_shared_queue_counts(
                    &runtime.tagset,
                    config.submit_queues,
                    poll_queues,
                );
            }
        }

        config.poll_queues = poll_queues;
        Ok(())
    }

    fn set_power(&self, enabled: bool) -> Result {
        if enabled {
            self.power_on()
        } else {
            self.power_off();
            Ok(())
        }
    }

    fn update_badblocks(&self, op: BadBlocksUpdate) -> Result {
        match op {
            BadBlocksUpdate::Set { start, len } => self.backing.badblocks.set(start, len),
            BadBlocksUpdate::Clear { start, len } => self.backing.badblocks.clear(start, len),
        }
    }
}

struct PageSlot {
    index: u64,
    data: Box<[u8; PAGE_BYTES]>,
}

struct StorageState {
    data: Vec<PageSlot>,
    cache: Vec<PageSlot>,
    cache_bytes: u64,
}

impl StorageState {
    fn new() -> Self {
        Self {
            data: Vec::new(),
            cache: Vec::new(),
            cache_bytes: 0,
        }
    }

    fn read_bytes(&self, offset: u64, dst: &mut [u8]) {
        let mut copied = 0usize;

        while copied < dst.len() {
            let absolute = offset + copied as u64;
            let page_index = absolute / PAGE_BYTES as u64;
            let page_offset = (absolute % PAGE_BYTES as u64) as usize;
            let len = cmp::min(PAGE_BYTES - page_offset, dst.len() - copied);

            dst[copied..copied + len].fill(0);

            if let Some(page) = Self::find_page(&self.cache, page_index) {
                dst[copied..copied + len].copy_from_slice(&page[page_offset..page_offset + len]);
            } else if let Some(page) = Self::find_page(&self.data, page_index) {
                dst[copied..copied + len].copy_from_slice(&page[page_offset..page_offset + len]);
            }

            copied += len;
        }
    }

    fn write_bytes(&mut self, offset: u64, src: &[u8], use_cache: bool, cache_limit: u64) -> Result {
        let mut written = 0usize;

        while written < src.len() {
            let absolute = offset + written as u64;
            let page_index = absolute / PAGE_BYTES as u64;
            let page_offset = (absolute % PAGE_BYTES as u64) as usize;
            let len = cmp::min(PAGE_BYTES - page_offset, src.len() - written);

            let (slot, inserted) = if use_cache {
                Self::get_or_create_page(&mut self.cache, page_index)?
            } else {
                Self::get_or_create_page(&mut self.data, page_index)?
            };
            slot.data[page_offset..page_offset + len]
                .copy_from_slice(&src[written..written + len]);
            if use_cache && inserted {
                self.cache_bytes = self.cache_bytes.saturating_add(PAGE_BYTES as u64);
            }
            written += len;
        }

        if use_cache && cache_limit > 0 && self.cache_bytes > cache_limit {
            self.flush_cache()?;
        }

        Ok(())
    }

    fn zero_range(&mut self, offset: u64, len: u64) -> Result {
        let mut remaining = len;
        let mut cursor = offset;

        while remaining > 0 {
            let page_index = cursor / PAGE_BYTES as u64;
            let page_offset = (cursor % PAGE_BYTES as u64) as usize;
            let chunk = cmp::min((PAGE_BYTES - page_offset) as u64, remaining) as usize;

            if let Some(slot) = Self::find_page_mut(&mut self.cache, page_index) {
                slot.data[page_offset..page_offset + chunk].fill(0);
            }
            if let Some(slot) = Self::find_page_mut(&mut self.data, page_index) {
                slot.data[page_offset..page_offset + chunk].fill(0);
            }

            cursor += chunk as u64;
            remaining -= chunk as u64;
        }

        Ok(())
    }

    fn flush_cache(&mut self) -> Result {
        while let Some(cache_page) = self.cache.pop() {
            let (data_page, _) = Self::get_or_create_page(&mut self.data, cache_page.index)?;
            data_page.data.copy_from_slice(&*cache_page.data);
        }

        self.cache_bytes = 0;
        Ok(())
    }

    fn find_page<'a>(slots: &'a Vec<PageSlot>, index: u64) -> Option<&'a [u8; PAGE_BYTES]> {
        slots
            .iter()
            .find(|slot| slot.index == index)
            .map(|slot| &*slot.data)
    }

    fn find_page_mut<'a>(slots: &'a mut Vec<PageSlot>, index: u64) -> Option<&'a mut PageSlot> {
        slots.iter_mut().find(|slot| slot.index == index)
    }

    fn get_or_create_page<'a>(
        slots: &'a mut Vec<PageSlot>,
        index: u64,
    ) -> Result<(&'a mut PageSlot, bool)> {
        if let Some(position) = slots.iter().position(|slot| slot.index == index) {
            return Ok((&mut slots[position], false));
        }

        let page = Box::new([0; PAGE_BYTES], GFP_KERNEL)?;
        slots.push(PageSlot { index, data: page }, GFP_KERNEL)?;
        Ok((
            slots.last_mut().expect("newly inserted page must be present"),
            true,
        ))
    }
}

struct PendingQueue<T> {
    entries: Vec<T>,
}

impl<T> PendingQueue<T> {
    fn with_capacity(capacity: usize) -> Result<Self> {
        Ok(Self {
            entries: Vec::with_capacity(capacity.max(1), GFP_KERNEL)?,
        })
    }

    fn push(&mut self, entry: T) -> Result {
        if self.entries.len() == self.entries.capacity() {
            return Err(ENOMEM);
        }
        self.entries.push(entry, GFP_ATOMIC)?;
        Ok(())
    }

    fn take_matching(&mut self, mut pred: impl FnMut(&T) -> bool) -> Option<T> {
        let index = self.entries.iter().position(|entry| pred(entry))?;
        Some(self.entries.swap_remove(index))
    }
}

struct CompletionEntry {
    id: usize,
    status: BlkStatus,
}

struct DelayedEntry {
    id: usize,
    rq: ARef<Request<RnullQueue>>,
    status: BlkStatus,
    ready_ns: i64,
}

struct PollEntry {
    id: usize,
    rq: ARef<Request<RnullQueue>>,
    status: BlkStatus,
    ready_ns: i64,
}

struct ThrottleState {
    next_ready_ns: i64,
}

impl ThrottleState {
    fn reserve_until(&mut self, now_ns: i64, bytes: u32, mbps: u32) -> i64 {
        if mbps == 0 || bytes == 0 {
            return now_ns;
        }

        let service_ns =
            ((bytes as u128) * (time::NSEC_PER_SEC as u128)) / ((mbps as u128) * (MIB as u128));
        let start_ns = self.next_ready_ns.max(now_ns);
        let finish_ns = start_ns.saturating_add(service_ns as i64);

        self.next_ready_ns = finish_ns;
        finish_ns
    }
}

#[pin_data]
struct RuntimeState {
    config: DeviceConfig,
    backing: Arc<DeviceBacking>,
    completions: Arc<SpinLock<PendingQueue<CompletionEntry>>>,
    delayed: Arc<SpinLock<PendingQueue<DelayedEntry>>>,
    poll_queues: Vec<Arc<SpinLock<PendingQueue<PollEntry>>>>,
    throttle: Arc<SpinLock<ThrottleState>>,
    #[pin]
    timer: HrTimer<Self>,
}

impl RuntimeState {
    fn new(config: DeviceConfig, backing: Arc<DeviceBacking>) -> impl PinInit<Self, Error> {
        let total_entries = (config.total_hw_queues() as usize)
            .saturating_mul(config.hw_queue_depth as usize)
            .max(1);
        let queue_capacity = config.hw_queue_depth as usize;

        try_pin_init!(Self {
            config,
            backing,
            completions: Arc::pin_init(
                new_spinlock!(PendingQueue::<CompletionEntry>::with_capacity(total_entries)?),
                GFP_KERNEL,
            )?,
            delayed: Arc::pin_init(
                new_spinlock!(PendingQueue::<DelayedEntry>::with_capacity(total_entries)?),
                GFP_KERNEL,
            )?,
            poll_queues: Self::build_poll_queues(queue_capacity)?,
            throttle: Arc::pin_init(
                new_spinlock!(ThrottleState { next_ready_ns: 0 }),
                GFP_KERNEL,
            )?,
            timer <- HrTimer::new(),
        })
    }

    fn build_poll_queues(capacity: usize) -> Result<Vec<Arc<SpinLock<PendingQueue<PollEntry>>>>> {
        let mut queues = Vec::with_capacity(MAX_TRACKED_HW_QUEUES as usize, GFP_KERNEL)?;

        for _ in 0..MAX_TRACKED_HW_QUEUES {
            queues.push(
                Arc::pin_init(
                    new_spinlock!(PendingQueue::<PollEntry>::with_capacity(capacity)?),
                    GFP_KERNEL,
                )?,
                GFP_KERNEL,
            )?;
        }

        Ok(queues)
    }

    fn queue_request(self: ArcBorrow<'_, Self>, rq: ARef<Request<RnullQueue>>) -> QueueResult {
        if rq.should_fake_timeout() {
            return Ok(());
        }

        let status = self.process_request(&rq);
        let ready_ns = self.ready_ns(rq.bytes());

        if rq.queue_kind() == QueueKind::Poll {
            return self.enqueue_poll(rq, status, ready_ns);
        }

        match (IrqMode::from_raw(self.config.irqmode), ready_ns > Ktime::ktime_get().to_ns()) {
            (_, true) | (IrqMode::Timer, _) => {
                let owner: Arc<Self> = self.into();
                owner.enqueue_delayed(rq, status, ready_ns)
            }
            (IrqMode::Softirq, false) => self.defer_completion(rq, status),
            (IrqMode::None, false) => {
                self.finish_request(rq, status);
                Ok(())
            }
        }
    }

    fn complete_request(&self, rq: ARef<Request<RnullQueue>>) {
        let status = {
            let mut completions = self.completions.lock();
            completions
                .take_matching(|entry| entry.id == rq.opaque_id())
                .map(|entry| entry.status)
                .unwrap_or(BlkStatus::IOERR)
        };

        self.finish_request(rq, status);
    }

    fn timeout_request(&self, rq: ARef<Request<RnullQueue>>) -> TimeoutResult {
        let id = rq.opaque_id();
        self.take_delayed(id);
        self.take_poll(id);
        {
            let mut completions = self.completions.lock();
            let _ = completions.take_matching(|entry| entry.id == id);
        }

        self.finish_request(rq, BlkStatus::TIMEOUT);
        TimeoutResult::Done
    }

    fn poll_ready(&self, hctx_index: u32) -> u32 {
        let Some(queue) = self.poll_queues.get(hctx_index as usize) else {
            return 0;
        };

        let mut completed = 0;
        let now = Ktime::ktime_get().to_ns();

        loop {
            let entry = {
                let mut queue = queue.lock();
                queue.take_matching(|entry| entry.ready_ns <= now)
            };

            let Some(entry) = entry else {
                break;
            };

            self.finish_request(entry.rq, entry.status);
            completed += 1;
        }

        completed
    }

    fn ready_ns(&self, bytes: u32) -> i64 {
        let now = Ktime::ktime_get().to_ns();
        let mut ready = now;

        if self.config.mbps > 0 {
            ready = ready.max(self.throttle.lock().reserve_until(now, bytes, self.config.mbps));
        }
        if IrqMode::from_raw(self.config.irqmode) == IrqMode::Timer {
            ready = ready.max(now.saturating_add(self.config.completion_nsec as i64));
        }

        ready
    }

    fn process_request(&self, rq: &Request<RnullQueue>) -> BlkStatus {
        match rq.op() {
            RequestOp::Flush => {
                if self.config.cache_size_mib == 0 {
                    return BlkStatus::OK;
                }

                match self.backing.storage.lock().flush_cache() {
                    Ok(()) => BlkStatus::OK,
                    Err(err) => status_from_error(err),
                }
            }
            RequestOp::Discard | RequestOp::Read | RequestOp::Write | RequestOp::WriteZeroes => {
                let sectors = rq.sectors();
                if sectors > 0 && self.backing.badblocks.check(rq.sector(), sectors) {
                    return BlkStatus::IOERR;
                }

                if self.config.memory_backed {
                    self.process_memory_backed(rq)
                } else {
                    self.process_unbacked(rq)
                }
            }
            RequestOp::Unknown(_) => BlkStatus::NOTSUPP,
        }
    }

    fn process_unbacked(&self, rq: &Request<RnullQueue>) -> BlkStatus {
        match rq.op() {
            RequestOp::Read => zero_segments(rq),
            RequestOp::Write | RequestOp::Discard | RequestOp::WriteZeroes | RequestOp::Flush => {
                BlkStatus::OK
            }
            RequestOp::Unknown(_) => BlkStatus::NOTSUPP,
        }
    }

    fn process_memory_backed(&self, rq: &Request<RnullQueue>) -> BlkStatus {
        let mut storage = self.backing.storage.lock();
        let cache_limit = self.config.cache_size_mib.saturating_mul(MIB);
        let use_cache = self.config.cache_size_mib > 0 && !rq.is_fua();

        match rq.op() {
            RequestOp::Read => {
                let mut sector = rq.sector();
                let result = rq.for_each_segment(|segment| {
                    let mut bounce = [0u8; PAGE_BYTES];
                    let len = segment.len();
                    storage.read_bytes(sector << SECTOR_SHIFT, &mut bounce[..len]);
                    let _ = segment.copy_from_slice(&bounce[..len]);
                    sector = sector.saturating_add((len >> SECTOR_SHIFT) as u64);
                    Ok(())
                });

                match result {
                    Ok(()) => BlkStatus::OK,
                    Err(err) => status_from_error(err),
                }
            }
            RequestOp::Write => {
                let mut sector = rq.sector();
                let result = rq.for_each_segment(|segment| {
                    let len = segment.len();
                    let mut bounce = [0u8; PAGE_BYTES];
                    let _ = segment.copy_to_slice(&mut bounce[..len]);
                    storage.write_bytes(sector << SECTOR_SHIFT, &bounce[..len], use_cache, cache_limit)?;
                    sector = sector.saturating_add((len >> SECTOR_SHIFT) as u64);
                    Ok(())
                });

                match result {
                    Ok(()) => BlkStatus::OK,
                    Err(err) => status_from_error(err),
                }
            }
            RequestOp::Discard => {
                if !self.config.discard {
                    return BlkStatus::NOTSUPP;
                }

                match storage.zero_range(rq.sector() << SECTOR_SHIFT, rq.bytes() as u64) {
                    Ok(()) => BlkStatus::OK,
                    Err(err) => status_from_error(err),
                }
            }
            RequestOp::WriteZeroes => {
                match storage.zero_range(rq.sector() << SECTOR_SHIFT, rq.bytes() as u64) {
                    Ok(()) => BlkStatus::OK,
                    Err(err) => status_from_error(err),
                }
            }
            RequestOp::Flush => match storage.flush_cache() {
                Ok(()) => BlkStatus::OK,
                Err(err) => status_from_error(err),
            },
            RequestOp::Unknown(_) => BlkStatus::NOTSUPP,
        }
    }

    fn defer_completion(&self, rq: ARef<Request<RnullQueue>>, status: BlkStatus) -> QueueResult {
        {
            let mut completions = self.completions.lock();
            completions
                .push(CompletionEntry {
                    id: rq.opaque_id(),
                    status,
                })
                .map_err(status_from_error)?;
        }

        Request::complete(rq);
        Ok(())
    }

    fn enqueue_delayed(
        self: Arc<Self>,
        rq: ARef<Request<RnullQueue>>,
        status: BlkStatus,
        ready_ns: i64,
    ) -> QueueResult {
        {
            let mut delayed = self.delayed.lock();
            delayed
                .push(DelayedEntry {
                    id: rq.opaque_id(),
                    rq,
                    status,
                    ready_ns,
                })
                .map_err(status_from_error)?;
        }

        <Self as HrTimerCallback>::start(
            &self,
            Ktime::from_ns((ready_ns - Ktime::ktime_get().to_ns()).max(0)),
        );
        Ok(())
    }

    fn enqueue_poll(&self, rq: ARef<Request<RnullQueue>>, status: BlkStatus, ready_ns: i64) -> QueueResult {
        let index = rq.hctx_index() as usize;
        let Some(queue) = self.poll_queues.get(index) else {
            return Err(BlkStatus::RESOURCE);
        };

        queue
            .lock()
            .push(PollEntry {
                id: rq.opaque_id(),
                rq,
                status,
                ready_ns,
            })
            .map_err(status_from_error)?;

        Ok(())
    }

    fn finish_request(&self, rq: ARef<Request<RnullQueue>>, status: BlkStatus) {
        if let Err(rq) = Request::end(rq, status) {
            let _ = self.defer_completion(rq, status);
        }
    }

    fn take_delayed(&self, id: usize) -> Option<DelayedEntry> {
        self.delayed.lock().take_matching(|entry| entry.id == id)
    }

    fn take_poll(&self, id: usize) -> Option<PollEntry> {
        for queue in &self.poll_queues {
            if let Some(entry) = queue.lock().take_matching(|entry| entry.id == id) {
                return Some(entry);
            }
        }

        None
    }

    fn next_timer_deadline(&self) -> Option<i64> {
        self.delayed
            .lock()
            .entries
            .iter()
            .map(|entry| entry.ready_ns)
            .min()
    }
}

impl HrTimerCallback for RuntimeState {
    fn run(this: ArcBorrow<'_, Self>, ctx: &mut HrTimerCallbackContext<'_, Self>) -> HrTimerRestart {
        loop {
            let now = Ktime::ktime_get().to_ns();
            let entry = this
                .delayed
                .lock()
                .take_matching(|entry| entry.ready_ns <= now);

            let Some(entry) = entry else {
                break;
            };

            this.finish_request(entry.rq, entry.status);
        }

        let Some(next_deadline) = this.next_timer_deadline() else {
            return HrTimerRestart::NoRestart;
        };

        let now = Ktime::ktime_get().to_ns();
        ctx.forward_now(Ktime::from_ns((next_deadline - now).max(0)));
        HrTimerRestart::Restart
    }
}

kernel::impl_has_hr_timer! {
    impl HrTimerCallback for RuntimeState {
        field: self.timer,
    }
}

struct RnullQueue;

#[vtable]
impl mq::Operations for RnullQueue {
    type QueueData = Arc<RuntimeState>;

    fn queue_rq(
        queue_data: ArcBorrow<'_, RuntimeState>,
        rq: ARef<Request<Self>>,
        _is_last: bool,
    ) -> QueueResult {
        queue_data.queue_request(rq)
    }

    fn commit_rqs(_queue_data: ArcBorrow<'_, RuntimeState>) {}

    fn complete(rq: ARef<Request<Self>>) {
        let runtime: Arc<RuntimeState> = rq.queue_data().into();
        runtime.complete_request(rq);
    }

    fn poll(queue_data: ArcBorrow<'_, RuntimeState>, hctx_index: u32) -> u32 {
        queue_data.poll_ready(hctx_index)
    }

    fn timeout(rq: ARef<Request<Self>>) -> TimeoutResult {
        let runtime: Arc<RuntimeState> = rq.queue_data().into();
        runtime.timeout_request(rq)
    }
}

enum BadBlocksUpdate {
    Set { start: u64, len: u32 },
    Clear { start: u64, len: u32 },
}

fn zero_segments(rq: &Request<RnullQueue>) -> BlkStatus {
    match rq.for_each_segment(|segment| {
        segment.zero();
        Ok(())
    }) {
        Ok(()) => BlkStatus::OK,
        Err(err) => status_from_error(err),
    }
}

fn status_from_error(err: Error) -> BlkStatus {
    if err.to_errno() == ENOMEM.to_errno() {
        BlkStatus::RESOURCE
    } else {
        BlkStatus::IOERR
    }
}

fn write_page(page: &mut [u8; bindings::PAGE_SIZE], args: fmt::Arguments<'_>) -> Result<usize> {
    let rendered = CString::try_from_fmt(args)?;
    let bytes = rendered.as_bytes();
    if bytes.len() > page.len() {
        return Err(ENOSPC);
    }

    page[..bytes.len()].copy_from_slice(bytes);
    Ok(bytes.len())
}

fn show_bool(page: &mut [u8; bindings::PAGE_SIZE], value: bool) -> Result<usize> {
    write_page(page, format_args!("{}\n", value as u8))
}

fn show_u32(page: &mut [u8; bindings::PAGE_SIZE], value: u32) -> Result<usize> {
    write_page(page, format_args!("{}\n", value))
}

fn show_u64(page: &mut [u8; bindings::PAGE_SIZE], value: u64) -> Result<usize> {
    write_page(page, format_args!("{}\n", value))
}

fn parse_str(page: &[u8]) -> Result<&str> {
    Ok(core::str::from_utf8(page).map_err(|_| EINVAL)?.trim())
}

fn parse_bool(page: &[u8]) -> Result<bool> {
    match parse_str(page)? {
        "1" | "y" | "Y" | "yes" | "YES" | "true" | "TRUE" | "on" | "ON" => Ok(true),
        "0" | "n" | "N" | "no" | "NO" | "false" | "FALSE" | "off" | "OFF" => Ok(false),
        _ => Err(EINVAL),
    }
}

fn parse_u32(page: &[u8]) -> Result<u32> {
    parse_str(page)?.parse::<u32>().map_err(|_| EINVAL)
}

fn parse_u64(page: &[u8]) -> Result<u64> {
    parse_str(page)?.parse::<u64>().map_err(|_| EINVAL)
}

fn parse_badblocks(page: &[u8]) -> Result<BadBlocksUpdate> {
    let text = parse_str(page)?;
    let (kind, range) = text.split_at(1);
    let (start, end) = range.split_once('-').ok_or(EINVAL)?;
    let start = start.parse::<u64>().map_err(|_| EINVAL)?;
    let end = end.parse::<u64>().map_err(|_| EINVAL)?;

    if start > end {
        return Err(EINVAL);
    }

    let len = end
        .checked_sub(start)
        .and_then(|span| span.checked_add(1))
        .ok_or(EINVAL)? as u32;

    match kind {
        "+" => Ok(BadBlocksUpdate::Set { start, len }),
        "-" => Ok(BadBlocksUpdate::Clear { start, len }),
        _ => Err(EINVAL),
    }
}
