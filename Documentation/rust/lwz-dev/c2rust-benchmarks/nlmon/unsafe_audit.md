# `nlmon` normalized unsafe audit

- 生成日期：2026-03-20
- 对应详细审计：`Documentation/rust/lwz-dev/unsafe-audit-2026-03-18-nlmon-rust-zh_CN.rst`

## 当前结论

`nlmon` 的驱动层仍保持零 `unsafe`。所有 `unsafe`、raw pointer、FFI bridge
和字段写入都被压缩在：

- `rust/kernel/net/skbuff.rs`
- `rust/kernel/net/netdevice.rs`
- `rust/kernel/net/rtnl.rs`
- `rust/helpers/net.c`

本轮 hardening 之后，当前没有再发现“safe 驱动不写 `unsafe` 也能直接破坏已注册对象不变式”
的明显路径。

## 重点不变式

### `SkBuff`

- 前置条件：回调交付的 `skb` 指针有效，且当前释放责任已经交给驱动。
- 后置条件：若未转交其他内核 API，则必须且只能释放一次。
- safe 破坏面：当前已移除重新把 raw pointer 以 safe 方式交还给驱动的出口。

### `NetDevice` / private data

- 前置条件：`netdev_priv()` 对应区域只初始化一次、析构一次。
- 后置条件：私有区内的 pinned registered state 不得再被 safe code 移动。
- safe 破坏面：宽泛 `&mut Private` 已移除，驱动只能通过 pinned access 操作私有区。

### `NetlinkTapHandle`

- 前置条件：注册动作只能绑定当前设备自身。
- 后置条件：已注册 tap 的存储位置保持稳定，析构与注销配对。
- safe 破坏面：`add/remove` 仅接受 `Pin<&mut Self>`，并且 `add` 只接受 current-device capability。

### `Registration<T>`

- 前置条件：注册对象在整个注册生命周期内保持 pinned 且有效。
- 后置条件：注销由 `PinnedDrop` 完成，注册对象与其底层存储一一配对。
- safe 破坏面：原型期可移动聚合对象已经收缩为 pinned opaque registration。

## 当前仍保留的审计关注点

- `Registration<T>` 的 `Send/Sync` 仍依赖结构性证明。
- vtable 的零尾初始化仍依赖当前树 ABI 约定。
- `rust/helpers/net.c` 仍需后续 helper 退役审计。

## 归档用途

这份 `md` 版不是替代详细 `rst` 审计，而是把当前 benchmark schema 需要的关键结论
压缩成 agent 和后续工具更容易复用的格式。
