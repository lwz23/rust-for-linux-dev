// SPDX-License-Identifier: GPL-2.0-only

//! Rust implementation of the generic DT-backed cpufreq driver.

use kernel::{
    c_str,
    clk::Clk,
    cpu, cpufreq,
    cpumask::CpumaskVar,
    device::Core,
    error::code::*,
    macros::vtable,
    module_platform_driver, opp, platform,
    prelude::*,
    str::CString,
    sync::Arc,
};

struct PolicyState {
    opp_table: opp::Table,
    freq_table: opp::FreqTable,
    _cpus: CpumaskVar,
    _config_token: Option<opp::ConfigToken>,
    clk: Clk,
}

impl cpufreq::PolicyResources for PolicyState {
    fn freq_table(&self) -> Option<&cpufreq::Table> {
        Some(&self.freq_table)
    }

    fn clk(&self) -> Option<&Clk> {
        Some(&self.clk)
    }
}

#[derive(Default)]
struct CpufreqDtDriver;

#[vtable]
impl opp::ConfigOps for CpufreqDtDriver {}

#[vtable]
impl cpufreq::Driver for CpufreqDtDriver {
    const NAME: &'static CStr = c_str!("cpufreq-dt");
    const FLAGS: u16 = cpufreq::flags::NEED_INITIAL_FREQ_CHECK | cpufreq::flags::IS_COOLING_DEV;
    const BOOST_ENABLED: bool = false;

    type PData = Arc<PolicyState>;

    fn init(policy: &mut cpufreq::Policy) -> Result<Self::PData> {
        let cpu = policy.cpu();

        match cpu::with_device(cpu, |cpu_dev| {
            let mut cpus = CpumaskVar::new_zero(GFP_KERNEL)?;
            cpus.set(cpu);

            let config_token = regulator_config(cpu_dev, cpu)?;
            let mut fallback = false;

            match opp::Table::of_sharing_cpus(cpu_dev, &mut cpus) {
                Ok(()) => {}
                Err(err) if err == ENOENT => {
                    if opp::Table::sharing_cpus(cpu_dev, &mut cpus).is_err() {
                        fallback = true;
                    }
                }
                Err(err) => return Err(err),
            }

            let mut opp_table = match opp::Table::from_of_cpumask(cpu_dev, &mut cpus) {
                Ok(table) => table,
                Err(err) if err == EPROBE_DEFER => return Err(err),
                Err(_) => match opp::Table::from_dev(cpu_dev) {
                    Ok(table) => table,
                    Err(_) => {
                        dev_err!(cpu_dev, "OPP table can't be empty\n");
                        return Err(ENODEV);
                    }
                },
            };

            match opp_table.opp_count() {
                Ok(count) if count > 0 => {}
                _ => {
                    dev_err!(cpu_dev, "OPP table can't be empty\n");
                    return Err(ENODEV);
                }
            }

            if fallback {
                cpus.setall();
                if let Err(err) = opp_table.set_sharing_cpus(&mut cpus) {
                    dev_err!(cpu_dev, "failed to mark OPPs as shared: {:?}\n", err);
                }
            }

            let transition_latency_ns = match u32::try_from(opp_table.max_transition_latency_ns()) {
                Ok(0) | Err(_) => cpufreq::ETERNAL_LATENCY_NS,
                Ok(latency) => latency,
            };

            policy
                .set_suspend_freq(opp_table.suspend_freq())
                .set_transition_latency_ns(transition_latency_ns)
                .set_dvfs_possible_from_any_cpu(true);
            cpus.copy(policy.cpus());

            let freq_table = match opp_table.cpufreq_table() {
                Ok(table) => table,
                Err(err) => {
                    dev_err!(cpu_dev, "failed to init cpufreq table: {:?}\n", err);
                    return Err(err);
                }
            };
            let clk = Clk::get(cpu_dev, None)?;

            Arc::new(
                PolicyState {
                    opp_table,
                    freq_table,
                    _cpus: cpus,
                    _config_token: config_token,
                    clk,
                },
                GFP_KERNEL,
            )
            .map_err(Into::into)
        }) {
            Err(err) if err == ENODEV => Err(EPROBE_DEFER),
            other => other,
        }
    }

