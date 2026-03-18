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

5. 最小 helper 补齐
~~~~~~~~~~~~~~~~~~

- 新增 ``rust/helpers/net.c``，只为本轮审计中确认的两个 inline 盲区提供包装：

  - ``rust_helper_netdev_priv()``
  - ``rust_helper_dev_lstats_add()``

- 在 ``rust/helpers/helpers.c`` 中按字母序纳入 ``net.c``。
- 本阶段不定义新的 C 结构体，不承载设备业务逻辑，只负责为后续
  ``rust/kernel/net`` 抽象提供可绑定的最小 FFI 入口。

6. net 基础抽象
~~~~~~~~~~~~~~~

- 新增 ``rust/kernel/net/skbuff.rs``，引入拥有型 ``SkBuff``，其 ``Drop`` 使用
  已存在主 bindings 中的 ``consume_skb()`` 完成释放语义。
- 新增 ``rust/kernel/net/netdevice.rs``，先只落地本阶段所需的最小安全对象：

  - ``DeviceRef``：共享 ``net_device`` 引用包装，供基础对象传递设备句柄
  - ``NetlinkTapHandle``：封装 ``netlink_add_tap()`` / ``netlink_remove_tap()``
  - ``LinkStats64``：封装 ``struct rtnl_link_stats64`` 的接收统计写入
  - ``LStats``：封装 ``dev_lstats_add()`` / ``dev_lstats_read()``
  - ``hardware / features / device_flags / priv_flags / stat_type / netlink / link_attrs`` 常量接口

- 更新 ``rust/kernel/net.rs``，导出上述基础抽象，同时保持与现有 ``net`` 模块布局一致。
- 本阶段刻意不引入 ``rtnl_link_ops`` / ``net_device_ops`` / ``ethtool_ops`` 桥接；
  这些内容留待下一阶段单独实现并提交，避免把“基础对象”和“回调桥接”混成一次提交。
- 编译验证命令：

  - ``/home/lwz/rfl-dev/scripts/build-kernel.sh``

- 编译结果：

  - ``rust/kernel.o`` 编译通过
  - 全量增量内核构建通过
  - 当前仍构建 C 版 ``drivers/net/nlmon.ko``，因为 Rust 驱动尚未接入

- 本阶段遇到的问题：

  - 将基础抽象单独拆出后，若干 ``from_raw()`` 构造入口暂时还未被桥接层调用，
    会被 ``-D warnings`` 视为 ``dead_code``；已仅对这些过渡性内部入口加最小
    ``#[allow(dead_code)]`` 标注，保持阶段切分而不提前混入第 7 阶段内容。

- 下一步：

  - 在 ``rust/kernel/net/`` 中补齐 ``NetDevice`` / ``SetupContext`` / ``Driver`` /
    ``Registration`` 等 netdev 与 rtnl 桥接抽象，完成纯安全驱动所需的回调边界。

7. netdev / rtnl 桥接抽象
~~~~~~~~~~~~~~~~~~~~~~~~~

- 新增 ``rust/kernel/net/rtnl.rs``，实现 ``#[vtable]`` 风格的 ``Driver`` trait，
  以及 ``Registration``、``AttrTable``、``ExtAck``、``TxStatus`` 等桥接对象。
- 扩展 ``rust/kernel/net/netdevice.rs``，补齐：

  - ``DeviceRef<T>``：与具体驱动类型绑定，并提供安全私有数据读取接口
  - ``NetDevice<T>``：封装回调上下文中的可变 ``net_device``
  - ``SetupContext<T>``：限制设备初始化时可设置的字段集合
  - ``install_ops()`` / ``init_private()`` / ``drop_private()``：把回调表安装、
    私有数据初始化与析构全部收敛在抽象层

- 特别修正了私有数据借用接口的生命周期：

  - ``DeviceRef<T>::private()`` 现在返回绑定到 ``&self`` 的借用，而不再使用过宽的
    ``'static`` 返回类型
  - ``NetDevice<T>::private()`` / ``private_mut()`` 则分别绑定到 ``&self`` /
    ``&mut self``，避免把原始 ``netdev_priv()`` 指针泄露到驱动层

