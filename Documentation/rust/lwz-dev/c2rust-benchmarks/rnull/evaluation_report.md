# `rnull` evaluation report

- 生成日期：2026-03-21
- 当前等级：`prototype-grade`
- benchmark 类型：historical calibration rerun

## 结论

这次 `rnull` 工作是一次 calibration rerun，不自动计入新的 formal benchmark
配额。它已经完成：

- 基于真实 `pre-rnull` 语义基线的 blind-first；
- hardening 后的 official compare；
- 结构化 artifact、差异台账和 `unsafe` 审计刷新。

它当前可以宣称：

- 风格状态已经达到 `benchmark-calibrated`；
- 驱动层保持零 `unsafe`；
- `None` / `Soft` request completion 与 queue-data ownership 没发现明确
  soundness 弱化。

它当前不能宣称：

- 已完成可信 runtime 验证；
- 已达到 `engineering-grade` 或 `mainline-grade`；
- 已与 current-tree `rnull` 完全对齐。

## compare 三问结论

### 1. blind-write 是否错误扩大或缩小了范围

有扩大，没有缩小。

- 扩大点一：blind-write 仍支持 `Timer` irqmode，而 current-tree `rnull`
  只支持 `None` / `Soft`。
- 扩大点二：blind-write root `features` 多列出了 `power`。

### 2. configfs 建模是否与 current-tree `rnull` 一致

只部分一致。

- 一致处：都把 live block device 放在 configfs child 生命周期之下，并在 powered-on
  时禁止配置漂移。
- 差异处：blind-write 用 `config + live` 两把 mutex 并在 `drop_item` 显式断电；
  current-tree `rnull` 用单 mutex，主要依赖 `disk: Option<GenDisk<_>>` 的最终 drop。

### 3. request completion / queue data ownership 是否有 soundness 弱化

对官方共享子集没有发现明确弱化。

- `None` / `Soft` 路径下，请求只完成一次、queue data 由 `GenDisk` drop 回收，这条
  所有权链是闭合的。
- 额外的 `Timer` 路径属于 scope expansion，不应当被包装成“与官方同等对齐”。

## 主要证据

- `/home/lwz/rfl-dev/linux` 已补全 shallow 历史，并重新定位出真实基线
  `3253aba3408aa4eb2e4e09365eede3e63ef7536b` 与引入提交
  `bc5b533b91ef0b8a09fe507e23d1c6c43c1fb0f5`。
- C / Rust 对象级切换构建均有通过证据。
- `drivers/block/rnull/*.rs` 驱动层零 `unsafe`。
- compare 后已把所有差异归类到 `difference_ledger.yaml`。

## 阻塞项

- 没有 configfs create/delete 与基本 I/O 的可信 runtime harness。
- blind-write 与 official reference 仍有 scope mismatch。
- configfs 新抽象的 `Send/Sync` 仍是结构性证明，不是 runtime 证据。
