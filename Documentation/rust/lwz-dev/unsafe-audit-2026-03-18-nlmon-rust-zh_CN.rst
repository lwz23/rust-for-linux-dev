.. SPDX-License-Identifier: GPL-2.0

2026-03-18 nlmon Rust 原型 ``unsafe`` 审计
==========================================

摘要
----

本文对 ``nlmon`` Rust 原型当前依赖的 ``unsafe`` 边界做一次研究级初审，
目标不是证明“当前已经安全”，而是明确：

- 每一个 ``unsafe`` 点到底依赖什么前置条件；
- 这些前置条件在调用后是否仍能保持必要的后置条件；
- 当前 safe API 是否把生命周期、初始化状态、别名约束、配置前提编码清楚；
- 哪些点只是“当前 ``nlmon_rust`` 恰好没踩中”，但仍不应继续作为宽泛 safe API 保留。

审计范围固定为：

- ``rust/kernel/net/skbuff.rs``
- ``rust/kernel/net/netdevice.rs``
- ``rust/kernel/net/rtnl.rs``
- ``rust/helpers/net.c``

审计结论总览
------------

当前实现可以被界定为：

- 驱动层 ``drivers/net/nlmon_rust.rs`` 为零 ``unsafe``；
- 抽象层已经把原始指针和 FFI 下沉，但仍有若干 safe API 过宽；
- 因此它是“分层方式正确的第一版原型”，还不是“已完成健全性论证的工程级抽象”。

本轮审计给出的风险分级如下：

高风险
~~~~~~

1. ``DeviceRef<T>`` 没有生命周期参数，且 ``private()`` 为公开 safe API。
   当前 safe 代码可以把 ``DeviceRef<T>`` 跨回调缓存，进而在设备释放后读取私有区，
   这一点必须收缩。
2. ``LStats`` 的 safe API 没有编码 ``NETDEV_PCPU_STAT_LSTATS`` 这一配置前提。
   这意味着对任意 ``DeviceRef<T>`` 调用 ``add/read`` 在类型层面都是允许的，但并不总是合法。
3. ``AttrTable::is_present()`` 没有上界信息，safe 调用者可以传入超界索引，导致越界访问。

中风险
~~~~~~

4. ``SetupContext::device_mut()`` 让 setup 阶段拿到了过宽的运行期设备能力。
5. ``NetlinkTapHandle::add()`` 接收任意 ``DeviceRef<T>`` 并缓存原始设备指针，
   当前行为依赖调用者“刚好传的是自身设备且后续严格成对 remove”。
6. ``Registration<T>`` 的 ``Send/Sync`` 断言目前只有注释，没有足够强的不变式证明。

低风险
~~~~~~

7. ``SkBuff`` 的所有权模型基本合理，但 ``into_raw()`` 作为 safe API 仍然过宽。
8. 三个 vtable 结构体通过 ``zeroed().assume_init()`` 构造，在当前内核语义下大概率成立，
   但需要把假设写得更明确。
9. ``rust/helpers/net.c`` 本身很薄，但它暴露的指针与统计 helper 仍然需要 Rust 侧
   把生命周期和配置状态编码清楚。

硬性判定标准
------------

本审计对每个 ``unsafe`` 点都按同一标准判断：

1. 前置条件：
   原始指针来源、对象是否初始化、对象是否仍然存活、是否具备唯一可变访问、是否处于正确锁上下文。
2. 后置条件：
   调用完成后是否仍保持所有权、析构配对、生命周期、别名、类型状态等不变式。
3. safe 破坏面：
   safe API 调用者是否可能在不写 ``unsafe`` 的情况下破坏这些前置条件或后置条件。
4. 当前结论：
   ``可接受``、``需要收缩``、``需要改造`` 三选一。

逐项审计
--------

1. ``skbuff.rs``: ``SkBuff`` 拥有型封装
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``SkBuff::from_raw()`` 第 26 行
- ``SkBuff::len()`` 第 33 行
- ``Drop for SkBuff`` 第 47 行

原始指针来源：

- 来自 ``ndo_start_xmit`` 回调传入的 ``struct sk_buff *``。

前置条件：

- 指针非空、指向有效 ``sk_buff``；
- 回调已把该 ``skb`` 的所有权转交给驱动；
- Rust 侧成为唯一负责释放该 ``skb`` 的一方。

后置条件：

- ``len()`` 只读访问头部字段，不改变所有权；
- ``Drop`` 必须且只能在 Rust 仍拥有该 ``skb`` 时调用一次 ``consume_skb()``；
- 如果通过 ``into_raw()`` 把指针转回 C，Rust 侧必须放弃释放职责。