- ``Registration<T>`` 内部现已负责：

  - 安装 ``net_device_ops`` / ``ethtool_ops`` / ``rtnl_link_ops``
  - ``setup`` / ``validate`` / ``open`` / ``stop`` / ``start_xmit`` /
    ``get_stats64`` / ``get_link`` 等 C ABI 回调到安全 Rust trait 的桥接
  - 注册失败与 ``Drop`` 时的 ``rtnl_link_unregister()`` 回收

- 编译验证命令：

  - ``/home/lwz/rfl-dev/scripts/build-kernel.sh``

- 编译结果：

  - ``rust/kernel.o`` 编译通过
  - 全量增量内核构建通过
  - 当前 Rust ``net`` 抽象已经可以承载后续 ``drivers/net/nlmon_rust.rs`` 的零
    ``unsafe`` 驱动编写

- 本阶段遇到的问题：

  - 初次编译时 ``build_assert!`` 宏未显式导入，已在抽象模块中补入最小导入修正；
    之后桥接层编译通过。

- 下一步：

  - 新增 ``drivers/net/nlmon_rust.rs`` 驱动骨架，并通过 ``CONFIG_NLMON_RUST`` 把
    Rust 实现真正接入 ``drivers/net/`` 的构建路径。

8. Rust 驱动骨架接入
~~~~~~~~~~~~~~~~~~~~

- 新增 ``drivers/net/nlmon_rust.rs``，完成最小可编译的 Rust 驱动骨架：

  - ``module!`` 元数据
  - ``alias: ["rtnl-link-nlmon"]``
  - ``NlmonModule`` 持有 ``net::Registration<NlmonDriver>``
  - ``NlmonDriver`` 实现 ``net::Driver`` trait

- 当前骨架已接通以下内容：

  - ``Registration::<NlmonDriver>::new()``，使 RTNL link 类型注册路径生效
  - 与 C 版一致的 ``setup`` 语义：

    - ``ARPHRD_NETLINK``
    - ``IFF_NO_QUEUE``
    - ``lltx = true``
    - ``NETIF_F_SG | NETIF_F_FRAGLIST | NETIF_F_HIGHDMA``
    - ``IFF_NOARP``
    - ``NETDEV_PCPU_STAT_LSTATS``
    - ``mtu = NLMSG_GOODSIZE``
    - ``min_mtu = sizeof(struct nlmsghdr)``

  - ``validate`` 的最小实现：拒绝 ``IFLA_ADDRESS``
  - ``get_link`` 的最小实现：返回链路 up

- 当前骨架仍刻意保留占位行为，留待下一阶段对齐：

  - ``open()``
  - ``stop()``
  - ``start_xmit()``
  - ``get_stats64()``
  - ``NetlinkTapHandle`` 私有状态接入

- 本阶段验证前，先用 ``scripts/config`` 确认 ``CONFIG_NLMON=m`` 与
  ``CONFIG_NLMON_RUST=y`` 可以同时成立；``olddefconfig`` 后该组合被保留，
  满足模块化差分测试需求。
- 编译验证命令：

  - ``/home/lwz/rfl-dev/linux/scripts/config --file /home/lwz/rfl-dev/build/.config -m NLMON -e NLMON_RUST``
  - ``make -C "$KERNEL_SRC" O="$KERNEL_BUILD" LLVM=1 olddefconfig``
  - ``/home/lwz/rfl-dev/scripts/build-kernel.sh``
  - ``modinfo /home/lwz/rfl-dev/build/drivers/net/nlmon_rust.ko``

- 验证结果：

  - 构建日志出现 ``RUSTC [M] drivers/net/nlmon_rust.o``
  - 构建日志出现 ``LD [M] drivers/net/nlmon_rust.ko``
  - ``modinfo`` 确认：

    - ``name: nlmon_rust``
    - ``alias: rtnl-link-nlmon``
    - ``description: Rust netlink monitoring device``

- 下一步：

  - 把 ``nlmon.c`` 的 ``open/close/xmit/get_stats64`` 等运行期行为逐项迁入 Rust，
    接通 ``NetlinkTapHandle`` 与统计路径，形成可进入差分测试的等价实现。

9. Rust 驱动行为对齐
~~~~~~~~~~~~~~~~~~~~

