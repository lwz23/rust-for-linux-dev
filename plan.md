# Standalone Replay Validation Plan

## Goal

Validate that the standalone `C2SafeRust` tooling repository can drive a fresh
external `rust-next` kernel worktree through the full `nlmon` smoke-path loop.

This branch is the second half of the standalone validation:

- sample family: `net-link-type`
- sample module: `drivers/net/nlmon.c`
- validation depth: full Kbuild/bindings/helpers/abstraction/driver/safety/gate/oracle loop

Unlike the first `et1011c` replay, this round must validate functional smoke,
not just module lifecycle:

- `ip link add nlmon0 type nlmon`
- `ip link set nlmon0 up`
- `ip link show nlmon0`
- `ip link set nlmon0 down`
- `ip link del nlmon0`

## Execution Model

- External tool repo:
  - `/home/lwz/rfl-dev/c2saferust-kernel-tooling`
- Tool branch:
  - `validation/standalone-replay-v1`
- Tool baseline commit for this replay:
  - `30ae8f3` (`Refine net-phy safety rule boundaries`)
- Target kernel tree:
  - `/home/lwz/rfl-dev/worktrees/standalone-nlmon-replay-v1`
- Base:
  - `rust-next`

All tool invocations must use:

```bash
python3 /home/lwz/rfl-dev/c2saferust-kernel-tooling/scripts/c2saferust/tool_cli.py ...
```

and pass:

```bash
--kernel-tree /home/lwz/rfl-dev/worktrees/standalone-nlmon-replay-v1
```

## Phases

### Phase A: External-tree bootstrap

- add `plan.md` / `task_context.md`
- generate the first `nlmon` artifact set from the external tool repo
- confirm artifact references stay inside this kernel tree

### Phase B: Replay the full `nlmon` loop

- apply Kbuild switch
- apply bindings/helper patches
- extend the required net abstractions
- land `drivers/net/nlmon_rust.rs`
- keep driver side zero-unsafe and free of `bindings::*`

### Phase C: Validate

- refresh artifacts
- run:
  - `verify-safety`
  - `gate-agent-candidate`
  - `run-oracle`
- use the `ip link add/up/show/down/del` smoke path as the oracle boundary

### Phase D: Standalone findings

- if the replay exposes standalone tool problems, fix them in the external tool
  repo first
- rerun tool self-tests
- rerun the external-tree workflow from artifacts forward

## Success Criteria

- the fresh external tree can be driven entirely by the standalone tool repo
- artifact generation and artifact references point into this kernel tree
- the replay reaches:
  - `verify-safety.pass = true`
  - `gate-agent-candidate.ready_for_smoke = true`
  - `run-oracle.pass = true`
- the `ip link add/up/show/down/del` smoke loop passes under QEMU
- any additional fixes needed here are generic standalone tool fixes, not
  `nlmon`-specific path hacks
