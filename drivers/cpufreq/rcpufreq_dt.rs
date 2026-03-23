// SPDX-License-Identifier: GPL-2.0

//! Generic DT based cpufreq driver.

use kernel::{
    alloc::{KBox, KVec},
    clk::Clk,
    cpu::{self, CpuId},
    cpufreq::{self, PolicyData, RegistrationConfig, TableIndex},
    cpufreq_dt::{self, PlatformCallbacks},
    cpumask::CpumaskVar,
    device::{self, Core},
    error::{code::*, Error, Result},
    module_platform_driver,
    opp::{self, RegulatorToken},
    platform,
    prelude::*,
    sync::{Arc, ArcBorrow},
};

#[derive(Copy, Clone)]
enum SupplyName {
    Cpu0,
    Cpu,
}

impl SupplyName {
    fn as_cstr(self) -> &'static CStr {
        match self {
            Self::Cpu0 => c"cpu0",
            Self::Cpu => c"cpu",
        }
    }
}

struct ClusterDescriptor {
    primary_cpu: CpuId,
    cpus: KVec<CpuId>,
    needs_sharing_fallback: bool,
    supply_name: Option<SupplyName>,
}

impl ClusterDescriptor {
    fn contains(&self, cpu: CpuId) -> bool {
        self.cpus.iter().copied().any(|member| member == cpu)
    }

    fn cpumask(&self) -> Result<CpumaskVar> {
        let mut mask = CpumaskVar::new_zero(GFP_KERNEL)?;
        for cpu in self.cpus.iter().copied() {
            mask.set(cpu);
        }
        Ok(mask)
    }
}

struct SharedConfig {
    clusters: KVec<ClusterDescriptor>,
    callbacks: Option<PlatformCallbacks>,
}

impl SharedConfig {
    fn discover(pdev: &platform::Device<Core>) -> Result<Self> {
        let callbacks = pdev
            .platdata::<cpufreq_dt::PlatformData>()
            .map(|data| data.copy_callbacks());

        let mut clusters: KVec<ClusterDescriptor> = KVec::new();

        cpu::for_each_present(|cpu| {
            if clusters.iter().any(|cluster| cluster.contains(cpu)) {
                return Ok(());
            }

            clusters.push(discover_cluster(cpu)?, GFP_KERNEL)?;
            Ok(())
        })?;

        if clusters.is_empty() {
            return Err(ENODEV);
        }

        Ok(Self {
            clusters,
            callbacks,
        })
    }

    fn flags(&self) -> u16 {
        let mut flags = cpufreq::flags::NEED_INITIAL_FREQ_CHECK | cpufreq::flags::IS_COOLING_DEV;

        if self
            .callbacks
            .as_ref()
            .is_some_and(|callbacks| callbacks.have_governor_per_policy())
        {
            flags |= cpufreq::flags::HAVE_GOVERNOR_PER_POLICY;
        }

        flags
    }

    fn descriptor_for(&self, cpu: CpuId) -> Option<&ClusterDescriptor> {
        self.clusters.iter().find(|cluster| cluster.contains(cpu))
    }
}

struct PolicyState {
    clock: Clk,
    freq_table: opp::FreqTable,
    opp_table: opp::Table,
    #[allow(dead_code)]
    regulator: Option<RegulatorToken>,
    cpus: CpumaskVar,
    #[allow(dead_code)]
    primary_cpu: CpuId,
}

struct CpufreqDtDriver;

#[vtable]
impl cpufreq::Driver for CpufreqDtDriver {
    const NAME: &'static CStr = c"cpufreq-dt";
    const FLAGS: u16 = cpufreq::flags::NEED_INITIAL_FREQ_CHECK | cpufreq::flags::IS_COOLING_DEV;
    const BOOST_ENABLED: bool = false;

    type PData = KBox<PolicyState>;
    type DData = Arc<SharedConfig>;

