# `ax88796b` blind-write worklog

- 生成日期：2026-03-20
- 工作树：`/home/lwz/rfl-dev/worktrees/ax88796b-blind-rust`
- 分支：`feature/ax88796b-blind-rust`
- C 金标准：`drivers/net/phy/ax88796b.c`
- 只读参考树：`/home/lwz/rfl-dev/linux`
- 当前阶段：stage 0 / stage 1 bootstrap

## 冻结记录

- blind-write 禁读直到 hardening 完成：
  - `drivers/net/phy/ax88796b_rust.rs`
  - `rust/kernel/net/phy.rs`
  - 提交 `79e25710e7227228902d672417b552dd1d7e5d3b` 的具体 diff
- runtime 当前固定为 `gap`，本轮不越级宣称 `engineering-grade` 或更高。
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
- 下一阶段将重点审查 phylib callback object model、safe API 宽度、
  `Send/Sync` 与生命周期边界。
