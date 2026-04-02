# TASK_CONTEXT

- Updated: 2026-04-01
- Scope: `/home/lwz/rfl-dev/worktrees/nlmon-tooling-clean-v1`
- Snapshot source: local filesystem + `git` branch state + same-tree official Rust references

## Task Goal

本 worktree 用于在干净 `rust-next` 基线上验证一个真正面向内核驱动的
`C2SafeRust` 工具原型。

核心目标不是“直接做出一个 `nlmon` Rust 驱动”，而是：

- 让工具能够从内核 C 模块出发，
- 沿着 Rust for Linux 的标准迁移流程，
- 产出静态 intake、抽象层计划、驱动计划与动态 oracle，
- 并用 `drivers/net/nlmon.c` 跑通第一轮样本验证。

## Workspace Summary

- 当前 worktree 对应分支：`feature/nlmon-tooling-clean-v1`
- 基线提交：`79e25710e722` (`rust-next`)
- 当前 worktree 创建方式：从 `linux/rust-next` 新 fork，不复用任何 `lwz23`
  的 `nlmon-*` 历史分支作为实现基底
- 允许参考的同树官方样本：
  - `drivers/net/phy/ax88796b_rust.rs`
  - `rust/kernel/net/phy.rs`

## Ground Truth

### 已确认事实

- 干净 `rust-next` 上已经包含官方 `ax88796b_rust.rs`，因此同树官方参考可直接使用。
- 基线 `rust-next` 上的原始事实是：
  - `drivers/net/nlmon.c` 存在
  - `drivers/net/Makefile` / `drivers/net/Kconfig` 中没有 `NLMON_RUST`
  - `rust/kernel/net.rs` 只导出 `net::phy`
  - 没有 `nlmon` 需要的 `rtnl/netdevice/skbuff` Rust 抽象
- 当前 worktree 已在该干净基线上继续前进，开始把阶段 1/2 的计划结果落成真实
  patch，并补最小 MVP 抽象层。
- 因此对 `nlmon` 来说，这一轮的顺序已演进为：
  - 阶段 1：Kbuild 集成方案 + 真实 patch
  - 阶段 2：bindings/helpers 审计 + 真实 patch
  - 阶段 3：最小 Rust net 抽象落地，并继续产出结构化计划
  - 阶段 4：受 artifact 约束的驱动层生成

### 明确不是基线的一部分

- `/home/lwz/rfl-dev/worktrees/nlmon-blind-rust`
- `/home/lwz/rfl-dev/linux` 当前 `feature/nlmon-rust` 上的未提交文档与工具
- 任何 `lwz23` 之前为 `nlmon` 额外添加的 abstraction / helper / Makefile / Kconfig 变更

这些内容只能当“经验样本”使用，不能直接当作本分支的起点。

## Current Status

当前已完成：

- 新建干净 worktree 与分支
- 确认 `HEAD == rust-next`
- 确认当前分支无额外历史提交
- 编写 `plan.md`
- 落地第一版静态 intake 工具
- 扩展 `scripts/c2saferust`，新增 patch-plan / abstraction / unsafe artifact 生成能力
- 扩展 `scripts/c2saferust`，新增 `translation-plan.json` 生成能力
- 扩展 `scripts/c2saferust`，新增 `safety-policy.json` /
  `soundness-discharge.json` / `safety-verdict.json` 生成与校验能力
- 扩展 `scripts/c2saferust`，新增 `agent-workflow-plan.json` /
  `agent-gate-report.json`，把 agent workflow 强制收敛到
  `translation-readiness -> verify-safety -> compile-driver-object`
- 在真实树上落地阶段 1/2 patch：
  - `drivers/net/Kconfig` 新增 `config NLMON_RUST`
  - `drivers/net/Makefile` 新增 `CONFIG_NLMON_RUST` 下的 `nlmon_rust.o` 条件切换
  - `rust/bindings/bindings_helper.h` 新增
    `linux/if_arp.h`、`linux/netdevice.h`、`linux/netlink.h`、
    `net/rtnetlink.h`
  - `rust/helpers/net.c` 新增 `dev_kfree_skb` / `dev_lstats_add` /
    `netdev_priv` 的 Rust helper 包装
  - `rust/helpers/helpers.c` 已纳入 `net.c`
