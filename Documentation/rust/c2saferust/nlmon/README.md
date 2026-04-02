# `nlmon` Bootstrap Artifacts

该目录保存干净 `rust-next` 基线上的 `nlmon` 工具实验产物。

当前已生成：

- `kbuild-plan.json`
- `binding-gap-audit.json`
- `helper-audit.json`
- `kbuild-patch-plan.json`
- `bindings-patch-plan.json`
- `helpers-patch-plan.json`
- `abstraction-plan.json`
- `unsafe-obligations.json`
- `translation-plan.json`
- `safety-policy.json`
- `soundness-discharge.json`
- `safety-verdict.json`
- `agent-workflow-plan.json`
- `agent-gate-report.json`

这些文件当前覆盖到 Rust 化五步法中的前四步约束层，并补上 safety gate：

1. Kbuild 集成规划
2. bindings / helpers 审计
3. abstraction / unsafe 约束
4. 最小驱动翻译计划
5. soundness / zero-unsafe verifier
6. agent workflow / compile gate

当前结论：

- `drivers/net/Kconfig` / `drivers/net/Makefile` 已落 `NLMON_RUST` 切换
- `rust/bindings/bindings_helper.h` 已补齐 MVP 所需头文件
- `rust/helpers/net.c` 已补齐 `dev_kfree_skb` / `dev_lstats_add` /
  `netdev_priv` 包装
- `rust/kernel/net/{rtnl,netdevice,netlink_tap,skbuff,stats}.rs` 已落 smoke-path 最小 MVP 抽象
- patch-plan artifact 现已能正确反映 `already_applied`
- `translation-plan.json` 已把最小 `nlmon_rust.rs` 约束为：
  - 做 `setup/validate/open/stop/start_xmit`
  - defer `get_stats64/ethtool`
  - 禁止驱动侧直接调用 raw rtnl/netlink/helper FFI
- `safety-policy.json` 已把 driver 侧规则收紧为：
  - 必须包含 `#![forbid(unsafe_code)]`
  - 禁止 `unsafe` / `bindings::*` / `extern "C"` / raw pointer token
- `soundness-discharge.json` 已把 abstraction proof site 与 structural rule 显式化
- `safety-verdict.json` 表示当前树已通过 quick gate + Rust verifier
- `agent-workflow-plan.json` 已把 agent 允许读/写的文件范围、deferred callbacks、
  必须读取的 artifact 与必过 gate 固化为结构化 contract
- `agent-gate-report.json` 已把
  `translation-readiness -> verify-safety -> compile-driver-object`
  这条 acceptance 链跑通
- `drivers/net/nlmon_rust.rs` 已按上述约束落下最小实现
- 当前环境已解除本地 LLVM/libclang 15 阻塞，且
  `make O=/tmp/nlmon-build LLVM=-15 -j$(nproc) drivers/net/nlmon_rust.o`
  已通过
- `make O=/tmp/nlmon-build LLVM=-15 -j$(nproc) bzImage` 已通过
- `toolchain-run-record.json` / `qemu-smoke.log` 已记录当前 smoke 结果：
  - `ip link add nlmon0 type nlmon`
  - `ip link set nlmon0 up`
  - `ip link show nlmon0`
  - `ip link set nlmon0 down`
  - `ip link del nlmon0`
  - 全部返回成功，当前 `RESULT=PASS`
- 当前 agent gate 已可直接运行：
  - `python3 scripts/c2saferust/tool_cli.py plan-agent-workflow --output Documentation/rust/c2saferust/nlmon/agent-workflow-plan.json`
  - `python3 scripts/c2saferust/tool_cli.py gate-agent-candidate --output Documentation/rust/c2saferust/nlmon/agent-gate-report.json`

注意：

- `#![forbid(unsafe_code)]` 在当前内核 Rust 驱动构建中可用，但它只是 driver 侧附加保险
- 它不能替代 `safety-policy.json` + `soundness-discharge.json` + Rust verifier，
  因为 abstraction soundness、proof site 覆盖和结构性规则不在该 lint 的覆盖范围内
- 后续 agent 产出只有在 `agent-gate-report.json.ready_for_smoke == true` 时，
  才应进入下一步 smoke / oracle 流程
- 本轮 smoke 的关键经验已固定到 artifact：
  - `ndo_start_xmit` 是 `nlmon` smoke 闭环的 MVP blocker
  - sound `xmit` 抽象应使用 move-only `SkBuff` + shared `&Device`
  - 不应向驱动层暴露并发 TX 路径下不成立的 `&mut Private`
