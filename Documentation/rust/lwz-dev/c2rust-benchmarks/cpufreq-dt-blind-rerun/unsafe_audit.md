# `cpufreq-dt` Unsafe Audit

## Summary

- Driver layer `drivers/cpufreq/rcpufreq_dt.rs` currently has zero `unsafe`.
- Driver layer currently has zero direct `bindings::` use.
- Remaining `unsafe` is concentrated in `rust/kernel/*`, where it serves FFI ownership, callback bridging, or raw C field access.

## Kernel-layer unsafe sites

### `rust/kernel/cpufreq.rs`

- `RegistrationConfig` / `Registration::new_with_config()`
  - Transfers shared driver config into `struct cpufreq_driver::driver_data`.
  - Obligation: reclaim exactly once on registration failure or unregister.
- callback wrappers
  - Borrow driver-wide config back from `cpufreq_get_driver_data()`.
  - Obligation: callbacks must not outlive registration-owned shared config.
- policy private data attach / clear
  - Stores and reclaims policy-owned state through `policy->driver_data`.
  - Obligation: error rollback and `exit` must remain paired.

### `rust/kernel/platform.rs`

- `Device::platdata<T>()`
  - Reads the raw `platform_data` field and converts it through `PlatData`.
  - Obligation: only immutable, long-lived typed views are exposed to safe code.

### `rust/kernel/cpufreq_dt.rs`

- `cpufreq_dt_pdev_register` export shim
  - Accepts a raw C `struct device *` and forwards to `platform_device_register_full()`.
  - Obligation: the exported function must remain a thin seam wrapper and not accumulate driver logic.

## Hardening results

- `cpufreq_driver.driver_data`
  - Audited as paired with `Registration::new_with_config()` / `Drop for Registration<T>`.
  - Ownership is reclaimed only after `cpufreq_unregister_driver()` returns, so callback re-borrows do not outlive the registered driver.
- `PolicyState` teardown
  - The driver now keeps the field order `clock -> freq_table -> opp_table -> regulator -> cpus -> primary_cpu`.
  - This preserves a drop order that is close to the C teardown pairs:
    - `clk_put`
    - `dev_pm_opp_free_cpufreq_table`
    - `dev_pm_opp_of_cpumask_remove_table`
    - `dev_pm_opp_put_regulators`
- `unsafe impl Send/Sync`
  - No driver-layer `unsafe impl Send/Sync` was introduced.
  - The only remaining Send/Sync site is the pre-existing kernel-layer `cpufreq::Registration<T>` wrapper.
- Helper retirement
  - The cpufreq-specific helper count stayed at `0`, so there is no helper retirement debt for this rerun.

## Current verdict

- Blind-first engineering goal is satisfied at the driver layer.
- Unsafe concentration goal is satisfied for the rerun's driver delta.
- Residual unsafe obligations remain in `rust/kernel/*`, not in the driver itself.
