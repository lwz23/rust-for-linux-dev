# Task Context

## Round

- Worktree:
  - `/home/lwz/rfl-dev/worktrees/standalone-nlmon-replay-v1`
- Branch:
  - `feature/standalone-nlmon-replay-v1`
- Base:
  - `rust-next@79e25710e722`
- External tool repo:
  - `/home/lwz/rfl-dev/c2saferust-kernel-tooling`
- External tool branch:
  - `validation/standalone-replay-v1`
- External tool baseline commit for this replay:
  - `30ae8f3`
- Focus family:
  - `net-link-type`
- Focus module:
  - `drivers/net/nlmon.c`

## Intent

This tree exists to validate the full standalone `nlmon` replay after the
external tool boundary has already been exercised successfully on
`et1011c`.

The concrete question here is:

- can a fresh `rust-next` worktree be driven from the external tool repo all
  the way through the `nlmon` smoke path, without copying tool scripts into the
  kernel tree?

## Confirmed Baseline

- this tree was created fresh from clean `rust-next`
- `drivers/net/nlmon.c` exists in the tree
- this kernel tree does not contain `scripts/c2saferust/`
- the standalone tool repo already passed a complete external-tree replay for
  `et1011c`
- no previous `nlmon` Rust implementation history was used as the branch base

## Phase Status

### Phase A: External-tree bootstrap

- Status: completed
- Completed work:
  - added `plan.md`
  - added `task_context.md`
  - ran the external tool repo against this tree
  - generated the first `nlmon` artifact set under
    `Documentation/rust/c2saferust/nlmon/`

### Phase B: Full `nlmon` replay

- Status: completed
- Completed work:
  - replayed the artifact-constrained `nlmon`
    Kbuild/bindings/helpers/abstraction/driver loop into this fresh tree
  - applied the minimal kernel-side file set required by the generated
    patch plans and abstraction plan
  - refreshed artifacts and confirmed
    `ready_for_minimal_driver_codegen = true`
- Landed file set:
  - `drivers/net/nlmon_rust.rs`
  - `drivers/net/Kconfig`
  - `drivers/net/Makefile`
  - `rust/bindings/bindings_helper.h`
  - `rust/helpers/helpers.c`
  - `rust/helpers/net.c`
  - `rust/kernel/net.rs`
  - `rust/kernel/net/netdevice.rs`
  - `rust/kernel/net/netlink_tap.rs`
  - `rust/kernel/net/rtnl.rs`
  - `rust/kernel/net/skbuff.rs`
  - `rust/kernel/net/stats.rs`

### Phase C: Validation

- Status: completed
- Outputs:
  - `Documentation/rust/c2saferust/nlmon/safety-verdict.json`
  - `Documentation/rust/c2saferust/nlmon/agent-gate-report.json`
  - `Documentation/rust/c2saferust/nlmon/toolchain-run-record.json`
  - `Documentation/rust/c2saferust/nlmon/qemu-smoke.log`
- Validation results:
  - `verify-safety.pass = true`
  - `gate-agent-candidate.pass = true`
  - `gate-agent-candidate.ready_for_smoke = true`
  - `run-oracle.pass = true`
  - `smoke_summary.result = PASS`
- Build/runtime notes:
  - build dir:
    `/tmp/standalone-nlmon-replay-build`
  - kernel image:
    `/tmp/standalone-nlmon-replay-build/arch/x86/boot/bzImage`
  - oracle boundary:
    `ip link add nlmon0 type nlmon && ip link set nlmon0 up && ip link show nlmon0 && ip link set nlmon0 down && ip link del nlmon0`

## Hard Constraints

- the standalone tool repo remains outside the kernel tree
- all tool invocations must use `--kernel-tree`
- artifact generation remains authoritative
- the target closure is the `ip link add/up/show/down/del` smoke path
- if reality diverges from plan, update this file before continuing
