# C2SafeRust `nlmon` v1 工具开发计划

- Updated: 2026-04-01
- Branch: `feature/nlmon-tooling-clean-v1`
- Baseline: `rust-next@79e25710e722`
- Worktree: `/home/lwz/rfl-dev/worktrees/nlmon-tooling-clean-v1`

## 1. 目标

本分支的目标不是“手工完成一个 `nlmon_rust.rs`”，而是：

- 在一棵从干净 `rust-next` 派生的新分支上，
- 开发一套面向内核驱动 Rust 化的 `C2SafeRust` 工具原型，
- 并用 `drivers/net/nlmon.c` 作为第一个实验样本，
- 跑通从静态 intake 到动态 oracle 的第一轮完整工具工作流。

第一里程碑不是 `C vs Rust equal`，而是：

- 工具能在干净树上产出 `nlmon` 的阶段 1/2 静态 artifact；
- 工具能约束后续 abstraction/driver 生成；
- 工具产出的 `nlmon` Rust 候选能真实编译、进入 QEMU，并通过
  `ip link add nlmon0 type nlmon` 的 baseline smoke。

## 2. 工作流分解

### Phase 0：基线约束

- 保持 `linux` 仓库的 `rust-next` 分支不被直接修改。
- 所有开发都只在当前 worktree 中进行。
- 所有结论必须带有结构化 artifact、命令与证据路径。

### Phase 1：Kbuild 集成规划

输出：

- `Documentation/rust/c2saferust/nlmon/kbuild-plan.json`

要求：

- 识别 `nlmon.c` 当前的 `Kconfig` / `Makefile` 入口。
- 产出面向 `NLMON_RUST` 的切换方案。
- 参考同树官方样本 `ax88796b_rust.rs` 的配置与 Makefile 组织方式。

### Phase 2：bindings / helpers 审计

输出：

- `Documentation/rust/c2saferust/nlmon/binding-gap-audit.json`
- `Documentation/rust/c2saferust/nlmon/helper-audit.json`

要求：

- 审计 `nlmon.c` 顶部头文件依赖。
- 区分：
  - 已被 `bindings_helper.h` 覆盖的头文件
  - 需要新增到 `bindings_helper.h` 的头文件
  - 需要通过 `rust/helpers/*.c` 提供薄封装的 inline/helper
- 显式标记 `netdev_priv` / `dev_lstats_add` 这类 helper gap。

### Phase 3：抽象层计划

输出：

- `Documentation/rust/c2saferust/nlmon/abstraction-plan.json`
- `Documentation/rust/c2saferust/nlmon/unsafe-obligations.json`

要求：

- 明确 `rust-next` 当前已有 Rust 网络抽象仅包含 `net::phy`；
- 对 `nlmon` 需要的 `rtnl/netdevice/skbuff/netlink tap/stats` 抽象作出
  结构化需求分解；
- agent 生成代码时只能消费这些 artifact，不允许无约束自由发挥。

### Phase 4：驱动层生成

输出：

- `Documentation/rust/c2saferust/nlmon/translation-plan.json`
- `drivers/net/nlmon_rust.rs`

要求：

- 第一轮先完成最小可运行闭环：
  - `module!`
  - `rtnl::Registration`
  - `rtnl::Driver`
  - `setup/validate/open/stop` 的最小实现
- 目标是让 `ip link add nlmon0 type nlmon` 成功，不是第一轮就实现完全等价。

### Phase 4.5：agent gate

输出：

- `Documentation/rust/c2saferust/nlmon/agent-workflow-plan.json`
- `Documentation/rust/c2saferust/nlmon/agent-gate-report.json`

要求：

- agent 只能在 `agent-workflow-plan.json` 指定的 managed files 范围内修改代码；
- agent 必须先读取 `translation-plan.json` / `safety-policy.json` /
  `soundness-discharge.json`；
- agent 产出必须先通过：
  - `translation-readiness`
  - `verify-safety`
  - `compile-driver-object`
- 只有当 `agent-gate-report.json.ready_for_smoke == true` 时，才能进入 smoke。

### Phase 5：动态 oracle

输出：

- `Documentation/rust/c2saferust/nlmon/toolchain-run-record.json`
- `Documentation/rust/c2saferust/nlmon/normalized-diff.json`
- build/rootfs/qemu/capture 证据

要求：

- 必须确认测到的是本轮真实重编产物；
- 必须导出 baseline capture；
- 若 diff 不等价，必须固定 `first differing normalized event`。

## 3. 本轮立刻执行内容

本轮先完成基础设施与 Phase 1/2 的第一版：

- 创建干净分支与 worktree
- 编写 `plan.md`
- 编写 `task_context.md`
- 落地第一版静态 intake 工具：
  - `scripts/c2saferust/tool_cli.py`
  - `scripts/c2saferust/intake.py`
  - `scripts/c2saferust/tests/test_tool_cli.py`
- 用工具生成 `nlmon` 的第一批静态 artifact

## 4. 关键约束

- `ax88796b_rust.rs` 是同树官方参考样本，但不是 `nlmon` 的实现模板。
- `nlmon` 是实验样本，目标是验证工具工作流，而不是单独交付一个驱动。
- 阶段 1/2 的结论应尽量由静态工具直接生成。
- 阶段 3/4 可以使用 agent，但必须被结构化 artifact 强约束。

## 5. 近期待办

- 将 Phase 1/2 产物跑出并固化到 `Documentation/rust/c2saferust/nlmon/`
- 根据静态审计结果补 Phase 3 的 abstraction plan
- 再进入 `nlmon` 的最小运行闭环实现