    fn init(
        policy: &mut cpufreq::Policy,
        shared: ArcBorrow<'_, SharedConfig>,
    ) -> Result<Self::PData> {
        let cluster = shared.descriptor_for(policy.cpu()).ok_or(ENODEV)?;
        let mut cpus = cluster.cpumask()?;

        with_cpu_device(cluster.primary_cpu, |cpu_dev| {
            let regulator = match cluster.supply_name {
                Some(name) => Some(RegulatorToken::new(cpu_dev, &[name.as_cstr()])?),
                None => None,
            };

            let mut opp_table = opp::Table::from_cpu_dev_cpumask(cpu_dev, &mut cpus)?;
            if opp_table.opp_count()? == 0 {
                return Err(ENODEV);
            }

            if cluster.needs_sharing_fallback {
                opp_table.set_sharing_cpus(&mut cpus)?;
            }

            let freq_table = opp_table.cpufreq_table()?;
            let clock = Clk::get(cpu_dev, None)?;

            Ok(KBox::new(
                PolicyState {
                    cpus,
                    regulator,
                    opp_table,
                    freq_table,
                    clock,
                    primary_cpu: cluster.primary_cpu,
                },
                GFP_KERNEL,
            )?)
        })
    }

    fn init_post(policy: &mut cpufreq::Policy, _shared: ArcBorrow<'_, SharedConfig>) -> Result {
        policy.copy_cpus_from::<Self::PData, _>(|state| &state.cpus)?;
        policy.install_clk_from::<Self::PData, _>(|state| &state.clock)?;
        policy.install_freq_table_from::<Self::PData, _>(|state| &state.freq_table)?;

        let (suspend_freq, transition_latency_ns) = {
            let state = policy.data::<Self::PData>().ok_or(ENOENT)?;
            let latency = state.opp_table.max_transition_latency_ns();
            let latency = if latency == 0 {
                cpufreq::DEFAULT_TRANSITION_LATENCY_NS
            } else {
                latency.try_into().unwrap_or(u32::MAX)
            };

            (state.opp_table.suspend_freq(), latency)
        };

        policy
            .set_suspend_freq(suspend_freq)
            .set_transition_latency_ns(transition_latency_ns)
            .set_dvfs_possible_from_any_cpu(true);

        Ok(())
    }

    fn exit(
        _policy: &mut cpufreq::Policy,
        _data: Option<Self::PData>,
        _shared: ArcBorrow<'_, SharedConfig>,
    ) -> Result {
        Ok(())
    }

    fn online(_policy: &mut cpufreq::Policy, _shared: ArcBorrow<'_, SharedConfig>) -> Result {
        Ok(())
    }

    fn offline(_policy: &mut cpufreq::Policy, _shared: ArcBorrow<'_, SharedConfig>) -> Result {
        Ok(())
    }

    fn suspend(policy: &mut cpufreq::Policy, shared: ArcBorrow<'_, SharedConfig>) -> Result {
        if let Some(callbacks) = shared.callbacks.as_ref() {
            callbacks.suspend(policy)
        } else {
            policy.generic_suspend()
        }
    }

    fn resume(policy: &mut cpufreq::Policy, shared: ArcBorrow<'_, SharedConfig>) -> Result {
        shared
            .callbacks
            .as_ref()
            .map_or(Ok(()), |callbacks| callbacks.resume(policy))
    }

    fn verify(data: &mut PolicyData, _shared: ArcBorrow<'_, SharedConfig>) -> Result {
        data.generic_verify()
    }

    fn target_index(
        policy: &mut cpufreq::Policy,
        _shared: ArcBorrow<'_, SharedConfig>,
        index: TableIndex,
    ) -> Result {
        let state = policy.data::<Self::PData>().ok_or(ENODEV)?;
        let freq = state.freq_table.freq(index)?;

        state.opp_table.set_rate(freq)
    }