- 在真实树上落地 MVP Rust net 抽象：
  - `rust/kernel/net.rs` 已导出 `rtnl` / `netdevice` / `netlink_tap` /
    `skbuff` / `stats`
  - `rust/kernel/net/rtnl.rs` 已提供最小 `Registration<T: Driver>` /
    `ValidateContext` / `NlAttrTable`
  - `rust/kernel/net/netdevice.rs` 已提供最小 `Device` 包装与
    `Operations` trait，并补 `TxOutcome` / `ndo_start_xmit` vtable wiring
  - `netdevice::Operations::Private` 已收紧为 `Zeroable`，把
    “`net_device` 私有区零初始化有效”编码为通用抽象契约
  - `rust/kernel/net/netlink_tap.rs` 已提供最小 tap 注册/注销封装
  - `netlink_tap::Tap` 已显式声明 zeroable 初始状态
  - `rust/kernel/net/skbuff.rs` 已提供 move-only skb owner
  - `rust/kernel/net/stats.rs` 已提供 `dev_lstats_add` 安全包装
- 在真实树上落地最小 `drivers/net/nlmon_rust.rs`：
  - 模块壳体为 `kernel::InPlaceModule` + `rtnl::Registration<NlmonDriver>`
  - 私有状态为 `NlmonPriv { tap: netlink_tap::Tap }`
  - 已实现 `setup/validate/open/stop/start_xmit`
  - 当前只 defer `get_stats64/ethtool`
  - 已补 `alias = rtnl-link-nlmon`
- 已完成当前可用验证：
  - `python3 -m unittest scripts/c2saferust/tests/test_tool_cli.py` 通过
  - `python3 scripts/c2saferust/tool_cli.py bootstrap-nlmon --output-dir Documentation/rust/c2saferust/nlmon`
    通过
  - `python3 scripts/c2saferust/tool_cli.py verify-safety --output Documentation/rust/c2saferust/nlmon/safety-verdict.json`
    通过
  - `python3 scripts/c2saferust/tool_cli.py plan-agent-workflow --output Documentation/rust/c2saferust/nlmon/agent-workflow-plan.json`
    通过
  - `python3 scripts/c2saferust/tool_cli.py gate-agent-candidate --output Documentation/rust/c2saferust/nlmon/agent-gate-report.json`
    通过，且 `ready_for_smoke = true`
  - 驱动侧静态检查确认未直接调用被 artifact 禁止的 raw rtnl/netlink/helper FFI
- 当前环境状态：
  - 已通过用户级本地 LLVM 15 工具链解除 `libclang >= 15` 阻塞：
    - `/tmp/llvm-15-local/usr/bin/clang-15`
    - `/tmp/llvm-15-local/usr/bin/llvm-config-15`
    - `/tmp/llvm-15-local/usr/bin/ld.lld-15`
    - `/tmp/llvm-15-local/usr/lib/x86_64-linux-gnu/libclang.so`
  - 在该环境下
    `PATH=/tmp/llvm-15-local/usr/bin:$PATH LIBCLANG_PATH=/tmp/llvm-15-local/usr/lib/x86_64-linux-gnu LD_LIBRARY_PATH=/tmp/llvm-15-local/usr/lib/x86_64-linux-gnu${LD_LIBRARY_PATH:+:$LD_LIBRARY_PATH} make LLVM=-15 rustavailable`
    已输出 `Rust is available!`
- 当前源码验证 blocker：
  - 已解除：
    - `rust/kernel/net/netdevice.rs` 已改为对齐 bindgen 实际布局：
      - `dev->type_`
      - `priv_flags/lltx` 使用 bindgen 生成的 union bitfield setter
      - `pcpu_stat_type` 使用 bindgen 生成 setter
    - `open/stop` trampoline 已从“经 `&mut self` 派生私有区引用”改为
      “private raw pointer -> 局部 `&mut Private`”模型，避开 borrow 冲突
  - 当前最小编译验证已通过：
    - `make O=/tmp/nlmon-build LLVM=-15 -j$(nproc) drivers/net/nlmon_rust.o`
  - 已在真实内核构建中验证
    `drivers/net/nlmon_rust.rs` 顶部 `#![forbid(unsafe_code)]` 可生效，且不会与
    当前 Rust for Linux 驱动构建规则冲突
  - 另外已把 `NETIF_F_*` bit 组合上收到 `netdevice` 抽象层；
    `NLMSG_GOODSIZE` 由于不是直接暴露的绑定常量，当前由驱动按内核宏等价公式静态推导
