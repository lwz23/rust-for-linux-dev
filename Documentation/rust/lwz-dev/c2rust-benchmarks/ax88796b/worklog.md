# `ax88796b` blind-write worklog

- 生成日期：2026-03-20
- 工作树：`/home/lwz/rfl-dev/worktrees/ax88796b-blind-rust`
- 分支：`feature/ax88796b-blind-rust`
- C 金标准：`drivers/net/phy/ax88796b.c`
- 只读参考树：`/home/lwz/rfl-dev/linux`
- 当前阶段：engineering-grade completed

## 冻结记录

- blind-write 禁读直到 hardening 完成：
  - `drivers/net/phy/ax88796b_rust.rs`
  - `rust/kernel/net/phy.rs`
  - 提交 `79e25710e7227228902d672417b552dd1d7e5d3b` 的具体 diff
- runtime 现在已从 `gap` 升级为 `passed`，依据是 benchmark-only KUnit +
  synthetic MDIO 的可重复差分验证。
- 所有输出工件固定写入当前目录，不污染只读参考树。

## 2026-03-20

- 已确认隔离 worktree 与分支存在且干净。
- 已确认 `ebbed9d02ece592c3e693db72197afad8de70af8` 基线中自带
  `ax88796b_rust.rs` 与 `AX88796B_RUST_PHY` 入口，因此按 benchmark
  方案先在 worktree 内做 blind baseline 回退。
- 已计划新增隔离 build/config wrapper，固定使用
  `/home/lwz/rfl-dev/build-ax88796b-blind`。
- 在一次过宽的符号搜索中意外命中了 `rust/kernel/net/phy.rs` 的匹配行；
  后续搜索已改为显式排除此文件，blind-first 设计不以该意外输出为依据。
- bindgen 审计结果显示以下需求均已由当前 bindings 覆盖：
  - `MII_BMCR`、`BMCR_SPEED100`、`BMCR_FULLDPLX`
  - `SPEED_10`、`SPEED_100`
  - `DUPLEX_HALF`、`DUPLEX_FULL`
  - `AUTONEG_ENABLE`、`PHY_IS_INTERNAL`、`phy_state_PHY_NOLINK`
  - `mdiobus_read`、`mdiobus_write`
  - `genphy_update_link`、`genphy_read_lpa`、`genphy_soft_reset`
  - `genphy_suspend`、`genphy_resume`
  - `phy_resolve_aneg_linkmode`、`phy_init_hw`、`_phy_start_aneg`
- blind-first 未新增 helper；当前判断为“bindings 充足，safe API 审计待 hardening”。
- blind-first 初版已恢复 `CONFIG_AX88796B_RUST_PHY` 切换入口，并保持驱动层零
  `unsafe`。
- 已通过定向构建：
  - `scripts/lwz-dev/ax88796b-build.sh c drivers/net/phy/ax88796b.o`
  - `scripts/lwz-dev/ax88796b-build.sh rust drivers/net/phy/ax88796b_rust.o`
- hardening 审计结论：
  - callback object model 继续采用 `Adapter` 回调桥接 +
    `Device::from_raw()` 的短生命周期借用；驱动未持有跨回调状态。
  - `Registration` 仍作为注册状态机边界：只接受已 pin 的 `'static`
    驱动表，注册成功后由 `Drop` 配对 `phy_drivers_unregister()`。
  - safe API 从 `set_speed(u32)` 收紧为
    `set_basic_speed(BasicSpeed)`，避免驱动层任意写入 `phydev->speed`。
  - 驱动切换到 `phy::flags::IS_INTERNAL`，不再直接依赖原始绑定常量。
  - 试验性移除 `unsafe impl Send/Sync` 后，`Module: Send + Sync`
    约束触发编译失败；因此保留最小必要的
    `unsafe impl Send/Sync for Registration`，并移除
    `unsafe impl Sync for DriverVTable`，把跨线程证明收敛到注册句柄。
  - blind-first 和 hardening 均未新增 helper；当前判断仍为 helper 不需要。
