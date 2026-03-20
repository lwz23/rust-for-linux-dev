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

本轮 hardening 之后，没有再保留“safe 驱动可以通过宽泛 setter 或 vtable 容器共享”
这样的直接破坏面。

## 重点不变式

### `Device` callback bridge

- 前置条件：`Device::from_raw()` 只在 phylib 交付的回调上下文中调用。
- 后置条件：`&mut Device` 只在回调作用域内短暂存在，不跨回调保存。
- safe 破坏面：驱动层拿不到 raw `phy_device *`，只能通过安全包装访问。

### `Registration`

- 前置条件：注册对象绑定的是已 pin 的 `'static` 驱动表。
- 后置条件：注册成功后，`Drop` 必须且只能对同一表调用一次
  `phy_drivers_unregister()`。
- safe 破坏面：`Registration` 不暴露内部驱动表引用；`Send/Sync` 证明只落在
  这个句柄上，而不是落在 `DriverVTable` 上。

### `DriverVTable`

- 前置条件：`create_phy_driver()` 产生的 `phy_driver` 初始化满足当前树 ABI。
- 后置条件：驱动表在注册期间保持固定地址，不被移动。
- safe 破坏面：benchmark 分支已移除 `unsafe impl Sync for DriverVTable`，避免把
  原始 vtable 容器误建模为可自由跨线程共享的对象。

## 当前仍保留的审计关注点

- `Registration` 的 `Send/Sync` 仍依赖结构性证明和 `Module: Send + Sync`
  的编译约束，而不是更强的自动类型推导。
- `create_phy_driver()` 对尾部零初始化的依赖仍受当前树 `struct phy_driver`
  ABI 影响。
- 本轮没有可信 runtime 证据，无法用真实 PHY 交互去补充这份静态审计。

## reference compare 归纳

- 参考树仍使用更宽的 `set_speed(u32)` safe API。
- 参考树仍把 `unsafe impl Sync` 放在 `DriverVTable` 上，并仅给
  `Registration` 一个 `unsafe impl Send`。
- benchmark 分支将这两点都收紧了，但没有改变 callback table 形状和
  驱动功能路径。