    fn get_intermediate(
        policy: &mut cpufreq::Policy,
        shared: ArcBorrow<'_, SharedConfig>,
        index: TableIndex,
    ) -> u32 {
        shared
            .callbacks
            .as_ref()
            .map_or(0, |callbacks| callbacks.get_intermediate_khz(policy, index))
    }

    fn target_intermediate(
        policy: &mut cpufreq::Policy,
        shared: ArcBorrow<'_, SharedConfig>,
        index: TableIndex,
    ) -> Result {
        shared.callbacks.as_ref().map_or(Ok(()), |callbacks| {
            callbacks.target_intermediate(policy, index)
        })
    }

    fn get(policy: &mut cpufreq::Policy, _shared: ArcBorrow<'_, SharedConfig>) -> Result<u32> {
        policy.generic_get()
    }

    fn register_em(policy: &mut cpufreq::Policy, _shared: ArcBorrow<'_, SharedConfig>) {
        policy.register_em_opp();
    }
}

struct PlatformState {
    #[allow(dead_code)]
    registration: cpufreq::Registration<CpufreqDtDriver>,
    #[allow(dead_code)]
    shared: Arc<SharedConfig>,
}

impl platform::Driver for PlatformState {
    type IdInfo = ();
    const OF_ID_TABLE: Option<kernel::of::IdTable<Self::IdInfo>> = None;

    fn probe(
        pdev: &platform::Device<Core>,
        _id_info: Option<&Self::IdInfo>,
    ) -> impl PinInit<Self, Error> {
        let shared = Arc::new(SharedConfig::discover(pdev)?, GFP_KERNEL)?;
        let registration = cpufreq::Registration::new_with_config(
            RegistrationConfig::<CpufreqDtDriver>::new(shared.clone()).flags(shared.flags()),
        )?;

        Ok(Self {
            registration,
            shared,
        })
    }
}

fn discover_cluster(cpu: CpuId) -> Result<ClusterDescriptor> {
    with_cpu_device(cpu, |cpu_dev| {
        let mut mask = CpumaskVar::new_zero(GFP_KERNEL)?;
        let mut needs_sharing_fallback = false;

        mask.set(cpu);

        match opp::Table::of_sharing_cpus(cpu_dev, &mut mask) {
            Ok(()) => {}
            Err(err) if err.to_errno() == ENOENT.to_errno() => {
                if opp::Table::sharing_cpus(cpu_dev, &mut mask).is_err() {
                    mask.setall();
                    needs_sharing_fallback = true;
                }
            }
            Err(err) => return Err(err),
        }

        let mut cpus = KVec::new();
        for candidate in 0..cpu::nr_cpu_ids() {
            let Some(candidate) = CpuId::from_u32(candidate) else {
                continue;
            };

            if mask.test(candidate) {
                cpus.push(candidate, GFP_KERNEL)?;
            }
        }

        Ok(ClusterDescriptor {
            primary_cpu: cpu,
            cpus,
            needs_sharing_fallback,
            supply_name: find_supply_name(cpu, cpu_dev),
        })
    })
}

fn with_cpu_device<R>(cpu: CpuId, f: impl FnOnce(&device::Device) -> Result<R>) -> Result<R> {
    match cpu::with_device(cpu, f) {
        Err(err) if err.to_errno() == ENODEV.to_errno() => Err(EPROBE_DEFER),
        other => other,
    }
}

fn find_supply_name(cpu: CpuId, dev: &device::Device) -> Option<SupplyName> {
    let fwnode = dev.fwnode()?;

    if cpu.as_u32() == 0 && fwnode.property_present(c"cpu0-supply") {
        return Some(SupplyName::Cpu0);
    }

    if fwnode.property_present(c"cpu-supply") {
        return Some(SupplyName::Cpu);
    }

    None
}

module_platform_driver! {
    type: PlatformState,
    name: "cpufreq-dt",
    authors: ["Viresh Kumar", "Shawn Guo"],
    description: "Generic cpufreq driver",
    license: "GPL",
    alias: ["platform:cpufreq-dt"],
}
