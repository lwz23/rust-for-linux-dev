# PHY Reference Round Plan

## Goal

Build the next `C2SafeRust` tool round on top of a clean `rust-next` tree, using
Ethernet PHY drivers as the new validation family.

This round is not a continuation of the `nlmon` code. Only the generalized
tooling is reused. The kernel-side target is a new, cleaner reference case that
fits current Rust-for-Linux upstream direction better than reimplementing an
existing link-type netdevice.

## Why PHY

- `rust/kernel/net/phy.rs` already exists upstream-side and provides an accepted
  abstraction style.
- `drivers/net/phy/ax88796b_rust.rs` and `drivers/net/phy/qt2025.rs` provide
  real Rust PHY precedents.
- Small PHY drivers let us validate whether the tool can map a C callback table
  into a zero-unsafe Rust driver under an existing subsystem abstraction.

## Family Strategy

- Family: `net-phy`
- Stage 1 sample: `drivers/net/phy/et1011c.c`
- Stage 2 sample: `drivers/net/phy/qsemi.c`

`et1011c` is first because its callback shape already fits the current
`kernel::net::phy::Driver` surface (`config_aneg`, `read_status`).

`qsemi` is intentionally deferred until the tool can describe and constrain new
interrupt-related PHY abstraction work (`config_init`, `config_intr`,
`handle_interrupt`).

## Tooling Objectives

1. Generalize the profile system from `net-link-type` to another family.
2. Keep phase 1/2 outputs static and structured:
   - Kbuild plan
   - bindings/helper patch plans
   - abstraction plan
   - unsafe obligation set
   - translation plan
   - safety policy / soundness discharge
3. Add a new oracle path for drivers where functional smoke is naturally
   module-lifecycle oriented instead of `ip link` oriented.
4. Keep driver generation constrained by artifacts and safety gates:
   - driver side must stay zero-unsafe
   - driver side must not use `bindings::*`
   - abstraction unsafety must remain in reviewed allowlisted files only

## Deliverables

### Phase A: Baseline and Profiles

- New `plan.md` / `task_context.md`
- `Documentation/rust/c2saferust/README.md`
- `net-phy` family/module/scenario/rule-pack profiles

### Phase B: Tool Generalization

- `intake.py` supports a second callback family via profile-driven analysis
- `oracle_runners.py` supports module lifecycle smoke
- `smoke.py` routes scenario execution through the generic runner layer

### Phase C: Static Artifacts

- Generate first artifact set for `et1011c`
- Record blockers and readiness in `task_context.md`

### Phase D: Minimal Rust Driver Loop

- Add `ET1011C_RUST_PHY` Kbuild switch
- Add the minimum missing safe PHY abstraction surface needed by `et1011c`
- Implement `drivers/net/phy/et1011c_rust.rs`
- Keep driver side zero-unsafe

### Phase E: Validation

- Refresh artifacts
- Run tool tests
- Run `verify-safety`
- Run `gate-agent-candidate`
- If feasible, run the module lifecycle oracle

## Success Criteria

- The same tool framework can describe both `net-link-type` and `net-phy`.
- `et1011c` can be planned and translated without ad-hoc, module-only logic.
- The Rust driver has no `unsafe`, no raw pointers, and no direct `bindings::*`.
- All remaining abstraction unsafety is explicitly captured by artifacts and the
  safety verifier.