- 当前 QEMU smoke 新发现：
  - 已通过
    `python3 scripts/c2saferust/tool_cli.py run-smoke-qemu --output Documentation/rust/c2saferust/nlmon/toolchain-run-record.json --qemu-log-output Documentation/rust/c2saferust/nlmon/qemu-smoke.log --build-dir /tmp/nlmon-build`
    进入真实 QEMU
  - 观测结果：
    - `ip link add nlmon0 type nlmon` 成功
    - `ip link set nlmon0 up` 返回成功
    - 随后内核在 `dev_hard_start_xmit()` 路径触发空函数指针调用并 panic
  - `qemu-smoke.log` / `toolchain-run-record.json` 证明：
    当前把 `nlmon_xmit` 继续标记为 deferred 是错误的；至少对于
    `ip link add/up/down/del` 这条 smoke 闭环，`ndo_start_xmit` 已经是 MVP blocker
  - 因此当前 artifact 与真实运行行为出现偏差，继续开发前必须先更新
    `translation-plan.json` / `abstraction-plan.json` / `unsafe-obligations.json`
    对 `xmit` 的范围定义
  - 下一步修正边界已经明确为：
    - 将 `nlmon_xmit` 从 deferred 前移到 MVP required callback
    - 将 `skbuff` 与 `stats` 中的 TX 最小表面前移为 smoke blocker
    - `start_xmit` 抽象必须把 `skb` 所有权封装在 abstraction 内，驱动层保持
      `#![forbid(unsafe_code)]` 且不暴露 raw pointer / `bindings::*`
    - `start_xmit` 不应向驱动层暴露 `&mut Private` 这类对并发 TX 路径不成立的
      独占借用；需要改为更保守、可证明 sound 的接口
- 当前已完成的修正（待真实编译 / smoke 复验）：
  - `scripts/c2saferust/intake.py` 已改为把
    `nlmon_xmit` 视为 MVP required callback
  - `abstraction-plan.json` / `translation-plan.json` /
    `unsafe-obligations.json` / `safety-policy.json` /
    `agent-workflow-plan.json` 的生成逻辑已同步收敛到新的 smoke 边界
  - `rust/kernel/net/netdevice.rs` 已扩展 `TxOutcome` 与 `start_xmit` vtable wiring
  - `rust/kernel/net/skbuff.rs` 已新增 move-only skb owner
  - `rust/kernel/net/stats.rs` 已新增 `dev_lstats_add` 安全包装
  - `drivers/net/nlmon_rust.rs` 已补最小 `start_xmit`
  - `scripts/c2saferust/tests/test_tool_cli.py` 已同步到新的 MVP / verifier 规则，
    且本地 Python 单测通过
- 当前下一步：
  - 已完成：
    - `python3 scripts/c2saferust/tool_cli.py verify-safety --output Documentation/rust/c2saferust/nlmon/safety-verdict.json`
      通过，`pass = true`
    - `python3 scripts/c2saferust/tool_cli.py gate-agent-candidate --output Documentation/rust/c2saferust/nlmon/agent-gate-report.json --build-dir /tmp/nlmon-build --make-llvm -15`
      通过，`ready_for_smoke = true`
    - `make O=/tmp/nlmon-build LLVM=-15 -j$(nproc) bzImage` 通过
    - `python3 scripts/c2saferust/tool_cli.py run-smoke-qemu --output Documentation/rust/c2saferust/nlmon/toolchain-run-record.json --qemu-log-output Documentation/rust/c2saferust/nlmon/qemu-smoke.log --build-dir /tmp/nlmon-build --timeout-sec 120`
      通过，`pass = true`
  - 当前 smoke 结论：
    - `ip link add nlmon0 type nlmon`：`add_rc = 0`
    - `ip link set nlmon0 up`：`up_rc = 0`
    - `ip link show nlmon0`：`show_rc = 0`
    - `ip link set nlmon0 down`：`down_rc = 0`
    - `ip link del nlmon0`：`del_rc = 0`
    - `toolchain-run-record.json` 当前 `result = PASS`
  - 本轮实验结论：
    - 先前 panic 的根因确认为 `ndo_start_xmit` 缺失
    - 对 `nlmon` 这类 netdevice，`xmit` 不能继续被视为 post-MVP defer
    - 一个更 sound 的最小抽象边界是：
      - driver `start_xmit` 只拿到 move-only `SkBuff`
      - 只拿到共享 `&Device`，而不是并发语义不成立的 `&mut Device` /
        `&mut Private`
      - `TxOutcome::Ok` / `TxOutcome::Busy(skb)` 显式表达 skb 所有权归属