- hardening 后验证：
  - `scripts/lwz-dev/ax88796b-build.sh rust drivers/net/phy/ax88796b_rust.o`
  - `rg -n '\\bunsafe\\b' drivers/net/phy/ax88796b_rust.rs`
  - 一次将 C/Rust 定向构建并行投递到同一 build dir 的尝试触发了
    `fixdep` 竞争失败；随后已改为串行重跑并通过，后续验证遵循同一 build
    目录串行构建。
- hardening 提交：
  - `57b35a2c6` `ax88796b: harden blind rust phy abstractions`
- hardening 完成后已读取只读参考实现：
  - `/home/lwz/rfl-dev/linux/drivers/net/phy/ax88796b_rust.rs`
  - `/home/lwz/rfl-dev/linux/rust/kernel/net/phy.rs`
- reference compare 首轮结论：
  - callback table 形状、`soft_reset` / `read_status` / `link_change_notify`
    语义路径与参考实现一致。
  - benchmark 分支的主要差异集中在 safe API 收紧、`Send/Sync` 证明边界收缩，
    以及 benchmark 专用模块元数据与 Kconfig 文案。
- 本地参考树中的提交 `79e25710e7227228902d672417b552dd1d7e5d3b`
  以 grafted 形式出现，父提交不可解析；因此最终对照采用“hardening 后读取当前
  参考文件 + 文件级 diff”的方式，而不是恢复该提交父子 diff。
- engineering 阶段新增：
  - 已新增 benchmark-only KUnit harness：
    `drivers/net/phy/ax88796b_bench_kunit.c`
  - 已新增 runtime orchestration：
    `scripts/lwz-dev/ax88796b-kunit.sh`
    `scripts/lwz-dev/ax88796b-runtime-diff.py`
    `scripts/lwz-dev/ax88796b-engineering-check.sh`
  - 已新增 KUnit config 片段：
    `base.kunitconfig`
    `c.kunitconfig`
    `rust.kunitconfig`
  - C mode 首次运行暴露的是 harness 自身断言写得过宽，随后改为检查“回调前后
    无新增访问”而不是绝对零读写。
  - Rust mode 首次运行暴露了 AX88772A `read_status` 在 link-down 和
    autoneg-complete 路径上的语义漂移。
  - 根因定位到 `phy::Device` 里手写的 `phy_device` bit offset；该实现与当前树
    bindgen 布局不再一致。
  - benchmark 分支随后切换到 bindgen 生成的
    `phy_device::{link, autoneg, autoneg_complete}` accessor。
  - 修复后重新运行：
    - `scripts/lwz-dev/ax88796b-kunit.sh c`
    - `scripts/lwz-dev/ax88796b-kunit.sh rust`
    - `python3 scripts/lwz-dev/ax88796b-runtime-diff.py compare Documentation/rust/lwz-dev/c2rust-benchmarks/ax88796b/runtime-c.yaml Documentation/rust/lwz-dev/c2rust-benchmarks/ax88796b/runtime-rust.yaml Documentation/rust/lwz-dev/c2rust-benchmarks/ax88796b/runtime_diff.yaml`
  - 结果为：
    - C mode 18/18 通过
    - Rust mode 18/18 通过
    - `runtime_diff.yaml` 状态为 `match`
  - refreshed reference compare 发现当前只读参考 `phy.rs` 仍保留旧的手写
    bit offset；该差异已进入 `difference_ledger.yaml`。
  - 已将两条新经验回写本地手册：
    - callback-only phylib 行为只能经已绑定 vtable 触发
    - bindgen 已生成 bitfield accessor 时，不要在 safe wrapper 里重写 bit offset
  - `ax88796b-kunit.sh` 已补充日志级 `not ok` 检查，防止 KUnit 失败被脚本静默吞掉。
  - 最终端到端校验已通过：
    - `scripts/lwz-dev/ax88796b-engineering-check.sh`
