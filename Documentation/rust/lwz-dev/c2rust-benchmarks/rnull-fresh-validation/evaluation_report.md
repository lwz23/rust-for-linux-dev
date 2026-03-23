# `rnull` evaluation report

## 1. 任务结论

这次 `rnull` fresh workflow validation rerun 已经完成从 bindings 审计、helper
审计、抽象补齐、blind-first 驱动、hardening、memory-debug guest A/B、official
compare 到最终文档的完整流程复测。

结论不是“完全通过”，而是：

- 对冻结的 `main.c core + configfs` 目标，已经拿到了大量静态和 runtime 证据。
- 但整轮仍不能宣称达到完整的 C 金标准功能等价。
- 根本原因不是 Rust candidate 全面失败，而是冻结目标中的 `mbps` 节流项在
  C baseline 上仍只收敛到 `runtime_gap`，因此 `runtime_state` 必须保持
  `unvalidated`。

## 2. 冻结目标

冻结的 `c_function_target` 是：

- `drivers/block/null_blk/main.c` 的 core + configfs 子集
- 纳入：
  - blk-mq request/completion 主路径
  - configfs 设备创建/删除与 `power` 生命周期
  - `submit_queues` / `poll_queues`
  - `shared_tags` / `shared_tag_bitmap`
  - `irqmode`
  - `memory_backed` / `cache_size` / `fua`
  - `badblocks`
  - `mbps`
  - 默认 `nr_devices`
  - 参数约束与拒绝路径
- 排除：
  - `zoned.c`
  - `CONFIG_BLK_DEV_NULL_BLK_FAULT_INJECTION`

这个范围没有在执行中被静默缩小。

## 3. 工程实现与抽象补齐

本轮补齐的公共抽象主要落在：

- `rust/kernel/configfs.rs`
- `rust/kernel/time/hrtimer.rs`
- `rust/kernel/block/badblocks.rs`
- `rust/kernel/block/mq/{request,tag_set,gen_disk,operations,status}.rs`

驱动层保持零 `unsafe`。`unsafe` 主要收敛在：

- configfs intrusive callback / container_of 边界
- hrtimer callback 边界
- blk-mq request/tagset/gendisk 边界
- generic helper bridge

这次 blind-first 严格遵守了：

- `bindgen -> helper -> abstraction -> driver`

并且在 official compare 前没有读取官方 `rnull` 源。

## 4. runtime 结果

memory-debug profile 下的 guest A/B 结果如下。

### C baseline

- `baseline`：通过
- `lifecycle`：通过
- `matrix-core`：通过
- `io-core`：基本通过
  - basic I/O：通过
  - discard / badblocks / irqmode none-softirq-timer / poll：通过
  - `mbps`：仅记录为 `runtime_gap`

### Rust candidate

- `baseline`：通过
- `lifecycle`：通过
- `matrix-core`：通过
- `io-core`：通过
  - basic I/O：通过
  - discard / badblocks / irqmode none-softirq-timer / poll：通过
  - `mbps`：在当前 harness 上观察到了明显节流

### A/B 结论

- `baseline` / `lifecycle` / `matrix-core`：对齐良好
- `io-core`：大部分路径对齐良好
- `mbps`：不能闭环
  - Rust candidate 有节流证据
  - C baseline 在同一 harness 上只能记为 `runtime_gap`

因此不能宣称完整功能等价。

## 5. 与官方 `rnull` 的比较

### 相比官方初版 `bc5b...`

官方初版真实文件位置是 `drivers/block/rnull.rs`，不是后来的目录结构。

官方初版只覆盖：

- 固定参数
- 不可配置
- direct completion
- 4KiB block size

它没有：

- configfs
- 生命周期管理
- shared tags
- queue count updates
- timer / poll
- memory-backed I/O
- discard
- badblocks
- `mbps`

所以本轮实现相对官方初版显著更宽，但这是为了追冻结的 C 金标准，而不是无意扩张。

### 相比 current-tree mature `63c37d...`

current-tree 参考已经具备：

- 独立目录 `drivers/block/rnull/`
- configfs
- `blocksize/size/rotational/irqmode`
- direct/softirq completion

但它仍然没有本轮冻结目标里的大部分功能面：

- queue count updates
- shared tags
- poll queues
- timer completion
- memory-backed storage
- discard
- badblocks
- `fua/cache/virt_boundary`
- `mbps`

因此本轮与 current-tree 的主要差异不是“功能少于官方”，而是“为了追 C 主线语义，我们的功能面更宽；同时工程上引入了更多本地抽象和 baseline 兼容层”。

## 6. 差异分类

### 版本演进

- `bc5b...` 单文件、固定功能
- `63c37d...` 独立目录 + configfs

这属于官方版本演进，不是本轮流程问题。

### 旧树约束

- `configfs_attrs!` 在本 baseline 目标形状上不可用，只能保留手工 `AttributeList`
  路径并把 `unsafe` 下沉到公共抽象
- generic blk-mq / hrtimer bridge helpers 仍需保留

### 工作流程缺陷

- guest runner 曾因 busybox `sh` 全局变量污染而无限循环
- guest I/O tool 最初漏掉 delayed writeback error
- helper audit 最初还停留在 planned 状态，没有反映真实 helper 面

这些问题都在本轮过程中被暴露并修正了。

### 真正工具失败

没有发现一个独立、稳定、最终仍未修复的 blind-first 工具失败。

未闭环项主要是：

- old-tree constraint
- runtime harness / C baseline `mbps` closure gap

## 7. 手册结论

这次复测说明当前手册里以下规则是有效的：

- 必须先做 bindgen 和 helper 审计，再做抽象和驱动
- 驱动层零 `unsafe`
- runtime A/B 对纯软件设备是强制项
- official compare 必须放在 runtime 之后
- 必须显式区分功能差异、工程差异、旧树约束

仍然不够细的地方：

- runtime harness 也需要像驱动一样有“hardening checklist”
- shell guest runner 的变量污染、错误传播、SMP 假设需要更早被约束
- lower-bound 官方参考的真实文件位置和目录演进需要在手册里写得更死

建议新增的硬规则：

- runtime 工具和 guest runner 也必须做一次“错误传播审计”
- 涉及 queue-count / poll-queue 的场景必须显式固定 guest CPU 数
- 若冻结目标含 `mbps` 一类性能行为，手册应要求先证明 C baseline 在同一 harness
  上能得到稳定证据，再把它当作 Rust 等价判定项

## 8. 最终判定

- 功能上是否达到 C 金标准目标：不能完整声称达到
- 工程上是否满足 Rust for Linux 要求：大体满足 blind-first、抽象优先、驱动零
  `unsafe` 的要求，但仍带有 old-tree 兼容层和 helper 退役未决项
- 是否存在 runtime 证据：存在，而且覆盖了大部分冻结目标
- 是否存在 runtime gap：存在，关键是 C baseline `mbps`
- 是否能宣称功能等价：不能

这次 rerun 已经证明当前流程手册大体可执行，也确实能把许多“看起来像风格问题”的
东西压缩成真实的工程 / 功能 / workflow 证据；但它同样证明，runtime harness
本身必须被纳入硬规则，而不能只把注意力放在驱动代码上。
