# `rnull` normalized unsafe audit

- 生成日期：2026-03-23
- 当前阶段：`hardening completed, memory-debug runtime finished, official compare finished`

## 当前结论

`rnull` 驱动层当前保持零 `unsafe`。显式 `unsafe`、raw pointer、FFI bridge
和 intrusive callback 只保留在：

- `rust/kernel/configfs.rs`
- `rust/kernel/time/hrtimer.rs`
- `rust/kernel/block/badblocks.rs`
- `rust/kernel/block/mq/{request,tag_set,gen_disk,operations}.rs`
- `rust/helpers.c`

本轮 hardening 和 runtime 前修复额外收紧了三点：

- configfs 手工属性表初始化不再把 `unsafe` 留在驱动文件中；
- configfs / power / queue-update 相关状态迁移现在由 controller 级生命周期锁串行化，
  去掉了原先 `config` 锁与 `runtime` 锁反向获取的死锁窗口。
- `badblocks` safe wrapper 现在会先检查启用状态，避免 disabled 状态下落到
  C helper 的 `WARN_ON`。

## 重点不变式

### `configfs`

- 前置条件：subsystem / group / item 在整个 configfs 生命周期内保持 pinned。
- 后置条件：`make_group` / `drop_item` / `release` 对同一个 Rust 对象只完成一次重建与销毁配对。
- safe 破坏面：驱动文件不再直接写 AttributeList 的 raw slot。

### `HrTimer`

- 前置条件：timer owner 在 callback 存活期间不可移动。
- 后置条件：延后完成与节流队列里的请求只能在 timer/poll/timeout 三条路径之一被终结。
- safe 破坏面：driver 只拿到安全的 `start/cancel/forward` 能力，不碰 raw `hrtimer *`。

### `Request` / completion

- 前置条件：request private data 由 blk-mq 按请求生命周期创建与释放。
- 后置条件：一次请求只能成功完成一次；超时和 poll 清理必须把同一个 `opaque_id` 从所有挂起队列中摘掉。
- safe 破坏面：driver 不直接做 `blk_mq_rq_to_pdu` 和 raw request 释放。

### `TagSet` / `GenDisk`

- 前置条件：tag set 成功注册后才允许进入 gendisk 构建与 queue 映射。
- 后置条件：`blk_mq_free_tag_set`、`del_gendisk` 与 queue_data foreign ownership 都要一一配对。
- safe 破坏面：共享 tag set 仍依赖 `TagSet: Send + Sync` 的正确性，但
  `matrix-core` 的 `shared_tags` memory-debug runtime 已经覆盖到 powered-on
  queue-count update 且未出现新的内核异常。

## 当前仍保留的审计关注点

- `rust/helpers.c` 中剩余的 blk-mq / hrtimer helper 仍需后续 compare 阶段复核是否可退役。
- `Vec<PageSlot>` 存储模型仍归类为工程差异，但 memory-debug guest A/B 已覆盖其
  基本读写、flush、discard 和 badblocks 子路径。
- `mbps` 延后完成模型目前仍不能闭环，因为 C baseline 在同一 harness 上只能记为
  `runtime_gap`，所以整轮 frozen target 仍不能宣称功能等价。

## 归档用途

这份 `md` 版不是替代详细长文审计，而是把 benchmark schema 需要的关键 `unsafe`
边界、前后置条件和当前保留项压缩成 agent / 工具更容易复用的格式。