生命周期与别名：

- 当前 ``SkBuff`` 自身不带显式生命周期，但作为按值对象进入驱动回调，
  生命周期由回调栈帧控制，整体模型基本合理。

失败回滚与析构配对：

- ``start_xmit()`` 返回前若未转移所有权，``Drop`` 自动释放，析构配对清晰。

safe 破坏面：

- ``into_raw()`` 是公开 safe API，但当前驱动层并没有安全地把该原始指针重新交回
  内核的配套接口；这会给后续 safe 驱动留下“拿走裸指针”的过宽出口。

当前结论：

- ``from_raw()/len()/Drop``：``可接受``
- ``into_raw()``：``需要收缩``

建议改造：

- 删除 ``into_raw()``，或把它降为 ``pub(crate) unsafe``，只允许在抽象层内部配套使用。

2. ``netdevice.rs``: ``DeviceRef<T>``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``DeviceRef::from_raw()`` 第 97 行
- ``DeviceRef::private_ptr()`` 第 111 行
- ``DeviceRef::private()`` 第 118 行

原始指针来源：

- 来自 ``NetDevice::device_ref()`` 或 ``rtnl`` 回调桥接。

前置条件：

- ``ptr`` 非空，且在整个使用期间都指向存活的 ``struct net_device``；
- 私有区确实按 ``T::Private`` 初始化；
- 读取私有区时不存在与其冲突的可变借用。

后置条件：

- 创建 ``DeviceRef<T>`` 后，不应允许它逃逸出当前回调可证明有效的生命周期；
- ``private()`` 返回的共享借用只能在私有区仍然初始化且对象仍存活时使用。

生命周期与别名：

- 当前 ``DeviceRef<T>`` 没有生命周期参数，且实现了 ``Copy``；
- safe 驱动可以把它缓存到私有状态、全局变量或异步对象中，超出回调期后继续使用；
- 一旦底层 ``net_device`` 已释放，就会形成 use-after-free 风险。

失败回滚与析构配对：

- 该类型本身不析构，但它允许 safe 代码长期保存失效句柄，这会破坏后续所有使用点的前提。

safe 破坏面：

- 非常高；调用者不需要写 ``unsafe`` 就能突破“仅在回调期有效”的隐藏约束。

当前结论：

- ``需要改造``

建议改造：

- 为 ``DeviceRef`` 引入显式生命周期，或完全不向驱动层暴露该类型；
- ``private()`` 至少降为 ``pub(crate)``，更理想的方向是只在受限上下文内提供只读私有区访问。

3. ``netdevice.rs``: ``NetDevice<T>`` 原始包装与私有区初始化
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``NetDevice::from_raw()`` 第 141 行
- ``NetDevice::as_mut_ref()`` 第 152 行
- ``NetDevice::private_ptr()`` 第 158 行
- ``NetDevice::init_private()`` 第 169 行
- ``NetDevice::drop_private()`` 第 175 行
- ``NetDevice::device_ref()`` 第 195 行
- ``NetDevice::private()`` 第 201 行
- ``NetDevice::private_mut()`` 第 208 行

原始指针来源：

- 来自 RTNL / netdev 回调传入的 ``struct net_device *``。

前置条件：

- 回调上下文确实提供该设备的唯一可变访问；
- ``setup`` 阶段的私有区尚未初始化；
- ``open/stop`` 等运行期回调的私有区已经完成初始化且尚未析构。

后置条件：

- ``from_raw()`` 返回的 ``&mut NetDevice<T>`` 不得逃逸出当前回调；
- ``init_private()`` 与 ``drop_private()`` 必须严格一一配对；
- ``private()/private_mut()`` 只能在私有区已初始化时使用。

生命周期与别名：

- ``&mut NetDevice<T>`` 本身带借用生命周期，单独看是可控的；
- 但 ``device_ref()`` 会从中派生出无生命周期的 ``DeviceRef<T>``，重新打开逃逸通道。

失败回滚与析构配对：

- 当前 ``setup_callback()`` 先 ``init_private()`` 再进入 ``T::setup()``；
- 由于 ``setup`` 返回 ``()``，没有显式失败回滚路径；
- 依赖 ``priv_destructor`` 在未来销毁时恰好调用 ``drop_private()``。

safe 破坏面：

- ``NetDevice<T>`` 自身问题不大，但 ``device_ref()`` 让它输出了过宽句柄；
- ``private()/private_mut()`` 作为运行期 API 基本合理，前提是禁止越界逃逸。

当前结论：