- 重生成 `nlmon` 的静态 artifact：
  - `Documentation/rust/c2saferust/nlmon/kbuild-plan.json`
  - `Documentation/rust/c2saferust/nlmon/binding-gap-audit.json`
  - `Documentation/rust/c2saferust/nlmon/helper-audit.json`
  - `Documentation/rust/c2saferust/nlmon/kbuild-patch-plan.json`
  - `Documentation/rust/c2saferust/nlmon/bindings-patch-plan.json`
  - `Documentation/rust/c2saferust/nlmon/helpers-patch-plan.json`
  - `Documentation/rust/c2saferust/nlmon/abstraction-plan.json`
  - `Documentation/rust/c2saferust/nlmon/unsafe-obligations.json`
  - `Documentation/rust/c2saferust/nlmon/translation-plan.json`
  - `Documentation/rust/c2saferust/nlmon/safety-policy.json`
  - `Documentation/rust/c2saferust/nlmon/soundness-discharge.json`
  - `Documentation/rust/c2saferust/nlmon/safety-verdict.json`
  - `Documentation/rust/c2saferust/nlmon/agent-workflow-plan.json`
  - `Documentation/rust/c2saferust/nlmon/agent-gate-report.json`

当前这批 artifact 的关键发现：

- 阶段 1/2 的计划已经转化为真实 patch；后续工具需要能识别
  `already_applied` 状态，而不是持续输出待打补丁项。
- 当前重生成后的 patch-plan 均已进入 `already_applied`：
  - `kbuild-patch-plan.json`
  - `bindings-patch-plan.json`
  - `helpers-patch-plan.json`
- `bindings-patch-plan.json` 已把 bindings 补丁范围收紧到 MVP 所需最小集合：
  - 需要新增到 `bindings_helper.h` 的头文件是
    `linux/if_arp.h`、`linux/netdevice.h`、`linux/netlink.h`、
    `net/rtnetlink.h`
  - `linux/module.h` / `linux/kernel.h` 不应直接靠 bindgen 暴露给驱动，
    而应由 `kernel::ThisModule` / Rust abstraction 吸收
  - `net/net_namespace.h` 目前被明确标记为 post-MVP defer 项
- `helper-audit.json` / `helpers-patch-plan.json` 现已明确：
  - `rust/helpers/` 缺 `netdev_priv` / `dev_lstats_add`
  - `dev_kfree_skb` 也应被视为 macro/helper gap，而不是 direct FFI
  - 推荐新增 `rust/helpers/net.c`，并在 `rust/helpers/helpers.c` 中纳入
- MVP Rust net 抽象现已覆盖 smoke 所需最小表面，因此
  `abstraction-plan.json` 已从“仅覆盖 `rtnl/netdevice/netlink_tap`”继续推进到
  “`rtnl/netdevice/netlink_tap/stats/skbuff` 全部就位”。
- `abstraction-plan.json` 当前已给出：
  - `phase_4_readiness.ready_for_minimal_driver_codegen = true`
  - MVP area 状态：
    - `rtnl`：implemented
    - `netdevice`：implemented
    - `netlink_tap`：implemented
    - `stats`：implemented（smoke-path MVP）
    - `skbuff`：implemented（smoke-path MVP）
- `unsafe-obligations.json` 已把后续生成必须满足的关键 unsafe 义务显式化：
  - `rtnl` 注册生命周期
  - `net_device` 私有数据布局/转换
  - `netlink_tap` 的 `dev/module` 指针有效性与 open/stop 配对
  - `tb[IFLA_ADDRESS]` 的索引/边界语义
  - `skb` 单次消费与 `stats64` 输出缓冲区写入语义
- `translation-plan.json` 当前已把最小驱动翻译范围静态化：
  - 模块壳体建议为 `kernel::InPlaceModule` + `rtnl::Registration<NlmonDriver>`
  - 私有状态建议为 `NlmonPriv { tap: netlink_tap::Tap }`
  - 私有状态初始化策略已明确为：
    `RTNL/net core zero-initialized private storage + Private: Zeroable`
  - 回调映射固定为：
    - `nlmon_setup -> rtnl::Driver::setup`
    - `nlmon_validate -> rtnl::Driver::validate`
    - `nlmon_open -> netdevice::Operations::open`
    - `nlmon_close -> netdevice::Operations::stop`
    - `nlmon_xmit -> netdevice::Operations::start_xmit`
  - `nlmon_get_stats64` / `always_on` 当前仍 defer
  - 驱动侧禁止直接调用
    `bindings::rtnl_link_register` / `bindings::netlink_add_tap` /
    `bindings::netdev_priv` / direct helper exports
