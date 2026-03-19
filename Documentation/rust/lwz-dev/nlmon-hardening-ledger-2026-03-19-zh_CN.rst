.. SPDX-License-Identifier: GPL-2.0

2026-03-19 nlmon hardening ledger
=================================

摘要
----

本文档记录 ``nlmon`` 在 ``rnull`` 复盘结论回灌之后，按
``soundness`` 与功能等价优先所做的第二轮 hardening。

本轮目标不是“把 diff 尽量做小”，而是把此前仍带原型期痕迹的
``rust/kernel/net/*`` 对象模型收缩成更接近主线 Rust-for-Linux 的形状，
尤其要堵住以下两类问题：

- 已注册对象在 safe API 下仍可被移动；
- 驱动私有区里一旦出现 registered / intrusive / callback-owned pinned 对象，
  safe 驱动仍然能拿到过宽的 ``&mut Private``。

最终结论是：

- ``nlmon`` 的业务行为仍与 ``drivers/net/nlmon.c`` 对齐；
- ``NetlinkTapHandle`` / ``Registration<T>`` / ``NlmonPrivate`` 已改成 pinned object；
- safe 驱动不再能通过宽泛 ``&mut Private`` 破坏“注册后不可移动”的事实；
- 本轮强制回灌出一条新的通用硬规则：

  - 只要驱动私有区可能包含 registered / intrusive / callback-owned pinned 对象，
    就不能再向 safe 驱动暴露宽泛 ``&mut Private``；
  - 必须改成 ``in-place pinned init + pinned access``。

变更分类账
----------

1. ``netdevice.rs``：私有区访问模型改成 pinned-private
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

本轮之前的问题：

- ``NetDevice<T>`` 通过普通 ``&mut self`` 暴露 ``private_mut()``；
- safe 驱动虽然不写 ``unsafe``，仍可把已注册对象从私有区里 safe 地挪来挪去；
- 文档和约定都知道“注册后不可移动”，但类型边界本身并没有把误用堵死。

本轮落地后的形状：

- ``NetDevice::from_raw()`` 改为返回 ``Pin<&mut NetDevice<T>>``；
- ``init_private()`` 改为接受 ``impl PinInit<T::Private>``，在 ``netdev_priv()``
  的稳定内存上做原位 pinned 初始化；
- ``private_mut()`` 被移除；
- ``with_private()`` 改为只给驱动：

  - ``Pin<&mut T::Private>``
  - ``CurrentDevice<'_, T>``

- 只保留共享只读访问 ``private(self: Pin<&Self>) -> &T::Private``。

归类：

- ``安全修复``
- ``主线式对象模型收缩``

2. ``NetlinkTapHandle``：从可移动 bindgen struct 收缩成 pinned registered object
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

本轮之前的问题：

- ``NetlinkTapHandle`` 直接持有可移动的 ``bindings::netlink_tap``；
- ``add()/remove()`` 接受普通 ``&mut self``；
- ``add()`` 还能接受任意 ``DeviceRef``，没有把“只能绑定当前设备自身”写进类型边界。

本轮落地后的形状：

- ``NetlinkTapHandle`` 改成 ``#[pin_data(PinnedDrop)]``；
- ``inner`` 改成 ``Opaque<bindings::netlink_tap>`` pinned 字段；
- ``new()`` 提供 pinned init；
- ``add()/remove()`` 只接受 ``Pin<&mut Self>``；
- ``add()`` 只接受 ``CurrentDevice<'_, T>``；
- best-effort 清理由 ``PinnedDrop`` 完成。

结果：

- safe 驱动不能再把已注册 tap 从私有区里移动出去；
- safe 驱动也不能再把 tap 绑定到“任意别的设备”。

归类：

- ``安全修复``
- ``功能约束编码到类型系统``

3. ``rtnl.rs``：注册对象改成主线式 pinned opaque registration
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

本轮之前的问题：

- ``Registration<T>`` 仍是 ``KBox<UnsafeCell<bindings::rtnl_link_ops>>``；
- 它更像 blind-first 阶段的原型聚合器，而不是主线常见的
  ``#[pin_data] + Opaque + PinnedDrop`` 注册对象。

本轮落地后的形状：

- ``Registration<T>`` 改成 ``#[pin_data(PinnedDrop)]``；
- ``ops`` 改成 pinned ``Opaque<bindings::rtnl_link_ops>``；
- ``new()`` 改为返回 ``impl PinInit<Self, Error>``；
- 模块侧通过 ``Pin<KBox<Registration<T>>>`` 持有注册对象；
- ``PinnedDrop`` 中完成 ``rtnl_link_unregister()``。

仍然保留但已写明审计理由的点：

- ``unsafe impl Send/Sync`` 仍存在，但其结构性理由已经与当前树成熟
  ``driver::Registration`` 模式对齐，并在代码与审计文档中写清；
- vtable 尾部仍使用 ``zeroed().assume_init()``，这是当前树成熟模式一致的 ABI 约定，
  不是已被类型系统完全消除的风险。

归类：

- ``安全修复``
- ``mainline-style refactor``

4. ``drivers/net/nlmon_rust.rs``：驱动私有数据改成 pinned private object
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

本轮之前的问题：

- ``NlmonPrivate`` 只是 ``derive(Default)``；
- ``tap`` 字段没有被标成 pinned 字段；
- ``open()/stop()`` 仍依赖原型期私有区访问模型。

本轮落地后的形状：

- ``NlmonPrivate`` 改成 ``#[pin_data]``；
- ``tap`` 改成 pinned 字段；
- ``private_init()`` 显式返回 ``NlmonPrivate::new()``；
- ``open()/stop()`` 通过 pinned projection 操作 ``tap``。

归类：

- ``安全修复``
- ``与主线已存在 Rust 模块风格对齐``

5. 明确认定的“非功能差异”与“旧树约束”
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

本轮同时把两类容易被误判的事项写死为审计结论：

``dev_kfree_skb()`` vs ``consume_skb()``
  当前树 ``include/linux/skbuff.h`` 明确把 ``dev_kfree_skb(a)`` 宏定义为
  ``consume_skb(a)``，因此 Rust 侧继续走 ``consume_skb()`` 不构成功能差异。

``rust/helpers/net.c`` 仍保留
  本轮没有直接删除 helper。原因不是“舍不得旧实现”，而是当前树生成的 helper 绑定
  仍然实际承接 ``netdev_priv()`` / ``dev_lstats_add()`` 的调用入口。
  这应被归类为 ``旧树约束``，后续单独做 helper 退役审计，而不是在本轮误删。

验证记录
--------

构建
~~~~

本轮先完成常规构建：

- ``/home/lwz/rfl-dev/scripts/build-kernel.sh``

随后按测试计划重建三套 debug build：

- ``/home/lwz/rfl-dev/build-nlmon-kasan``
- ``/home/lwz/rfl-dev/build-nlmon-lockdep``
- ``/home/lwz/rfl-dev/build-nlmon-kmem``

并确认：

- 无新增 ``git diff --check`` 问题；
- ``drivers/net/nlmon_rust.rs`` 仍为零 ``unsafe``。

运行时
~~~~~~

本轮按计划重跑 Rust 场景：

- ``memory-debug``：

  - ``baseline``
  - ``lifecycle``
  - ``matrix``

- ``concurrency-debug``：

  - ``baseline``
  - ``lifecycle``

- ``leak-debug``：

  - ``baseline``
  - ``lifecycle``

所有 Rust 侧重跑日志都给出：

- ``status=ok``
- ``dmesg_anomaly.*=0``

关键摘要如下：

``2026-03-19 memory-debug rust baseline``
  ``38244`` bytes / ``2451`` decoded lines / ``154`` captured

``2026-03-19 memory-debug rust matrix``
  ``dummy=600424/38300/1900``
  ``veth=492824/31600/1900``
  ``bridge=542824/34800/2300``
  ``route_rule=132824/8650/800``
  ``netns=142224/9200/800``

``2026-03-19 concurrency-debug rust baseline``
  ``38244`` bytes / ``2451`` decoded lines / ``154`` captured

``2026-03-19 leak-debug rust baseline``
  ``38244`` bytes / ``2451`` decoded lines / ``154`` captured

由于 ``2026-03-19`` 的 ``memory-debug rust baseline`` 相比 ``2026-03-18`` 历史日志
出现了摘要变化，本轮按计划补跑 C 对照：

- ``2026-03-19 memory-debug c baseline``
- ``2026-03-19 memory-debug c matrix``

补跑结果表明：

- ``2026-03-19 memory-debug c baseline`` 也同样变为
  ``38244`` bytes / ``2451`` decoded lines / ``154`` captured；
- ``2026-03-19 memory-debug c matrix`` 与同日 Rust ``matrix`` 的
  ``pcap.bytes`` / ``decoded.lines`` / captured 计数逐项对齐。

因此本轮对 baseline 摘要变化的分类是：

- ``不是 Rust 特有功能差异``
- ``更像同日运行环境/调试配置下的共同漂移``

新回灌的硬规则
--------------

本轮确认应升级为共享手册硬规则的一条结论是：

- 只要驱动私有区可能包含 registered / intrusive / callback-owned pinned 对象，
  safe API 就不能再公开宽泛 ``&mut Private``；
- 必须改成：

  - ``in-place pinned init``
  - ``pinned private access``
  - 如有必要，再配合 current-device capability

这条规则是从 ``rnull`` 复盘得到的方向性结论，在 ``nlmon`` hardening 中被再次坐实。

剩余保留项
----------

本轮仍有三类保留项，但它们都已经从“明显 safe API 过宽洞”降级为“后续继续工程化”的问题：

1. ``Registration<T>`` 的 ``Send/Sync`` 仍依赖结构性证明，而不是完全由类型系统自动导出。
2. vtable 零尾初始化仍依赖当前内核 ABI 对零值尾部的解释。
3. ``rust/helpers/net.c`` 仍需后续单独做 helper 退役审计。

当前结论
--------

对 ``nlmon`` 当前 hardening 结果，更准确的结论是：

- 已消除本轮重点关注的 ``soundness`` 风险面；
- 已把“注册后不可移动”的关键事实编码进 safe API 边界；
- 当前未发现与 C 金标准之间未分类的功能差异；
- 已经比 ``2026-03-18`` 的 blind-first 版本更接近主线 Rust-for-Linux 的成熟对象模型。
