.. SPDX-License-Identifier: GPL-2.0

2026-03-18 工作日志：nlmon Rust 化启动
======================================

日期
----

- 日期：2026-03-18
- 记录人：``lwz23``
- 主题：启动 ``drivers/net/nlmon.c`` 的 Rust 化工作

今日目标
--------

今天的目标不是一次性完成 ``nlmon`` 的 Rust 化，而是按可回滚、可验证的方式，
把这项工作拆成若干个可以独立提交的小阶段。

硬性约束如下：

- 永远不删除原始 ``drivers/net/nlmon.c``。
- 最终 Rust 驱动层不允许出现任何 ``unsafe``。
- 所有 ``unsafe`` 必须封装在 ``rust/kernel/`` 下的抽象层中。
- Rust 与 C 版本必须支持可切换构建。
- 差分测试以原 C 版 ``nlmon`` 的行为为金标准。

当前起点
--------

开始本轮工作前，已确认以下事实：

- 主开发基线为 ``/home/lwz/rfl-dev/linux``。
- 当前本地长期开发分支为 ``dev``，远端跟踪 ``origin/dev``。
- QEMU 启动链路已经验证成功。
- out-of-tree Rust 模块已经验证成功。
- in-tree Rust sample ``rust_minimal`` 已经验证成功。
- 原 C 版 ``nlmon`` 在虚拟机中的基础功能此前已经验证通过。

本轮执行策略
------------

本轮将遵循下面的实施顺序：

1. 先建立功能分支和工作日志。
2. 先补 Kconfig/Makefile 切换开关，保证 C/Rust 实现可以二选一构建。
3. 先扩展 ``rust/bindings/bindings_helper.h``，通过主 bindgen 暴露 ``nlmon`` 所需 C 类型与接口。
4. 对生成结果做缺口审计，确认哪些能力已由主 bindings 覆盖，哪些能力仍需额外处理。
5. 仅对 ``bindgen`` 无法直接处理的 inline 函数和复杂宏补最小 ``rust/helpers`` 包装。
6. 在 ``rust/kernel/net`` 下建立安全抽象层，把 ``unsafe`` 与 FFI 细节收敛到抽象内部。
7. 在抽象之上实现 ``drivers/net/nlmon_rust.rs``，保证驱动层零 ``unsafe``。
8. 最后执行构建、启动、差分测试，并根据结果继续迭代。

当前仓库基线（修正后）
----------------------

在修正本文档顺序时，再次确认当前仓库状态如下：

- 当前工作分支为 ``feature/nlmon-rust``。
- 当前 ``HEAD`` 为提交 ``60689628b``，主题为 ``Documentation: rust: add 2026-03-18 nlmon work log``。
- 目前除本文档与索引入口外，还没有任何 ``bindings/helper/抽象/驱动`` 代码提交落地。
- 从这一点开始，后续所有改动都遵循“单阶段、单提交、同步更新工作日志”的规则。

阶段记录
--------

1. 初始化阶段
~~~~~~~~~~~~~

- 从 ``dev`` 切出功能分支 ``feature/nlmon-rust``。
- 建立本日志文件，作为本轮所有提交的同步记录入口。

2. 流程顺序修正
~~~~~~~~~~~~~~~

- 重新核对 ``Documentation/rust/general-information.rst`` 与 ``rust/Makefile`` 中的实际机制。
- 确认本仓库必须遵循 ``Kbuild -> 主 bindgen -> 缺口审计 -> helper -> 抽象 -> 驱动 -> 差分测试`` 的顺序。
- 确认 ``rust/helpers`` 的绑定生成规则只允许导出 ``rust_helper_*`` 函数，不能承载新的 C 类型定义，因此 helper 不能替代主 bindings 阶段。

3. Kbuild 切换开关
~~~~~~~~~~~~~~~~~~

- 在 ``drivers/net/Kconfig`` 中新增 ``config NLMON_RUST``，作为 Rust 实现选择器。
- 在 ``drivers/net/Makefile`` 中参考 ``AX88796B_RUST_PHY`` 模式，按 ``CONFIG_NLMON_RUST`` 在 ``nlmon.o`` 与 ``nlmon_rust.o`` 之间二选一。
- 这一阶段只建立构建切换入口，不引入 bindings/helper/抽象/驱动代码。

4. 主 bindings 暴露与缺口审计
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在 ``rust/bindings/bindings_helper.h`` 中补入：

  - ``<linux/if_arp.h>``
  - ``<linux/netdevice.h>``
  - ``<linux/netlink.h>``
  - ``<net/rtnetlink.h>``

- 在 ``rust/bindgen_parameters`` 中为 ``NLMSG_GOODSIZE``、``NETIF_F_SG``、
  ``NETIF_F_FRAGLIST``、``NETIF_F_HIGHDMA`` 补入 ``blocklist-item``，并在
  ``bindings_helper.h`` 中增加对应 ``RUST_CONST_HELPER_*`` 常量，确保这些
  宏值能稳定进入主 bindings。
- 先尝试用单目标 ``make ... rust/bindings/bindings_generated.rs`` 做增量刷新，
  发现旧产物未自动失效；随后改用既有脚本
  ``/home/lwz/rfl-dev/scripts/build-kernel.sh`` 触发完整增量构建，确认主
  ``bindgen`` 实际重新执行。
- 当前主 bindings 已确认覆盖以下 ``nlmon`` 依赖：

  - ``struct net_device``
  - ``struct net_device_ops``
  - ``struct rtnl_link_ops``
  - ``struct netlink_tap``
  - ``rtnl_link_register`` / ``rtnl_link_unregister``
  - ``netlink_add_tap`` / ``netlink_remove_tap``
  - ``dev_lstats_read``
  - ``ARPHRD_NETLINK``
  - ``IFF_NO_QUEUE``
  - ``IFF_NOARP``
  - ``NETDEV_PCPU_STAT_LSTATS``
  - ``NETDEV_TX_OK``
  - ``NLMSG_GOODSIZE``
  - ``NETIF_F_SG`` / ``NETIF_F_FRAGLIST`` / ``NETIF_F_HIGHDMA``

- 本轮审计后仍需要通过 helper 解决的剩余盲区为：

  - ``netdev_priv``（inline）
  - ``dev_lstats_add``（inline）

- ``dev_kfree_skb`` 暂不列入 helper 缺口，因为当前 bindings 已直接提供
  ``consume_skb``，后续在抽象层再决定是否需要额外兼容包装。

后续阶段会继续在本文件中追加记录。
