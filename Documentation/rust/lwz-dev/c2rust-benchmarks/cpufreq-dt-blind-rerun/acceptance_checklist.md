# `cpufreq-dt` Acceptance Checklist

- `[x]` Stage 0 environment freeze completed in a fresh worktree and fresh build directory.
- `[x]` `module_manifest.yaml`, `dut-spec.json`, and `worklog` were created before driver development.
- `[x]` `c-structure-map.json` was produced from the required C/Kconfig/Makefile inputs.
- `[x]` Main bindings were extended before any helper was introduced.
- `[x]` The cpufreq-specific helper count remained `0`.
- `[x]` Public abstraction work preceded the blind-first driver and landed in `rust/kernel/*`.
- `[x]` The blind-first driver keeps driver-layer `unsafe` at `0`.
- `[x]` The blind-first driver keeps driver-layer direct `bindings::` use at `0`.
- `[x]` C/Rust config snapshots were saved for both runtime environments.
- `[x]` A minimal Kconfig switching integration patch was applied and classified as benchmark harness switching integration / old-tree constraint only.
- `[x]` Runtime harness scripts were created under `tools/testing/rust/cpufreq-dt/`.
- `[x]` Runtime was attempted in `env_a.sabrelite` and `env_b.mcimx7d-sabre`.
- `[x]` C baseline was always run before Rust candidate in each tested environment.
- `[x]` Full QEMU logs were saved under `test-results/cpufreq-dt/`.
- `[ ]` Trusted policy-level A/B runtime validation was achieved.
- `[ ]` Functional equivalence can be claimed.
- `[x]` `runtime_state` remains `unvalidated` because runtime gaps / baseline observability gaps remain open.
- `[x]` Official compare was deferred until after hardening and runtime work.

## Open acceptance blockers

- Neither tested environment produced policy sysfs, governor, or frequency-table evidence for the C baseline.
- `runtime_validated_core` therefore remains empty.
- `env_b` retains an open runtime difference candidate in deferred-probe visibility between C and Rust.
