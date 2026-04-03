# Task Context

## Round

- Worktree: `/home/lwz/rfl-dev/worktrees/phy-reference-clean-v1`
- Branch: `feature/phy-reference-clean-v1`
- Base: `upstream/rust-next`
- Focus family: `net-phy`
- Stage 1 module: `drivers/net/phy/et1011c.c`
- Stage 2 module: `drivers/net/phy/qsemi.c`

## Intent

This round validates whether the generalized `C2SafeRust` workflow can be moved
from `net-link-type` drivers to Ethernet PHY drivers without falling back to a
one-off manual rewrite.

The primary output remains the tool and its artifact-constrained workflow, not
just a single Rust driver.

## Confirmed Baseline

- Fresh worktree was created from clean `rust-next`.
- Only `scripts/c2saferust/` was copied forward from the previous round.
- No `nlmon` kernel code or experiment artifacts were carried into this tree.
- `rust/kernel/net/phy.rs` and existing Rust PHY drivers are available as
  reference style:
  - `drivers/net/phy/ax88796b_rust.rs`
  - `drivers/net/phy/qt2025.rs`
- `et1011c.c` fits current Rust PHY callback coverage:
  - `config_aneg`
  - `read_status`
- `qsemi.c` is a deliberate second-stage target because it needs new PHY IRQ
  callback surfaces:
  - `config_init`
  - `config_intr`
  - `handle_interrupt`

## Current Assessment

- The imported tool is already generic at the CLI/workflow level, but static
  source analysis is still biased toward `net-link-type` callback tables.
- The runner layer already exists, but only the `ip link`/QEMU scenario is
  implemented.
- Safety policy and soundness discharge already support profile-driven rules and
  are suitable for extension to `net-phy`.

## Phase Status

### Phase A: Baseline and Profiles

- Status: completed
- Completed work:
  - new `plan.md`
  - new `task_context.md`
  - new `Documentation/rust/c2saferust/README.md`
  - new `net-phy` family/module/scenario/rule-pack profiles
- Result:
  - the new round is now formally detached from the old `nlmon` framing
  - `et1011c` / `qsemi` are first-class tool profiles rather than ad-hoc notes

### Phase B: Tool Generalization

- Status: completed
- Completed work:
  - `intake.py` now dispatches source analysis by profile-driven source model
  - `net-phy` uses a dedicated `phy_driver` source model
  - `oracle_runners.py` now contains a generic `qemu-module-lifecycle` runner
  - `smoke.py` now renders scenario inputs generically instead of assuming
    `ip link`-style inputs
  - LLVM auto-detect now verifies the local toolchain is actually runnable
    instead of blindly trusting the `/tmp/llvm-15-local` path
- Current result:
  - `python3 -m py_compile scripts/c2saferust/*.py` passes
  - `refresh-artifacts --module-path drivers/net/phy/et1011c.c` succeeds
  - dedicated `net-phy` regression coverage has been added to
    `scripts/c2saferust/tests/test_tool_cli.py`

### Phase C: Static Artifacts

- Status: completed
- Completed work:
  - first artifact set for `et1011c` has been generated under
    `Documentation/rust/c2saferust/et1011c/`
- Final static finding:
  - `bindings-patch-plan.json`: no new bindings are needed for the first
    `et1011c` Rust loop
  - `helpers-patch-plan.json`: no helper wrappers are needed
  - `abstraction-plan.json`: `phy` and `phy_config_aneg` are both implemented
  - `translation-plan.json`: ready for minimal driver codegen
  - `agent-workflow-plan.json`: ready for agent codegen with a single
    abstraction allowlist file (`rust/kernel/net/phy.rs`)
  - `qsemi` planning artifacts also generate successfully and explicitly block
    stage 2 on `phy_irq_callbacks` and `phy_config_init`

### Phase D: Minimal `et1011c` Rust Driver

- Status: completed
- Completed work:
  - added `LSI_ET1011C_RUST_PHY` Kconfig switch
  - added Rust/C object selection in `drivers/net/phy/Makefile`
  - added safe `phy::Device::genphy_config_aneg()` wrapper
  - added zero-unsafe `drivers/net/phy/et1011c_rust.rs`
- Driver-side status:
  - no driver-side `unsafe`
  - no direct `bindings::*`
  - no driver-owned raw pointers
  - no Rust private state introduced
- Notable design decision:
  - the C driver's global `static int speed` was deliberately not translated;
    the Rust driver keeps the status/config decision local to `read_status`
    instead of introducing cross-device mutable state

### Phase E: Validation

- Status: completed
- Completed validation:
  - `python3 -m py_compile scripts/c2saferust/*.py`
  - `python3 -m unittest scripts.c2saferust.tests.test_tool_cli`
  - `verify-safety` for `et1011c`: pass
  - `gate-agent-candidate` for `et1011c`: pass
  - `run-oracle` with `qemu-module-lifecycle`: pass
- Concrete outputs:
  - build dir: `/tmp/c2saferust-et1011c-build`
  - runnable LLVM 15 copy: `/home/lwz/.local/c2saferust-llvm-15/usr`
  - safety verdict:
    `Documentation/rust/c2saferust/et1011c/safety-verdict.json`
  - gate report:
    `Documentation/rust/c2saferust/et1011c/agent-gate-report.json`
  - oracle record:
    `Documentation/rust/c2saferust/et1011c/toolchain-run-record.json`
  - oracle log:
    `Documentation/rust/c2saferust/et1011c/qemu-smoke.log`

## Hard Constraints

- Keep the goal tool-centric, not manual-driver-centric.
- Treat artifact generation as the authoritative planning source.
- Keep driver code zero-unsafe.
- Keep direct `bindings::*` usage out of the driver.
- If implementation reality diverges from plan, update this file before
  continuing.