- 扩展 ``drivers/net/nlmon_rust.rs`` 中的私有状态：

  - ``NlmonPrivate`` 现在持有 ``net::NetlinkTapHandle``

- 按 ``drivers/net/nlmon.c`` 对齐运行期行为：

  - ``open()``：通过 ``dev.private_mut().tap.add(dev.device_ref())`` 注册 netlink tap
  - ``stop()``：通过 ``tap.remove()`` 注销 netlink tap
  - ``start_xmit()``：调用 ``net::LStats::add()`` 记录字节数并返回 ``NETDEV_TX_OK``
  - ``SkBuff`` 在离开作用域时自动 ``Drop``，由抽象层内部调用 ``consume_skb()``
  - ``get_stats64()``：调用 ``net::LStats::read()`` 填充 ``rx_packets/rx_bytes``

- 当前 ``drivers/net/nlmon_rust.rs`` 继续保持零 ``unsafe``；所有 FFI、原始指针、
  结构体字段写入和 C ABI 回调桥接仍全部位于 ``rust/kernel/net/*`` 抽象层中。
- 编译验证命令：

  - ``/home/lwz/rfl-dev/scripts/build-kernel.sh``

- 验证结果：

  - ``RUSTC [M] drivers/net/nlmon_rust.o`` 成功
  - ``LD [M] drivers/net/nlmon_rust.ko`` 成功
  - 说明当前 Rust 驱动已具备进入 QEMU 差分测试的构建条件

- 下一步：

  - 以原 C 版 ``nlmon`` 为金标准，在 QEMU 中执行同一套创建、启停、抓包与重复删除
    测试，逐项比对 ``ip -d`` / ``ip -s`` / ``pcap`` / ``dmesg`` 结果。

10. C / Rust 差分测试
~~~~~~~~~~~~~~~~~~~~~

- 先后切换 ``CONFIG_NLMON_RUST=n`` 与 ``CONFIG_NLMON_RUST=y``，分别构建 C 基线轮与
  Rust 对照轮；两轮都通过相同脚本更新 ``rootfs`` 中的 ``dummy.ko`` 与对应
  ``nlmon`` 模块，并重新打包 ``initramfs``。
- 实际在 QEMU 中测试时，确认了两个与环境相关的重要事实：

  - BusyBox 版本的 ``ip`` 不支持 ``ip -d link show`` 与 ``ip -s link show``；
    因此最终比较改为 ``ip link show`` + ``/sys/class/net/*`` 读取 ``type`` /
    ``flags`` / ``mtu`` / ``statistics``。
  - 在当前环境里，``insmod /lib/modules/dummy.ko`` 后会直接出现 ``dummy0``，
    因而原始命令 ``ip link add dummy0 type dummy`` 在首次测试中返回
    ``RTNETLINK answers: File exists``。这不是 ``dummy`` 驱动失效，而是接口已存在。
    在删除该接口后，再执行 ``ip link add dummy0 type dummy`` 可以成功，证明
    ``dummy`` 功能本身正常。

- C 基线轮最终使用的观测序列：

  - ``insmod /lib/modules/dummy.ko``
  - ``insmod /lib/modules/nlmon.ko``
  - 复用预存在的 ``dummy0``
  - ``ip link add nlmon0 type nlmon``
  - ``ip link set nlmon0 up``
  - ``tcpdump -i nlmon0 -w /tmp/cf_nlmon.pcap``
  - ``ip link set dummy0 up``
  - ``ip addr add 192.0.2.1/24 dev dummy0``
  - ``tcpdump -nn -r /tmp/cf_nlmon.pcap``
  - ``ip link show`` 与 ``/sys/class/net/*`` 采样

- Rust 对照轮使用完全相同的观测序列，只把 ``insmod /lib/modules/nlmon.ko`` 替换为
  ``insmod /lib/modules/nlmon_rust.ko``。
- C / Rust 两轮对比结果：

  - ``dummy0_preexisting=yes``，两轮一致
  - ``dummy0`` 链路属性一致：

    - ``<BROADCAST,NOARP,UP,LOWER_UP>``
    - ``mtu 1500``
    - 成功配置 ``192.0.2.1/24``

  - ``nlmon0`` 链路属性一致：

    - ``<NOARP,UP,LOWER_UP>``
    - ``mtu 3776``
    - ``link/[824]``

  - ``/sys/class/net/nlmon0`` 关键值一致：

    - ``type = 824``（即 ``ARPHRD_NETLINK``）
    - ``flags = 0x81``
    - ``mtu = 3776``
    - ``rx_packets = 21``
    - ``rx_bytes = 30532``

  - ``/sys/class/net/dummy0/statistics/tx_packets = 1``，两轮一致
  - ``tcpdump -nn -r`` 的文本解码结果均非空，且两轮文本总行数同为 ``494``，
    说明抓包路径与报文数量级一致
  - 严格 ``dmesg`` 检查（``warning/oops/lockdep/kasan/use-after-free``）两轮均为空

- Rust 轮额外执行了重复创建设备/删除设备测试：

  - ``ip link add nlmon0 type nlmon``
  - ``ip link set nlmon0 up``
  - ``ip link del nlmon0``
  - 上述序列连续执行两次，均成功完成
  - 严格 ``dmesg`` 检查仍为空

- 与 ``nlmon`` 无关但测试中反复出现的已知噪声：

  - ``rootfs`` 的 ``/init`` 仍会自动尝试加载旧的
    ``/lib/modules/rust_out_of_tree.ko``，因版本魔数不匹配而报
    ``invalid module format``；这是既有 ``rootfs`` 状态带来的独立问题，不属于
    本轮 ``dummy/nlmon`` C/Rust 对照差异。

- 当前结论：

  - 原 C 版 ``nlmon`` 可正常工作
  - Rust 版 ``nlmon_rust`` 在本轮已实现与 C 版一致的核心外部行为
  - ``dummy`` 在本轮仅作为制造 rtnetlink/netlink 事件的测试发生器，而不是 Rust 化目标
  - 以本轮观测结果看，``nlmon`` Rust 化后的当前实现已经达到可接受的第一版功能对齐状态，
    但还不能据此宣称抽象层已通过工程级安全性验收

11. 总结报告整理
~~~~~~~~~~~~~~~~

- 新增总结文档 ``report-2026-03-18-nlmon-rust-zh_CN.rst``，集中说明：

  - 今天完成了哪些阶段性工作
  - 当前 ``nlmon`` Rust 化处于什么状态
  - C / Rust 差分测试结论如何
  - 当前实现与 Rust-for-Linux 工程要求的符合度
  - 开发中遇到的困难、解决办法与后续优化建议
  - 面向内核的 C2Rust 工具可以从本项目中吸收哪些经验

- 更新 ``Documentation/rust/lwz-dev/index.rst``，将该总结文档加入索引，方便后续查阅和回溯。

后续阶段会继续在本文件中追加记录。

12. 工程口径修正与研究级第二阶段启动
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 基于后续复盘与问题澄清，统一修正本项目当前口径：

  - DUT 始终只有 ``nlmon``
  - ``dummy``、``veth``、``bridge``、``addr/route/rule`` 与 ``netns`` 都只作为
    事件发生器使用
  - ``drivers/net/nlmon_rust.rs`` 为零 ``unsafe``，但 ``rust/kernel/net/*`` 仍含
    ``unsafe``，其健全性仍需研究级审计与收缩

- 因此，本轮截至 ``e6dbe5553`` 的成果应被界定为：

  - 已完成第一版 Rust 原型
  - 已完成基础 C/Rust 差分测试
  - 尚未完成 ``unsafe`` 契约审计
  - 尚未完成调试内核下的强化差分与压力测试
  - 尚未达到“可以宣称工程级安全”的验收状态

- 从本阶段开始，后续工作固定按照以下顺序推进：

  1. 先对 ``rust/kernel/net/*`` 与 ``rust/helpers/net.c`` 做逐点 ``unsafe`` 审计
  2. 再收缩 safe API 表面，修正任何需要调用者脑补生命周期或别名约束的接口
  3. 建立专用调试内核与自动化测试基线
  4. 执行强化差分测试矩阵与长时压力测试
  5. 只有在上述证据链闭合后，才输出工程验收结论与完整可复现流程文档

- 本阶段修改文件：

  - ``Documentation/rust/lwz-dev/report-2026-03-18-nlmon-rust-zh_CN.rst``
  - ``Documentation/rust/lwz-dev/worklog-2026-03-18-zh_CN.rst``

- 本阶段验证命令：

  - ``git -C /home/lwz/rfl-dev/linux diff -- Documentation/rust/lwz-dev/report-2026-03-18-nlmon-rust-zh_CN.rst Documentation/rust/lwz-dev/worklog-2026-03-18-zh_CN.rst``

- 本阶段验证结果：

  - 现有文档已不再把 ``dummy`` 表述成与 ``nlmon`` 并列的改写目标
  - 现有文档已明确当前实现是“第一版原型”，不是“已完成工程级安全验收”的最终版本

- 下一步：

  - 新增正式的 ``unsafe`` 审计文档，逐项记录 ``rust/kernel/net/*`` 与
    ``rust/helpers/net.c`` 中每一个 ``unsafe`` 点的前置条件、后置条件、
    生命周期、别名约束与析构配对关系。

13. ``unsafe`` 契约审计初稿
~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 新增正式审计文档：

  - ``Documentation/rust/lwz-dev/unsafe-audit-2026-03-18-nlmon-rust-zh_CN.rst``

- 审计范围固定为：

  - ``rust/kernel/net/skbuff.rs``
  - ``rust/kernel/net/netdevice.rs``
  - ``rust/kernel/net/rtnl.rs``
  - ``rust/helpers/net.c``

- 本阶段按“前置条件 / 后置条件 / 所有权 / 生命周期 / 别名 / 析构配对 /
  safe API 是否可能破坏不变式”的统一格式，逐项审计所有 ``unsafe`` 点。

- 本阶段得到的关键结论：

  - ``DeviceRef<T>`` 缺少生命周期参数且公开 ``private()``，这是当前 safe API 中
    最需要优先收缩的高风险点
  - ``LStats`` 没有把 ``NETDEV_PCPU_STAT_LSTATS`` 这一配置前提编码进类型
  - ``AttrTable::is_present()`` 没有边界信息，当前是一个 safe 越界入口
  - ``SetupContext::device_mut()`` 泄漏了 setup 阶段不应暴露的运行期能力
  - ``NetlinkTapHandle::add()`` 目前依赖调用者“恰好传的是当前设备”，这一点还没有被 API 编码
  - ``Registration<T>`` 的 ``Send/Sync`` 证明目前仍偏弱，需要在重构阶段复核

- 本阶段修改文件：

  - ``Documentation/rust/lwz-dev/index.rst``
  - ``Documentation/rust/lwz-dev/unsafe-audit-2026-03-18-nlmon-rust-zh_CN.rst``
  - ``Documentation/rust/lwz-dev/worklog-2026-03-18-zh_CN.rst``

- 本阶段验证命令：

  - ``rg -n \"unsafe|unsafe impl|extern \\\"C\\\"\" rust/kernel/net rust/helpers/net.c``
  - ``nl -ba rust/kernel/net/skbuff.rs``
  - ``nl -ba rust/kernel/net/netdevice.rs``
  - ``nl -ba rust/kernel/net/rtnl.rs``
  - ``nl -ba rust/helpers/net.c``

- 本阶段验证结果：

  - 已形成一份可交接的正式审计文档
  - 已明确 P0/P1/P2 级别的抽象收缩任务
  - 在完成这些改造前，不应继续把当前抽象层称为“已证明健全的 safe API”

- 下一步：

  - 按审计文档给出的 P0/P1 顺序重构 ``rust/kernel/net/*``，
    先收缩 ``DeviceRef`` / ``LStats`` / ``AttrTable`` / ``SetupContext`` 的 safe API 表面，
    再重新编译并进入调试内核与强化差分测试阶段。

14. 抽象层第一轮收缩重构
~~~~~~~~~~~~~~~~~~~~~~~

- 按上一阶段审计结论，先实现一轮最小但实质性的 safe API 收缩，修改文件如下：

  - ``rust/kernel/net/skbuff.rs``
  - ``rust/kernel/net/netdevice.rs``
  - ``rust/kernel/net/rtnl.rs``
  - ``rust/kernel/net.rs``
  - ``drivers/net/nlmon_rust.rs``

- 本阶段完成的核心改造：

  - ``DeviceRef`` 现在带有显式生命周期参数，不再暴露 ``private()``，从类型层面阻断
    了“跨回调缓存设备句柄再读取私有区”的 safe 逃逸路径
  - ``NetDevice::with_private()`` 取代了驱动层手动拼接设备句柄与私有状态的方式，
    让 ``open()`` 这类路径可以在受限闭包中同时使用 ``&mut Private`` 和
    回调期设备引用
  - ``SetupContext`` 删除 ``device_mut()``，同时把通用 ``set_pcpu_stat_type()``
    收缩为 ``enable_lstats()``，避免 setup 阶段继续泄漏运行期能力
  - 原先全局静态的 ``LStats`` safe API 被替换为回调期 ``LStatsHandle`` 能力对象；
    只有在运行时确认设备已配置 ``NETDEV_PCPU_STAT_LSTATS`` 后，驱动才能拿到该能力
  - ``AttrTable`` 新增 ``max_index`` 边界信息；``validate`` 回调现在对 top-level attrs
    使用 ``__IFLA_MAX - 1``，对 driver-private attrs 使用 ``DATA_ATTR_MAX``，
    不再保留无边界的 safe 越界入口
  - ``SkBuff::into_raw()`` 从 safe 驱动接口中移除，继续缩小裸指针泄漏面
  - ``Registration<T>`` 的 ``Send/Sync`` 断言未被简单删除，而是保留并补强说明；
    实际编译验证表明 ``Module`` / ``InPlaceModule`` 仍要求模块状态满足
    ``Send + Sync``，因此这组断言目前仍是必要边界

- 驱动层同步调整：

  - ``drivers/net/nlmon_rust.rs`` 继续保持零 ``unsafe``
  - ``open()`` 改为使用 ``with_private()``
  - ``start_xmit()`` / ``get_stats64()`` 改为通过 ``dev.lstats()`` 获取受检统计能力

- 本阶段验证命令：

  - ``/home/lwz/rfl-dev/scripts/build-kernel.sh``

- 本阶段验证结果：

  - ``rust/kernel.o`` 编译通过
  - ``RUSTC [M] drivers/net/nlmon_rust.o`` 成功
  - ``LD [M] drivers/net/nlmon_rust.ko`` 成功
  - ``arch/x86/boot/bzImage`` 重建成功

- 本阶段遇到的问题与修正：

  - 初版 ``with_private()`` 因为同时持有 ``&self`` 与 ``&mut self`` 触发借用冲突，
    后续改为先提取原始 ``net_device *``，再在受控 ``unsafe`` 边界内构造回调期
    ``DeviceRef``，并确保该类型已不再暴露私有区访问
  - 试图直接去掉 ``Registration<T>`` 的 ``Send/Sync`` 断言后，模块编译失败，
    说明当前内核 Rust 模块基础设施确实要求模块状态满足这两个 trait；因此本阶段
    选择保留断言，但补强其局部不变式说明，并保留后续继续审查空间

- 下一步：

  - 开始建立专用调试内核输出目录、配置片段、测试 rootfs 与自动化测试脚本，
    为强化差分测试矩阵做准备。

15. 调试内核与自动化测试基线脚本
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在仓库中新增可追踪的测试基础设施目录：

  - ``tools/testing/rust/nlmon/common.sh``
  - ``tools/testing/rust/nlmon/configure-debug-kernel.sh``
  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh``
  - ``tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - ``tools/testing/rust/nlmon/run-qemu-test.sh``

- 本阶段设计目标：

  - 不再复用单一 ``/home/lwz/rfl-dev/build`` 作为所有测试场景的输出目录
  - 用仓库内脚本固定化三套调试 profile：

    - ``memory-debug`` -> ``/home/lwz/rfl-dev/build-nlmon-kasan``
    - ``concurrency-debug`` -> ``/home/lwz/rfl-dev/build-nlmon-lockdep``
    - ``leak-debug`` -> ``/home/lwz/rfl-dev/build-nlmon-kmem``

  - 用专用 initramfs 和自动执行的 guest runner 取代“手工进 shell 再逐条敲命令”的测试方式

- 脚本能力概览：

  - ``configure-debug-kernel.sh``：

    - 从现有基线 ``.config`` 派生独立 build dir
    - 打开 ``NLMON/DUMMY/VETH/BRIDGE/STP/LLC`` 等事件发生器所需配置
    - 根据 profile 打开 ``KASAN``、``PROVE_LOCKING``、``DEBUG_KMEMLEAK`` 等调试项
    - 在 ``olddefconfig`` 后显式检查关键配置是否真的生效，并对缺失项打印 warning

  - ``prepare-test-rootfs.sh``：

    - 生成独立测试 rootfs staging tree 与 ``initramfs-nlmon-test.cpio.gz``
    - 除 BusyBox 外，额外分发宿主机上的 ``ip``、``tcpdump``、``sha256sum``、
      ``cmp``、``awk``、``sed``、``grep``
    - 若宿主机存在 ``ethtool`` 则一并分发；若不存在，则在准备阶段打印降级 warning
    - 按 ``c/rust`` 选择要打包的 ``nlmon`` 模块，并一并打包 ``dummy/llc/stp/bridge/veth``
    - 生成自动执行的 ``/init`` 与 ``/etc/nlmon-test.env``

  - ``nlmon-guest-runner.sh``：

    - 在 guest 中自动加载模块
    - 自动运行 ``baseline/lifecycle/matrix/stress`` 场景
    - 自动触发 ``dummy/veth/bridge/route_rule/netns`` 五类事件发生器
    - 自动采集 ``ip -d`` / ``ip -s`` / ``sysfs`` / ``tcpdump`` / ``sha256sum`` / ``dmesg``
    - 自动检测 ``KASAN/UAF/lockdep/RCU/kmemleak`` 关键词

  - ``run-qemu-test.sh``：

    - 接收 ``build dir``、``initramfs``、``log file`` 和超时参数
    - 自动启动 QEMU 并把串口日志重定向到主机文件

- 本阶段验证命令：

  - ``bash -n tools/testing/rust/nlmon/common.sh tools/testing/rust/nlmon/configure-debug-kernel.sh tools/testing/rust/nlmon/prepare-test-rootfs.sh tools/testing/rust/nlmon/run-qemu-test.sh``
  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - ``tools/testing/rust/nlmon/configure-debug-kernel.sh memory-debug``
  - ``rg -n "CONFIG_(KASAN|SLUB_DEBUG_ON|DEBUG_OBJECTS|NLMON|DUMMY|VETH|BRIDGE|STP|LLC|NET_NS|PACKET|IP_MULTIPLE_TABLES|RUST)=" /home/lwz/rfl-dev/build-nlmon-kasan/.config``

- 本阶段验证结果：

  - 新增脚本均通过语法检查
  - ``memory-debug`` profile 已成功生成 ``/home/lwz/rfl-dev/build-nlmon-kasan/.config``
  - 关键配置项已确认开启：

    - ``CONFIG_RUST=y``
    - ``CONFIG_NET_NS=y``
    - ``CONFIG_PACKET=y``
    - ``CONFIG_IP_MULTIPLE_TABLES=y``
    - ``CONFIG_STP=m``
    - ``CONFIG_BRIDGE=m``
    - ``CONFIG_LLC=m``
    - ``CONFIG_DUMMY=m``
    - ``CONFIG_VETH=m``
    - ``CONFIG_NLMON=m``
    - ``CONFIG_SLUB_DEBUG_ON=y``
    - ``CONFIG_DEBUG_OBJECTS=y``
    - ``CONFIG_KASAN=y``
    - ``CONFIG_KASAN_GENERIC=y``

- 本阶段发现的环境/树内现实问题：

  - 当前这棵内核树里没有 ``CONFIG_REFCOUNT_FULL``，因此 ``memory-debug`` profile
    在校验阶段会显式给出 warning，而不是静默假装已经开启
  - 宿主机当前没有 ``/usr/sbin/ethtool``，因此测试 rootfs 准备脚本会显式记录
    观测降级，而不是假装具备完整 ``ethtool`` 采样能力

- 下一步：

  - 基于这些脚本继续生成并构建三套调试内核
  - 准备 C / Rust 两轮测试所需的专用 initramfs
  - 先执行功能基线与生命周期循环，再进入行为矩阵和长时压力测试。