- 真实 QEMU smoke 现已完成回证：
  - `nlmon_xmit` 前移为 MVP required callback 之后，`ip link add/up/show/down/del`
    闭环通过
  - `nlmon_get_stats64` / `always_on` 在当前 smoke 闭环下仍可 defer
- `safety-policy.json` / `soundness-discharge.json` / `safety-verdict.json`
  当前已把“sound abstraction + zero-unsafe driver”进一步工具化：
  - driver 侧显式要求 `#![forbid(unsafe_code)]`
  - driver 侧仍独立禁止 `unsafe` / `bindings::*` / `extern "C"` / raw pointer token
  - abstraction 侧要求：
    - `rtnl::ValidateContext` 必须暴露 typed `LinkAttr` / `InfoAttr`
    - `netlink_tap::Tap` 必须以 `Pin<&mut Self>` 形式进行 add/remove
    - `rtnl::Registration<T>` 必须保留 `build_assert!(!needs_drop::<T::Private>())`
  - Rust verifier 会检查：
    - allowlisted abstraction 文件中的每个 `unsafe` proof site 是否被 discharge 覆盖
    - structural rule 是否仍成立
    - driver 是否违反 forbidden token 约束
  - 结论：`#![forbid(unsafe_code)]` 已证明“可用”，但只能作为 driver 侧附加保险，
    不能替代 soundness artifact + verifier，因为它无法覆盖 abstraction proof/discharge、
    raw FFI surface 泄漏与结构性 soundness 规则
- `agent-workflow-plan.json` / `agent-gate-report.json` 当前已把
  “agent 必须受 artifact 约束且必须过 gate” 进一步流程化：
  - `agent-workflow-plan.json` 固定：
    - agent 必读 artifact
    - agent 允许写入的 managed files
    - deferred callbacks 边界
    - driver 必需 crate attribute（当前为 `#![forbid(unsafe_code)]`）
    - acceptance gate 顺序
  - `agent-gate-report.json` 当前固定 gate 顺序为：
    - `translation-readiness`
    - `verify-safety`
    - `compile-driver-object`
  - 只有当 `agent-gate-report.json.ready_for_smoke == true` 时，
    后续 smoke/QEMU/oracle 才应继续
  - 当前这一轮已进一步把该链路补全为：
    `artifact -> verify-safety -> compile-driver-object -> qemu smoke`
    并已由 `toolchain-run-record.json` 记录真实 PASS 结果

## First Milestone

第一里程碑定义为：

- 工具能在干净树上生成 `nlmon` 的阶段 1/2 静态 artifact
- 后续 agent 能在这些 artifact 约束下编写抽象层与驱动层
- 最终候选能在 QEMU 中通过 `ip link add nlmon0 type nlmon`

注意：

- 第一里程碑不要求 `C vs Rust` 立刻 `equal`
- 但必须要求 oracle 流程真实、可重跑、可导出 diff artifact

## Tool Entry Points

当前工具入口为：

- `scripts/c2saferust/tool_cli.py intake-kbuild`
- `scripts/c2saferust/tool_cli.py audit-bindings`
- `scripts/c2saferust/tool_cli.py plan-patches`
- `scripts/c2saferust/tool_cli.py plan-abstractions`
- `scripts/c2saferust/tool_cli.py plan-translation`
- `scripts/c2saferust/tool_cli.py plan-safety-policy`
- `scripts/c2saferust/tool_cli.py plan-agent-workflow`
- `scripts/c2saferust/tool_cli.py verify-safety`
- `scripts/c2saferust/tool_cli.py gate-agent-candidate`
- `scripts/c2saferust/tool_cli.py bootstrap-nlmon`

推荐的第一条命令：

- `python3 scripts/c2saferust/tool_cli.py bootstrap-nlmon --output-dir Documentation/rust/c2saferust/nlmon`

## Next Steps

1. 以当前 smoke-pass 结果为基线，决定是否把 `run-smoke-qemu` 进一步纳入
   agent workflow 的正式 acceptance gate
2. 继续扩展 post-MVP 面：`get_stats64` / `ethtool`
3. 为更强 oracle/diff 闭环准备 C vs Rust 动态比对 artifact
4. 将当前 soundness 规则推广到更多 netdevice 样本，而不只限于 `nlmon`