- ``from_raw()/as_mut_ref()/private_ptr()/private()/private_mut()``：``可接受，但依赖缩窄外围 API``
- ``device_ref()``：``需要改造``
- ``init_private()/drop_private()``：``可接受，但需要在文档中把配对关系写死``

建议改造：

- 阻断 ``device_ref()`` 直接向驱动层暴露长期句柄；
- 在回调桥接层中把“setup 已初始化、destructor 正好析构一次”的状态机写成明确注释；
- 如后续引入会失败的 setup 逻辑，需要补充失败回滚而不是继续依赖隐式销毁。

4. ``netdevice.rs``: ``SetupContext`` 字段写入接口
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``add_private_flags()`` 第 234 行
- ``set_lltx()`` 第 241 行

原始指针来源：

- 来自 ``self.dev.as_mut_ref()`` 返回的底层 ``struct net_device``。

前置条件：

- 当前确实处于 ``setup`` 阶段；
- 写入的是 C 版 ``nlmon_setup()`` 允许设置的字段；
- bitfield 布局与 bindgen 生成的访问器一致。

后置条件：

- 设备配置字段完成与 C 版一致的写入；
- 不破坏其他 bitfield 字段。

生命周期与别名：

- 由 ``&mut SetupContext`` 保证唯一可变访问，基本成立。

safe 破坏面：

- 真正的问题不在这些 ``unsafe`` 位访问本身，而在 ``device_mut()`` 第 223 行把
  ``&mut NetDevice<T>`` 整体暴露给了 setup 调用者；
- 这让 safe 驱动可以在 setup 阶段调用运行期能力，如 ``private_mut()``、
  ``device_ref()`` 或 ``NetlinkTapHandle::add()``，从而破坏阶段边界。

当前结论：

- bitfield 写入本身：``可接受``
- ``SetupContext::device_mut()`` 造成的能力泄漏：``需要改造``

建议改造：

- 移除 ``device_mut()``，让 ``SetupContext`` 只保留受限字段设置 API。

5. ``netdevice.rs``: ``NetlinkTapHandle``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``current_module_ptr()`` 第 284 行
- ``add()`` 第 304 行
- ``remove()`` 第 316 行
- ``Drop`` 第 326 行

原始指针来源：

- ``current_module_ptr()`` 来自 ``__this_module``；
- ``add()`` 把 ``DeviceRef<T>`` 内的 ``net_device *`` 缓存到 ``netlink_tap.dev``。

前置条件：

- ``add()`` 只能对尚未注册的 tap 调用一次；
- 传入设备在整个 tap 注册期间都必须存活；
- 当前 ``net_device`` 的生命周期必须覆盖 ``remove()`` 或 ``Drop``；
- ``remove()`` 只应对已经注册的 tap 调用。

后置条件：

- ``registered`` 标志要与内核中的注册状态保持一致；
- ``add`` 成功后，后续必须有一次且仅一次 ``remove`` 或 ``Drop`` 清理；
- ``remove`` 失败时不能错误地把本地状态改成“已移除”。

生命周期与别名：

- 当前类型不记录“该 tap 绑定的是哪个设备、绑定期持续到何时”；
- safe 驱动可以传入任意 ``DeviceRef<T>``，不要求必须是“当前设备自身”；
- 由于 ``DeviceRef<T>`` 本身无生命周期，这里的存活性证明也不完整。

失败回滚与析构配对：

- ``add()`` 只有在成功后才置 ``registered = true``，这一点是正确的；
- ``remove()`` 失败时保持 ``registered = true``，也便于后续重试；
- ``Drop`` 中忽略错误是合理的“best-effort”，但应明确这是析构兜底，不是常规路径。

safe 破坏面：

- 中高；当前 ``nlmon_rust`` 只把它用于“自己的设备”，因此实际路径可用，
  但抽象层没有把这个约束编码进去。

当前结论：

- ``需要改造``

建议改造：

- 让 ``add()`` 接受受限的“当前运行期设备上下文”而非任意 ``DeviceRef<T>``；
- 保持现有 ``registered`` 状态机，但把“绑定设备必须与私有状态同寿命”的假设写进 API。

6. ``netdevice.rs``: ``LinkStats64`` 与 ``LStats``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``LinkStats64::from_raw()`` 第 343 行
- ``LinkStats64::set_rx()`` 第 356 行
- ``LStats::add()`` 第 371 行
- ``LStats::read()`` 第 380 行

原始指针来源：

- ``LinkStats64`` 来自 ``ndo_get_stats64`` 回调；
- ``LStats`` 调用的是 C helper ``dev_lstats_add()`` 与 ``dev_lstats_read()``。