    fn exit(_policy: &mut cpufreq::Policy, _data: Option<Self::PData>) -> Result {
        Ok(())
    }

    fn online(_policy: &mut cpufreq::Policy) -> Result {
        Ok(())
    }

    fn offline(_policy: &mut cpufreq::Policy) -> Result {
        Ok(())
    }

    fn suspend(policy: &mut cpufreq::Policy) -> Result {
        policy.generic_suspend()
    }

    fn verify(data: &mut cpufreq::PolicyData) -> Result {
        data.generic_verify()
    }

    fn target_index(policy: &mut cpufreq::Policy, index: cpufreq::TableIndex) -> Result {
        let data = policy.data::<Self::PData>().ok_or(ENODEV)?;
        let freq = data.freq_table.freq(index)?;
        data.opp_table.set_rate(freq)
    }

    fn get(policy: &mut cpufreq::Policy) -> Result<u32> {
        policy.generic_get()
    }

    fn register_em(policy: &mut cpufreq::Policy) {
        policy.register_em_opp();
    }

    fn set_boost(policy: &mut cpufreq::Policy, state: i32) -> Result {
        policy.boost_set_sw(state)
    }
}

struct DtPlatformDriver;

impl platform::Driver for DtPlatformDriver {
    type IdInfo = ();
    const OF_ID_TABLE: Option<kernel::of::IdTable<Self::IdInfo>> = None;

    fn probe(
        pdev: &platform::Device<Core>,
        _id_info: Option<&Self::IdInfo>,
    ) -> Result<Pin<KBox<Self>>> {
        let config = registration_config(pdev);
        cpufreq::Registration::<CpufreqDtDriver>::new_foreign_owned_with(pdev.as_ref(), &config)?;

        Ok(KBox::new(Self, GFP_KERNEL)?.into())
    }
}

module_platform_driver! {
    type: DtPlatformDriver,
    name: "cpufreq-dt",
    authors: [
        "Viresh Kumar <viresh.kumar@linaro.org>",
        "Shawn Guo <shawn.guo@linaro.org>",
        "Rust for Linux Contributors",
    ],
    description: "Generic DT based cpufreq driver",
    license: "GPL",
}

fn regulator_config(cpu_dev: &kernel::device::Device, cpu: u32) -> Result<Option<opp::ConfigToken>> {
    let supply = if cpu == 0 && cpu_dev.property_present(c_str!("cpu0-supply")) {
        Some("cpu0")
    } else if cpu_dev.property_present(c_str!("cpu-supply")) {
        Some("cpu")
    } else {
        dev_dbg!(cpu_dev, "no regulator for cpu{}\n", cpu);
        None
    };

    let Some(supply) = supply else {
        return Ok(None);
    };

    let mut names = KVec::new();
    let supply = if supply == "cpu0" {
        CString::try_from(c_str!("cpu0"))?
    } else {
        CString::try_from(c_str!("cpu"))?
    };
    names.push(supply, GFP_KERNEL)?;

    opp::Config::<CpufreqDtDriver>::new()
        .set_regulator_names(names)?
        .set(cpu_dev)
        .map(Some)
}

fn registration_config(pdev: &platform::Device<Core>) -> cpufreq::RegistrationConfig {
    let mut config = cpufreq::RegistrationConfig::new();

    let Some(data) = cpufreq::dt_platform_data(pdev) else {
        return config;
    };

    if data.have_governor_per_policy() {
        config.add_flags(cpufreq::flags::HAVE_GOVERNOR_PER_POLICY);
    }

    if let Some(callback) = data.resume_raw() {
        config.set_resume_raw(callback);
    }

    if let Some(callback) = data.suspend_raw() {
        config.set_suspend_raw(callback);
    }

    match (data.get_intermediate_raw(), data.target_intermediate_raw()) {
        (Some(get), Some(target)) => {
            config
                .set_get_intermediate_raw(get)
                .set_target_intermediate_raw(target);
        }
        (None, None) => {}
        _ => {
            dev_warn!(
                pdev.as_ref(),
                "ignoring incomplete platform intermediate callbacks\n"
            );
        }
    }

    config
}
