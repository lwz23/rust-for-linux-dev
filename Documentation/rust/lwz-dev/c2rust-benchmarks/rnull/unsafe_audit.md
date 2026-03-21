# `rnull` normalized unsafe audit

- 生成日期：2026-03-21
- 校准类型：historical calibration rerun

## 当前结论

`rnull` 的驱动层保持零 `unsafe`。新增 `unsafe`、raw pointer、FFI bridge 和
`unsafe impl Send/Sync` 主要集中在：

- `rust/kernel/configfs.rs`
- `rust/kernel/block/mq/request.rs`
- `rust/kernel/block/mq/operations.rs`
- `rust/kernel/block/mq/gen_disk.rs`

本轮 hardening 后，没有发现“safe 驱动不写 `unsafe` 也能直接破坏已注册 block
对象或 configfs child 生命周期”的明显路径，但 runtime 证据仍然缺失。

## 重点不变式

### Request completion

- 前置条件：`queue_rq` 拿到的请求已经由 blk-mq 交给驱动，并且私有 PDU 已初始化。
- 后置条件：请求必须只完成一次，且 `ARef<Request<_>>` 的 refcount 交接必须与
  blk-mq completion callback 配对。
- safe 破坏面：驱动层只能通过 `Request::end_ok` 或 `Request::complete` 完成请求，
  不能直接操作 raw `struct request *`。

### QueueData / GenDisk / TagSet

- 前置条件：queue data 在 `GenDiskBuilder::build(..., queue_data)` 时转成 foreign-owned
  指针，并与 live disk 绑定。
- 后置条件：`GenDisk` drop 必须在 `del_gendisk` 后回收 queue data，且 tagset 需要
  继续存活到磁盘注销完成。
- safe 破坏面：驱动层拿到的 queue data 只是借用引用，不能手动释放或移动。

### Configfs child lifecycle

- 前置条件：`make_group` 分配的 child group 只能在匹配的 release callback 中回收。
- 后置条件：如果 child 仍持有 live disk，则 `drop_item` 与 final release 两条路径都
  必须被审计，避免遗漏提前断电或重复释放。
- safe 破坏面：驱动层只能通过受控的 `set_power()` 路径切换 live state；配置更新在
  powered-on 状态下会被拒绝。

## `Send/Sync` 审计结论

- 驱动层没有 `unsafe impl Send/Sync`。
- 抽象层保留的 `unsafe impl Send/Sync` 已记录在
  `unsafe-obligations.json`。
- 当前仍需保留关注：
  - `rust/kernel/configfs.rs` 的 wrapper `Send/Sync` 主要依赖“底层对象由内核同步”
    这条结构性证明；
  - 这部分没有被升级成 runtime 结论，只能算 hardening 期的静态审计结论。

## 当前仍保留的风险

- 没有可信 runtime harness 来证明 configfs create/delete、power on/off、以及
  基本 I/O completion 在真实内核中的行为。
- blind-write 额外支持了 `Timer` irqmode，这属于 scope expansion，已经在
  `difference_ledger.yaml` 中保持 `open`。
