# `ax88796b` normalized unsafe audit

- 生成日期：2026-03-20
- 参考对照：`/home/lwz/rfl-dev/linux/drivers/net/phy/ax88796b_rust.rs`
  与 `/home/lwz/rfl-dev/linux/rust/kernel/net/phy.rs`

## 当前结论

`ax88796b` 的驱动层保持零 `unsafe`。当前 `unsafe`、raw pointer 和 FFI bridge
都被压缩在：

- `rust/kernel/net/phy.rs`
- `rust/kernel/net/phy/reg.rs`
- `kernel::module_phy_driver!` 展开的静态注册路径

本轮 runtime、diff 和 reference compare 完成后，没有发现阻塞
`engineering-grade` 的未证明 soundness 缺口。

## runtime-backed findings

- benchmark-only KUnit runtime 首次把 `read_status` 的语义漂移暴露成了可重复失败。
- 根因不是驱动层 `unsafe`，而是 `phy::Device` 对 `phy_device` bitfield 的手写
  offset 已与当前树布局漂移。
- benchmark 分支现已切换到 bindgen 生成的 `phy_device::{link, autoneg,
  autoneg_complete}` accessor。
- 修复后，C / Rust 18 个场景 observation 完全一致，`runtime_diff.yaml`
  状态为 `match`。

## 重点不变式

### `Device` callback bridge

- 前置条件：`Device::from_raw()` 只在 phylib 交付的回调上下文中调用。
- 后置条件：`&mut Device` 只在回调作用域内短暂存在，不跨回调保存。
- safe 破坏面：驱动层拿不到 raw `phy_device *`，只能通过安全包装访问。

### `Device` field access

- 前置条件：safe wrapper 不自行复制 `struct phy_device` bitfield 布局知识。
- 后置条件：状态读取优先依赖 bindgen 为当前布局生成的 accessor。
- safe 破坏面：如果继续在 safe wrapper 里手写 bit offset，safe API 会把错误
  的 `link` / `autoneg` / `autoneg_complete` 状态暴露给驱动逻辑，形成静默的
  语义漂移。

### `Registration`

- 前置条件：注册对象绑定的是已 pin 的 `'static` 驱动表。
- 线程迁移：移动 `Registration` 只会移动持有的句柄，不会移动底层 pinned
  driver table。
- 别名限制：`Registration` 不暴露内部驱动表引用；shared reference 也无法读写
  已注册表项。
- drop 配对：`register()` 成功后才会构造 `Registration`，`Drop` 必须且只能对同一
  表调用一次 `phy_drivers_unregister()`。
- 回调生命周期：驱动回调都由 phylib 通过 `'static` vtable 触发，`Registration`
  本身不提供跨回调状态保存通道。
- safe 破坏面：`Send/Sync` 证明只落在这个句柄上，而不是落在 `DriverVTable` 上。

### `DriverVTable`

- 前置条件：`create_phy_driver()` 产生的 `phy_driver` 初始化满足当前树 ABI。
- 后置条件：驱动表在注册期间保持固定地址，不被移动。
- safe 破坏面：benchmark 分支没有恢复 `unsafe impl Sync for DriverVTable`，
  避免把原始 vtable 容器误建模为可自由跨线程共享的对象。

## 当前仍保留的边界

- `create_phy_driver()` 对尾部零初始化的依赖仍受当前树 `struct phy_driver`
  ABI 影响。
- 本轮的可信 runtime 证据来自 synthetic MDIO + KUnit，而不是真实硬件 bring-up。
- 因此本轮可以宣称 `engineering-grade`，但不能越级宣称 `mainline-grade`。

## refreshed reference compare 归纳

- 参考树仍使用更宽的 `set_speed(u32)` safe API。
- 参考树仍把 `unsafe impl Sync` 放在 `DriverVTable` 上，并仅给
  `Registration` 一个 `unsafe impl Send`。
- 参考树仍手写 `phy_device` bit offset，而 benchmark 分支已经切换到
  bindgen accessor。
- benchmark 分支将上述三点都收紧或修正了，同时没有改变 callback table
  形状和驱动功能路径。
