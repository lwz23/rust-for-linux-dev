# `ax88796b` evaluation report

- 生成日期：2026-03-20
- 当前等级：`engineering-grade`
- benchmark 类型：current-tree small sample

## 结论

`ax88796b` 现在可以在这个 benchmark 分支内宣称 `engineering-grade`。

判断依据不是单次构建成功，而是完整闭环已经具备：

- blind-first、hardening、reference compare 已完成并刷新；
- C / Rust debug 完整构建通过；
- benchmark-only KUnit runtime 在 C mode 和 Rust mode 都通过了 18/18 场景；
- `runtime_diff.yaml` 对标准化 observation 做逐条精确比对，结果为 `match`；
- 驱动层仍保持零 `unsafe`。

它仍然不能宣称 `mainline-grade`，因为本轮运行证据来自 synthetic MDIO +
KUnit，而不是真实硬件 bring-up。

## 主要证据

### runtime 与差分

- `scripts/lwz-dev/ax88796b-kunit.sh c`
  生成 `runtime-c.yaml`，18 个场景全部通过。
- `scripts/lwz-dev/ax88796b-kunit.sh rust`
  生成 `runtime-rust.yaml`，18 个场景全部通过。
- `python3 scripts/lwz-dev/ax88796b-runtime-diff.py compare ...`
  生成 `runtime_diff.yaml`，状态为 `match`。
- `scripts/lwz-dev/ax88796b-engineering-check.sh`
  将 `git diff --check`、debug 构建、双模 KUnit 和最终 diff 串成可重复流水线。

### 功能等价范围

runtime matrix 覆盖了以下关键行为：

- AX88772A / AX88772C exact-match 绑定
- AX88796B model-mask 绑定
- 三个 ID 的 `soft_reset`
- `soft_reset` 首次写失败传播
- AX88772A `read_status` 的 link-down、autoneg complete、autoneg incomplete
- `BMCR` / `LPA` 读取失败传播
- AX88772A `link_change_notify`
- AX88772A / AX88772C `suspend` / `resume`

每个 scenario 都比较：

- 返回码
- `speed` / `duplex` / `link`
- `autoneg` / `autoneg_complete`
- `pause` / `asym_pause`
- `suspended`
- 最终 `BMCR`
- 读写计数
- 完整有序 MDIO trace

## hardening 与 soundness

- 驱动层保持零 `unsafe`。
- `Registration` 继续作为注册状态机边界，并保留最小必要的
  `unsafe impl Send/Sync`；结构性证明已写入 `unsafe_audit.md`。
- `set_speed(u32)` 仍被收紧为 `set_basic_speed(BasicSpeed)`，避免 safe 驱动任意
  写入 `phydev->speed`。
- 本轮 runtime 暴露出 `phy::Device` 对 `phy_device` bitfield 的手写 offset
  已经与当前树布局漂移。benchmark 分支已切换到 bindgen 生成的 accessor，
  并在修复后恢复了 C/Rust observation 的逐条一致。

## refreshed reference compare

hardening 后的首次 compare 已完成；本轮因生产抽象发生修正，又重新做了一次只读
参考 compare。

当前与参考树的主要差异是：

- safe speed setter 仍更窄
- `Send/Sync` 证明边界仍收敛在 `Registration`
- benchmark 元数据仍保持 blind-write 风格
- 本地参考历史仍无法恢复 `79e25710e...` 的父子 diff
- benchmark 分支已把 `phy_device` 状态读取改为 bindgen accessor，而当前参考
  `phy.rs` 仍手写 bit offset

这些差异均已写入 `difference_ledger.yaml`。

## 结论边界

当前允许写：

- benchmark 分支已达到 `engineering-grade`
- Rust 驱动与 C 金标准在当前 runtime matrix 上功能等价
- 当前 `unsafe` 证明没有留下未记录的结构性阻塞项

当前仍不允许写：

- 已达到 `mainline-grade`
- 已完成真实硬件 bring-up
- 当前参考树里的 phylib 抽象已经是最终上游形态