前置条件：

- ``storage`` 在整个回调期间可写；
- 设备已配置 ``pcpu_stat_type = NETDEV_PCPU_STAT_LSTATS``；
- 统计区已经由网络核心正确分配。

后置条件：

- ``set_rx()`` 只覆盖 ``rx_packets`` / ``rx_bytes``；
- ``add/read()`` 不应在未配置 ``LSTATS`` 的设备上被调用。

生命周期与别名：

- ``LinkStats64`` 的回调期生命周期基本合理；
- ``LStats`` 只接收 ``DeviceRef<T>``，没有把“设备已配置 LSTATS”编码进类型。

失败回滚与析构配对：

- 统计 helper 自身无析构问题，但调用前提属于设备配置状态不变式的一部分。

safe 破坏面：

- 很高；当前任何 safe 驱动只要拿到 ``DeviceRef<T>`` 就能调用 ``LStats::add/read``，
  即使它根本不是 LSTATS 设备。

当前结论：

- ``LinkStats64``：``可接受``
- ``LStats``：``需要改造``

建议改造：

- 把 ``LStats`` 与“已配置为 LSTATS 的设备”绑定起来，例如只允许在 setup 中建立能力标记，
  或者直接做成 ``nlmon`` 私有的更窄接口而不是通用 safe API。

7. ``rtnl.rs``: ``AttrTable`` 与 ``ExtAck``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``AttrTable::from_raw()`` 第 32 行
- ``AttrTable::is_present()`` 第 47 行
- ``ExtAck::from_raw()`` 第 61 行

原始指针来源：

- 来自 RTNL 核心传入的属性数组和 ``netlink_ext_ack``。

前置条件：

- 数组在整个 ``validate`` 回调期间有效；
- ``index`` 必须处于当前数组实际边界内；
- ``extack`` 若非空，则在回调期间可变有效。

后置条件：

- ``is_present()`` 只能读取有效槽位，不能越界；
- ``ExtAck`` 借用不得逃逸出回调。

生命周期与别名：

- ``AttrTable<'a>`` 有生命周期参数，但它没有记录长度；
- ``ExtAck`` 的生命周期由 ``Option<&mut ExtAck>`` 承载，相对更清楚。

safe 破坏面：

- ``AttrTable::is_present()`` 目前是明显的 safe 越界入口；
- ``ExtAck`` 当前尚未暴露可写方法，风险较低。

当前结论：

- ``AttrTable``：``需要改造``
- ``ExtAck``：``可接受``

建议改造：

- 为 ``AttrTable`` 增加最大索引信息，或改为只暴露经过枚举封装的合法属性查询。

8. ``rtnl.rs``: ``Registration<T>`` 的线程断言与静态 vtable 构造
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``unsafe impl Send`` 第 128 行
- ``unsafe impl Sync`` 第 130 行
- ``NET_DEVICE_OPS`` 第 139 行
- ``ETHTOOL_OPS`` 第 149 行
- ``VTABLE`` 第 162 行

原始指针来源：

- 静态 vtable 由 Rust 构造，随后注册进 RTNL 核心。

前置条件：

- 零初始化尾部字段在 C 语义上等价于“未设置回调/字段为 0”；
- ``Registration<T>`` 不会在共享引用下并发修改注册对象；
- ``UnsafeCell<rtnl_link_ops>`` 的可变访问只发生在构造和销毁时。

后置条件：

- 注册后的 vtable 存储在 ``KBox`` 中并在整个注册期间保持地址稳定；
- ``Send/Sync`` 不能让 safe 代码构造未同步的并发可变访问。

生命周期与别名：

- 当前 ``Registration<T>`` 不存储 ``T`` 的实例，只存储 vtable；
- 但 ``Send/Sync`` 的证明目前只停留在注释，没有明确把“构造后不再修改”的约束写透。

safe 破坏面：

- 中等；当前模块实例化方式较简单，短期不太会踩，但抽象要面向复用时证明不够。

当前结论：

- ``需要补强证明``

建议改造：

- 保留实现前，先把不变式写全；
- 如果无法给出严格证明，就不要维持过宽的 ``Send/Sync`` 断言。

9. ``rtnl.rs``: 注册、注销与回调桥接
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``rtnl_link_register()`` 第 170 行
- ``priv_destructor_callback()`` 第 175-180 行
- ``setup_callback()`` 第 183-193 行
- ``validate_callback()`` 第 196-207 行
- ``open_callback()`` 第 212-216 行
- ``stop_callback()`` 第 221-225 行
- ``start_xmit_callback()`` 第 230-240 行
- ``get_stats64_callback()`` 第 243-250 行
- ``get_link_callback()`` 第 253-256 行
- ``Drop::drop()`` 第 264 行

