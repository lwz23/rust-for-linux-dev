# `cpufreq-dt` Evaluation Report

## Outcome

The rerun succeeded on the engineering axis more than on the runtime axis. It produced a blind-first Rust `cpufreq-dt` implementation with driver-layer zero `unsafe`, zero direct `bindings::` use, and a set of public abstractions that push unsafe concentration into `rust/kernel/*`. It did not produce enough runtime evidence to claim functional equivalence.

## Static and engineering assessment

- The rerun kept the full stage ordering discipline: bindings audit, helper audit, abstraction work, blind-first driver, hardening, runtime, then official compare.
- The implementation did not treat the official Rust initial file as the functional ceiling. It instead targeted the broader C gold scope, including the shared C seam and optional platform-data callback plumbing.
- The rerun added meaningful common abstractions in `cpu.rs`, `platform.rs`, `opp.rs`, `cpufreq.rs`, and a thin `cpufreq_dt.rs` seam module.
- The hardening audit closed the main driver-layer ownership concerns around `PolicyState`, `PlatformState`, and `cpufreq_driver.driver_data`.

## Runtime assessment

- `env_a.sabrelite`
  - Both C baseline and Rust candidate booted and loaded the cpufreq driver module.
  - Neither run exposed policy sysfs or deeper cpufreq observability.
  - Verdict: baseline observability gap for the generic standalone path.
- `env_b.mcimx7d-sabre`
  - Both C baseline and Rust candidate loaded the provider module and the cpufreq driver module.
  - Both runs exposed the `cpufreq-dt` platform device and driver directory, but the device stayed unbound and no policy sysfs appeared.
  - C emitted `platform cpufreq-dt: deferred probe pending`; Rust reached the same unbound state without the same dmesg line.
  - Verdict: provider-assisted baseline observability gap plus one open runtime difference candidate.

## Claim ceiling

- `static target covered`: yes, with one notable open boost-path gap.
- `runtime validated core`: none.
- `runtime gap paths`: generic standalone path, provider-assisted path beyond module load, policy-level observability, callback plumbing, and export-path coverage.
- Functional equivalence claim: not justified.

## Manual conclusion on the updated handbook

### Rules that worked well

- The strict blind-first ordering prevented premature copying from the official implementation.
- The explicit `runtime_state=unvalidated` discipline prevented over-claiming once QEMU evidence stalled below policy level.
- The config-parity rule was effective and kept C/Rust drift bounded even across two environments.
- The requirement to classify findings into functional difference, engineering difference, old-tree constraint, runtime harness defect, and baseline observability gap was highly useful in practice.

### Rules that still need more detail

- The runtime-harness guidance should say earlier and more explicitly that guest initramfs contents must be guest-architecture executable, not merely present on the host.
- The runtime runner guidance should explicitly require CRLF-safe marker parsing and a policy for QEMU instances that halt without exiting.
- The build instructions should recommend targeted dtb/module goals for single-board validation so large ARM `dtbs` builds do not dominate the rerun.

### Candidate new hard rules

- Add a pre-runtime hard rule: every staged guest executable in the initramfs must pass an architecture check before QEMU boot.
- Add a host-runner hard rule: serial verdict parsing must normalize carriage returns and record whether QEMU required forced stop.
- Add a runtime verdict hard rule: if the C baseline never reaches policy sysfs in an environment, that environment must be classified as a baseline observability gap immediately and cannot be summarized as A/B validated.
