.. SPDX-License-Identifier: GPL-2.0

2026-03-18 nlmon Rust 抽象层 ``unsafe`` 审计（2026-03-19 hardening 刷新版）
==========================================================================

摘要
----

本文档在 ``rust: tighten nlmon net abstraction lifetimes`` 之后首次生成，
并在 ``2026-03-19`` 的 ``nlmon`` hardening 完成后再次刷新。它的目的不是夸大为
“已经被形式化证明安全”，而是把现在这棵树上真正存在的事实说清楚：

- ``drivers/net/nlmon_rust.rs`` 仍保持零 ``unsafe``；
- 所有 ``unsafe``、裸指针、FFI 回调桥接、结构体字段写入都被限制在
  ``rust/kernel/net/*`` 与极小的 ``rust/helpers/net.c``；
- 早期审计里指出的几处明显 safe API 过宽问题，已经在当前代码中被收紧；
- safe API 已不再允许驱动通过宽泛 ``&mut Private`` 移动已注册 ``netlink_tap``；
- 目前没有再发现“驱动层不写 ``unsafe`` 也能轻易破坏抽象不变式”的明显洞；
- 但这并不等于抽象已经可以直接作为通用 ``netdev/rtnl`` 子系统抽象上游化，
  仍存在几处需要继续收紧或继续积累证据的点。

当前总体定性为：

- 对 ``nlmon`` 当前单驱动用例而言，这套抽象边界已经达到“工程上可接受”的水平；
- 对“可推广到更多 Rust 网络驱动的通用抽象”而言，仍建议继续收紧能力边界。

为什么要刷新
------------

此前的 ``unsafe`` 审计文档生成于抽象层第一轮收缩之前，当时确实存在以下问题：

- ``DeviceRef`` 没有生命周期参数；
- ``DeviceRef`` 暴露了私有区读取能力；
- ``SetupContext`` 能把运行期设备能力泄漏给 setup 阶段；
- ``SkBuff`` 还保留过宽的 ``into_raw()`` 出口；
- ``AttrTable`` 缺少足够直接的边界检查。

这些问题在当前源码中已经发生变化，因此旧审计结论如果继续沿用，会直接和代码现实脱节。

审计范围
--------

本轮范围固定为：

- ``rust/kernel/net/skbuff.rs``
- ``rust/kernel/net/netdevice.rs``
- ``rust/kernel/net/rtnl.rs``
- ``rust/helpers/net.c``

不覆盖：

- 其他 Rust-for-Linux 子系统抽象
- ``drivers/net/nlmon_rust.rs`` 之外的驱动使用方
- 形式化证明

审计口径
--------

每个 ``unsafe`` 点都按同一组问题检查：

1. 原始指针或 FFI 对象从哪里来；
2. 调用前必须满足哪些前置条件；
3. 调用后必须继续成立哪些后置条件；
4. 所有权、生命周期、别名/可变性边界是否被编码；
5. safe API 调用者能否在不写 ``unsafe`` 的前提下破坏这些条件；
6. 当前应判为：

   - ``可接受``
   - ``可接受，但保留后续优化项``
   - ``需要继续改造``

结论总览
--------

当前已经解决的旧问题
~~~~~~~~~~~~~~~~~~~~

1. ``DeviceRef`` 现在带显式生命周期 ``'a``，且只保留了回调期共享设备能力。
2. ``DeviceRef`` 不再向驱动层暴露私有区读取接口。
3. ``SetupContext`` 已收缩为 setup 阶段专用 setter 集合，不再泄漏运行期能力。
4. ``SkBuff`` 已移除 ``into_raw()`` 这类会把所有权模型重新打穿的 safe 出口。
5. ``AttrTable::is_present()`` 已先做空指针和索引上界判断，再触及底层数组。

当前可以接受的核心结论
~~~~~~~~~~~~~~~~~~~~~~

1. 驱动层零 ``unsafe`` 这一硬约束已经成立。
2. 当前 ``nlmon`` 所需的 ``unsafe`` 已被收敛到少量可审计的抽象点。
3. 这些抽象点现在基本都带有局部不变式注释，前置条件和后置条件比第一版清晰得多。
4. 结合调试内核差分结果，目前没有证据表明这套抽象在 ``nlmon`` 当前路径上触发了
   明显的 UAF、double-free、RCU、lockdep 或 ``kmemleak`` 异常。

当前仍需保留的注意点
~~~~~~~~~~~~~~~~~~~~

1. ``Registration<T>`` 的 ``Send/Sync`` 仍然依赖内核注册对象生命周期契约，而不是更强的
   类型约束。
2. 三个 vtable 仍使用 ``MaybeUninit::zeroed().assume_init()`` 构造尾部，这一写法与现有
   Rust-for-Linux 风格一致，但本质上仍依赖 C 侧“未填字段全 0 即 None/默认值”的 ABI 约定。
3. ``rust/helpers/net.c`` 仍然是当前树这两个入口的实际绑定来源，应在后续单独做 helper
   退役审计，但不应在本轮 hardening 里误删。

逐项审计
--------

1. ``skbuff.rs``: ``SkBuff`` 拥有型封装
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及点：

- ``SkBuff::from_raw()``
- ``SkBuff::len()``
- ``Drop for SkBuff``

原始指针来源：

- ``ndo_start_xmit`` 回调提供的 ``struct sk_buff *``。

前置条件：

- 指针非空且有效；
- 当前回调语义已经把该 ``skb`` 的释放责任交给驱动；
- Rust 侧在本次回调内拥有唯一释放职责。

后置条件：

- ``len()`` 只做只读字段访问；
- 若没有把 ``skb`` 交还给其他内核 API，则离开作用域时必须且只能释放一次；
- ``Drop`` 调用 ``consume_skb()`` 后，Rust 侧不再持有此对象。

safe 破坏面：

- 当前 ``SkBuff`` 不再提供把原始指针重新以 safe 方式拿出的接口；
- 因此驱动层很难在不写 ``unsafe`` 的情况下破坏其所有权模型。

结论：

- ``可接受``

2. ``netdevice.rs``: ``DeviceRef`` / ``NetDevice`` / ``SetupContext``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``DeviceRef<'a, T>``
^^^^^^^^^^^^^^^^^^^^

当前形态：

- 带显式生命周期参数；
- 只暴露 ``as_raw()``（crate 内）和 ``lstats()`` 这类受限能力；
- 不再向驱动层暴露私有区访问。

前置条件：

- ``from_raw()`` 只能从内核回调传入的有效 ``net_device *`` 构造；
- 生命周期 ``'a`` 必须被限制在回调期内。

后置条件：

- 只读共享设备能力不会跨越回调边界泄漏为长期句柄；
- ``lstats()`` 只在 ``pcpu_stat_type == LSTATS`` 时返回 capability。

safe 破坏面：

- 早期“可跨回调缓存设备句柄”的大洞已经被生命周期参数堵住；
- 当前驱动层如果想 misuse，需要绕过类型系统或重新引入 ``unsafe``。

结论：

- ``可接受``

``NetDevice<T>``
^^^^^^^^^^^^^^^^

当前形态：

- 仅在 ``setup/open/stop`` 等串行化回调中，通过 ``from_raw()`` 得到短生命周期
  ``Pin<&mut NetDevice<T>>``；
- ``init_private()`` 在 ``netdev_priv()`` 的稳定内存上执行 ``PinInit<T::Private>``；
- ``with_private()`` 把 ``Pin<&mut T::Private>`` 与 ``CurrentDevice<'_, T>`` 一并交给闭包；
- ``private_mut()`` 已被移除，只保留共享只读 ``private()``。

前置条件：

- 回调由网络核心提供，且当前上下文确实允许唯一可变访问；
- ``init_private()`` 与 ``drop_private()`` 严格一一配对；
- 私有区按 ``T::Private`` 初始化一次、析构一次。

后置条件：

- ``setup`` 完成后私有区进入已初始化状态；
- ``priv_destructor`` 路径负责与之匹配的析构；
- ``with_private()`` 不会再向 safe 驱动暴露可移动的已注册私有子对象。

safe 破坏面：

- safe 驱动不再能通过普通 ``&mut Private`` 把已注册 tap safe 地移出私有区；
- 仍需依赖注册/销毁回调配对这一内核契约，但这已经属于抽象内部受控边界。

结论：

- ``可接受``

``SetupContext<'a, T>``
^^^^^^^^^^^^^^^^^^^^^^^

当前形态：

- 只保留 ``set_type()``、``add_private_flags()``、``set_lltx()``、``set_features()``、
  ``set_flags()``、``enable_lstats()``、``set_mtu()``、``set_min_mtu()``。

前置条件：

- 只能在 ``setup`` 回调期间使用；
- bitfield 写入依赖 bindgen 生成字段与当前内核布局一致。

后置条件：

- setup 阶段完成 C 版 ``nlmon_setup()`` 等价字段配置；
- 不会把运行期设备访问能力带给驱动。

safe 破坏面：

- 早期 ``device_mut()`` 已被移除；
- 当前 safe 调用者不能再在 setup 阶段偷偷进入运行期私有数据或 tap 注册逻辑。

结论：

- ``可接受``

``LStatsHandle<'a, T>``
^^^^^^^^^^^^^^^^^^^^^^^

当前形态：

- 只能从 ``DeviceRef::lstats()`` 获得；
- 返回 ``Option``，把 ``NETDEV_PCPU_STAT_LSTATS`` 这一配置前提编码成能力存在与否。

前置条件：

- 设备在 setup 阶段已经启用了 ``enable_lstats()``；
- 当前设备在回调期内仍存活。

后置条件：

- ``add()`` / ``read()`` 只对当前设备统计区生效；
- 统计读写不会突破设备生命周期。

safe 破坏面：

- 相比旧版本“任何设备都能直接调统计 helper”，当前已经明显收紧；
- ``nlmon_rust`` 中用 ``expect()`` 表达“本驱动 setup 必定启用 LSTATS”，属于业务逻辑断言，
  不是新的内存安全洞。

结论：

- ``可接受``

3. ``netdevice.rs``: ``NetlinkTapHandle``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

涉及点：

- ``current_module_ptr()``
- ``add()``
- ``remove()``
- ``PinnedDrop``

原始指针来源：

- ``__this_module``、``CurrentDevice`` 内的 ``net_device *``，以及 pinned
  ``Opaque<bindings::netlink_tap>`` 存储。

前置条件：

- ``add()`` 只能对未注册状态调用；
- 传入设备在 tap 注册期间必须持续存活；
- tap 自身在注册期间必须保持 pinned；
- ``remove()`` 只对已注册状态生效。

后置条件：

- ``registered`` 必须与内核注册状态同步；
- ``add`` 成功后，必须恰好有一次 ``remove`` 或 ``Drop`` 做回收；
- ``remove`` 失败时不能误把本地状态写成“已移除”。

当前优点：

- ``inner`` 已收缩成 pinned ``Opaque<bindings::netlink_tap>``；
- 已显式维护 ``registered`` 状态机；
- ``add()`` 现在只接受 ``CurrentDevice``，已经把“只能绑定当前设备自身”编码进类型边界；
- ``PinnedDrop`` 只在已注册时做 best-effort 卸载；
- ``nlmon_rust`` 的实际使用路径是：

  - ``open()`` 中把自身设备注册为 tap
  - ``stop()`` 中移除
  - 若设备销毁前仍处于注册状态，则 ``PinnedDrop`` 兜底清理

结论：

- ``可接受``

4. ``rtnl.rs``: ``AttrTable`` / ``ExtAck`` / ``Registration<T>``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``AttrTable<'a>``
^^^^^^^^^^^^^^^^^

当前形态：

- ``from_raw()`` 只在回调桥接内部使用；
- ``is_present()`` 先检查 ``ptr.is_null()`` 和 ``index > max_index``，再做底层访问。

结论：

- ``可接受``

``ExtAck``
^^^^^^^^^^

当前形态：

- ``from_raw()`` 把可空 ``netlink_ext_ack *`` 转成 ``Option<&mut ExtAck>``；
- 生命周期被限制在回调期。

结论：

- ``可接受``

``Registration<T>``
^^^^^^^^^^^^^^^^^^^

当前形态：

- ``new()`` 通过 pinned ``Opaque<bindings::rtnl_link_ops>`` 完成原位初始化与注册；
- ``PinnedDrop`` 负责注销；
- ``setup/open/stop/start_xmit/get_stats64/get_link`` 都在桥接层内部把原始回调参数包装成
  窄化后的 Rust 对象。

``Send/Sync`` 断言：

- 当前已经补充了更明确的局部不变式说明：

  - ``Registration<T>`` 仅持有 pinned ``Opaque<rtnl_link_ops>`` 和 ``PhantomData<T>``
  - 真正的回调状态由网络核心管理
  - 注册成功后不再通过共享引用去并发修改表项

vtable 构造：

- 仍依赖 ``zeroed().assume_init()`` 构造未使用回调槽位；
- 这一点与现有 Rust-for-Linux 其他 vtable 抽象风格一致，但仍需在代码审阅时显式意识到
  它依赖 C ABI 对零初始化尾部的解释。

safe 破坏面：

- 当前驱动层无法直接接触原始 ``rtnl_link_ops`` 或 ``net_device_ops``；
- 回调桥接已把大多数前置条件封在抽象内部。

结论：

- ``可接受，但保留后续优化项``

5. ``rust/helpers/net.c``
~~~~~~~~~~~~~~~~~~~~~~~~~

当前 helper 只做两件事：

- ``rust_helper_netdev_priv()``
- ``rust_helper_dev_lstats_add()``

特点：

- 不引入新的 C 结构；
- 不持有状态；
- 不承载驱动业务逻辑；
- 只为 bindgen 无法直接表达的 C 入口补一个最薄包装。

结论：

- ``可接受``

最终结论
--------

基于当前源码、更新后的抽象边界，以及本轮调试内核差分结果，可以得出比旧审计更准确的结论：

1. 当前 ``nlmon_rust`` 驱动层零 ``unsafe`` 的目标已经稳定成立。
2. 早期那几处明显的 safe API 过宽问题，已经在当前代码中被实质性修正。
3. 目前没有再观察到“safe 驱动调用者不写 ``unsafe`` 也能轻易打穿抽象边界”的明显路径。
4. 当前剩余的主要问题，不再是立刻可见的内存安全洞，而是抽象的通用化边界是否还要继续缩窄。

因此本轮建议的工程定性是：

- 对 ``nlmon`` 当前分支内的实验目标：``可进入工程验收汇总``；
- 对“作为更通用 ``netdev/rtnl`` Rust 抽象直接长期推广”：``仍建议继续收紧并扩大用例``。