原始指针来源：

- 全部来自网络核心在相应回调时传入。

前置条件：

- 每个回调都只在对应 C 子系统约定的上下文中被调用；
- 设备指针、属性数组、统计存储、``skb`` 在回调期间有效；
- ``setup_callback()`` 对每个设备只初始化一次私有区；
- ``priv_destructor_callback()`` 对每个设备只析构一次私有区；
- ``Drop`` 只在注册成功后调用一次注销。

后置条件：

- ``setup`` 后私有区进入“已初始化”状态；
- ``priv_destructor`` 后私有区进入“已析构”状态；
- ``start_xmit`` 后 ``skb`` 所有权被消费，不重复释放；
- 注销发生时，底层 ``rtnl_link_ops`` 存储仍然有效。

生命周期与别名：

- 回调桥接本身的生命周期模型基本依附于内核 C 约定；
- 真正的薄弱点是它们把原始指针包装成了若干过宽 safe 类型，再交给驱动层。

失败回滚与析构配对：

- 注册失败时 ``new()`` 直接返回错误，不生成 ``Registration<T>``，这一点正确；
- 注册成功后的注销由 ``Drop`` 负责，路径清楚；
- 私有区初始化/析构依赖 ``setup_callback`` 与 ``priv_destructor_callback`` 一一配对，
  当前需要在文档中把这一假设写死。

safe 破坏面：

- 回调桥接本身问题不算最大，核心问题仍是下游 safe 类型过宽。

当前结论：

- ``可接受，但依赖缩窄下游 safe API``

建议改造：

- 先收缩 ``DeviceRef``、``AttrTable``、``LStats``、``SetupContext`` 等类型，再复核
  回调桥接是否还存在新的逃逸面。

10. ``rust/helpers/net.c``: C helper 边界
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及位置：

- ``rust_helper_netdev_priv()`` 第 5-8 行
- ``rust_helper_dev_lstats_add()`` 第 10-13 行

原始指针来源：

- 都来自 Rust 侧传入的 ``struct net_device *``。

前置条件：

- ``netdev_priv()``：``dev`` 必须指向仍然存活的 ``net_device``，其私有区布局与
  Rust 预期一致；
- ``dev_lstats_add()``：``dev`` 必须已配置为 ``NETDEV_PCPU_STAT_LSTATS``，
  且统计区已正确分配。

后置条件：

- helper 自身不持有指针，不延长生命周期；
- 它们只是把内核 inline 语义暴露给 Rust，真正的不变式仍需 Rust 抽象层维护。

safe 破坏面：

- helper C 自身很薄，但如果 Rust safe API 没编码前提，就会把这里的前提泄漏给调用者。

当前结论：

- ``可接受，但依赖 Rust 侧收缩 API``

优先级排序后的改造任务
----------------------

P0
~~

1. 收缩 ``DeviceRef<T>``，禁止无生命周期的长期逃逸。
2. 收缩 ``LStats``，把 ``LSTATS`` 配置前提编码进类型或更窄接口。
3. 收缩 ``AttrTable``，加入边界信息或改为枚举化查询。

P1
~~

4. 收缩 ``SetupContext``，移除 ``device_mut()`` 这类会泄漏运行期能力的接口。
5. 收缩 ``NetlinkTapHandle::add()``，让它只接受“当前设备上下文”而不是任意 ``DeviceRef<T>``。
6. 给 ``Registration<T>`` 的 ``Send/Sync`` 与注册生命周期补强证明，必要时撤销断言。

P2
~~

7. 收紧 ``SkBuff::into_raw()``。
8. 为三个零初始化 vtable 写出更严格的不变式说明。
9. 在后续强化差分测试中重点关注：

   - 重复 add/del 生命周期；
   - 统计路径在高频事件下的正确性；
   - tap 注册/移除配对；
   - 调试内核下是否出现 UAF、double-free、refcount、lockdep、RCU 异常。

当前阶段结论
------------

当前 ``nlmon`` Rust 原型的主要成绩是：驱动层已经实现零 ``unsafe``。

当前 ``nlmon`` Rust 原型的主要缺口是：抽象层中若干 safe API 仍然过宽，
还没有把前置条件与后置条件完全编码为类型和生命周期约束。

因此，下一阶段必须先做抽象层收缩与重构，再进入强化差分测试和调试内核验证，
不能把当前实现直接视为“已经证明内存安全”的最终版本。
