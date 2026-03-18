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

16. guest 结果摘要外显化
~~~~~~~~~~~~~~~~~~~~~~~

- 在准备实际执行 QEMU 差分前，发现上一阶段的 guest runner 虽然会在 guest 内部留下
  ``pcap``、解码结果和接口观测文件，但这些信息不会自动出现在主机侧串口日志中。
  如果不先修正，后续即使测试完成，也无法在主机侧自动对比关键指标。

- 因此本阶段先最小修正 ``tools/testing/rust/nlmon/nlmon-guest-runner.sh``：

  - 新增 ``emit_file_value()``，把 guest 内部结果文件压缩成单行结果输出
  - 在 ``finalize_capture()`` 中追加输出：

    - ``pcap`` 的 ``sha256``
    - ``tcpdump -nn -r`` 的文本总行数

  - 在 ``observe_iface()`` 中追加输出：

    - ``type``
    - ``flags``
    - ``mtu``
    - ``rx_packets``
    - ``rx_bytes``
    - ``tx_packets``
    - ``tx_bytes``

- 本阶段验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``

- 本阶段验证结果：

  - guest runner 语法检查通过
  - 后续主机侧 QEMU 日志已具备承载关键差分指标的能力，不再只能看到 ``status=ok`` 这种过粗粒度结果

- 下一步：

  - 开始真正构建调试 profile、准备测试 initramfs，并执行 C / Rust 的功能基线和生命周期循环差分。

17. 自动化测试框架稳健性修正
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在首次执行 ``memory-debug`` / C 版 baseline 场景时，QEMU 已成功启动并进入自动化
  guest runner，但测试并未顺利结束：

  - guest 侧确实已经开始输出 ``NLMON_RESULT`` 摘要
  - 但在事件发生器清理路径上出现了 ``ip: SIOCGIFFLAGS: No such device``
  - 由于 guest runner 与 ``/init`` 都是 ``set -e``，导致 ``init`` 提前退出，
    最终触发 ``Attempted to kill init!`` 类型的 panic

- 这一失败说明：

  - 当前遇到的不是 ``nlmon`` 行为差异，而是测试框架本身对“设备可能在清理阶段已自动消失”
    这一现实处理得不够稳健
  - 如果不先修复，后续所有差分都会混入测试框架自身失败带来的噪声

- 因此本阶段先修正测试脚本：

  - ``tools/testing/rust/nlmon/nlmon-guest-runner.sh``：

    - 为各个事件发生器补充 ``log`` 输出，明确当前执行到哪一类事件
    - 对 ``ip link del`` / ``ip netns del`` 这类清理命令改为 ``|| true``，
      避免设备已被联动删除时误判为测试失败

  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh``：

    - 调整生成的 ``/init``，改为显式捕获 guest runner 返回码，再执行
      ``poweroff/reboot``，避免因为 ``set -e`` 直接杀掉 ``init`` 进程

- 本阶段验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - ``bash -n tools/testing/rust/nlmon/prepare-test-rootfs.sh``

- 本阶段验证结果：

  - 两个脚本语法检查通过
  - 后续 baseline 重跑时，guest 失败将不再直接表现为 ``init`` 被杀导致的 panic，
    而会在串口日志中保留更可分析的错误上下文

- 下一步：

  - 基于修正后的脚本重新执行 ``memory-debug`` 下的 C / Rust baseline，
    再进入生命周期循环差分。

18. 事件发生器“尽力执行”策略
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在修正 ``init`` / 清理路径之后重新观察 baseline 日志，发现失败点进一步收缩为：

  - ``veth`` 事件发生器中的附加配置步骤（``ip link set ... up`` /
    ``ip addr add ...``）会在当前 guest 环境里触发 ``No such device``
  - 这些步骤的失败会提前终止整轮 baseline，但它们本身并不是
    “是否能够产生 netlink 事件”的最小必要条件

- 因此本阶段对 ``tools/testing/rust/nlmon/nlmon-guest-runner.sh`` 做第二轮稳健性调整：

  - ``gen_veth()``
  - ``gen_bridge()``
  - ``gen_netns()``

  中那些用于“增加事件丰富度”的附加步骤统一改为 ``|| true``：

  - 接口 ``up``
  - 地址配置
  - bridge 挂载
  - netns 内部接口启用

- 调整后的原则是：

  - 设备/namespace 的创建本身仍然必须成功，否则该发生器确实无效
  - 但创建成功后那些额外增强步骤若失败，不应直接把整轮差分测试打断；
    测试应优先完成主线采样并把失败上下文保留在日志中

- 本阶段验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``

- 本阶段验证结果：

  - guest runner 语法检查继续通过
  - baseline 后续重跑时，``veth/bridge/netns`` 的附加动作将不再把整个场景提前打断

- 下一步：

  - 重新执行 ``memory-debug`` / C 版 baseline，确认 baseline 可以完整结束并输出
    足够的主机侧摘要结果；
  - 若通过，再切换到 Rust 版执行同一路径对照。

19. ``route_rule`` 发生器降级为尽力执行
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在再次重跑 ``memory-debug`` / C 版 baseline 后，自动化框架已经不再因为
  ``init`` 退出而 panic，但新的提前终止点暴露为：

  - ``route_rule`` 发生器中的 ``ip route add ... dev lo table 100``
  - 当前 guest 环境下该步骤会报 ``RTNETLINK answers: Network is down``

- 这一失败的性质与上一阶段相同：

  - 它属于“为了制造更多 netlink 事件而执行的增强步骤”
  - 不是验证 ``nlmon`` 创建、启停、抓包与统计主路径是否成立的最小必要条件

- 因此本阶段继续把 ``gen_route_rule()`` 调整为“尽力执行”：

  - 先尝试 ``ip link set lo up || true``
  - ``route add`` / ``rule add`` / ``rule del`` / ``route del`` 全部改为 ``|| true``

- 本阶段验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``

- 本阶段验证结果：

  - guest runner 语法检查继续通过
  - 后续 baseline 重跑时，``route_rule`` 的增强步骤将不再直接中断整轮采样

- 下一步：

  - 再次重跑 ``memory-debug`` / C 版 baseline，直到基线场景能够完整结束并输出
    可用于对照的摘要结果。

20. 修正 guest ``ip`` 工具优先级
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在继续追查 baseline 提前退出原因时，串口日志给出了决定性线索：

  - guest 中打印出了 BusyBox ``ip`` 的帮助文本
  - 这说明当前测试实际上优先命中了 ``/bin/ip``（BusyBox applet），而不是
    事先分发进 rootfs 的完整 ``/usr/sbin/ip``（iproute2）

- 这一点非常关键，因为：

  - BusyBox ``ip`` 不支持本项目需要的完整 ``veth`` / ``bridge`` / ``netns`` /
    ``ip -d`` 行为
  - 如果不先修正，之前发生器失败的很多现象都不能作为 ``nlmon`` 的有效测试信号

- 因此本阶段修正三个位置：

  - ``tools/testing/rust/nlmon/common.sh``

    - BusyBox applet 列表中不再创建 ``/bin/ip`` 链接

  - ``tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh``

    - 统一把 guest ``PATH`` 调整为 ``/usr/sbin:/usr/bin:/bin:/sbin``，
      确保完整 ``iproute2`` 优先于 BusyBox applet

- 本阶段验证命令：

  - ``bash -n tools/testing/rust/nlmon/common.sh tools/testing/rust/nlmon/prepare-test-rootfs.sh``
  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``

- 本阶段验证结果：

  - 相关脚本语法检查继续通过
  - 后续 rootfs 重建后，guest 将优先使用完整 ``iproute2``，此前由 BusyBox ``ip``
    造成的能力缺失不再污染基线结论

- 下一步：

  - 基于修正后的 rootfs 再次重跑 ``memory-debug`` / C 版 baseline，
    观察五类事件发生器是否终于能在完整 ``iproute2`` 下走完主路径。

21. 修正 rootfs 中的 ``iproute2`` 软链接污染
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在继续复盘 ``memory-debug`` / C 版 baseline 的失败根因时，又确认了一个更底层的问题：

  - 宿主机上的 ``/usr/sbin/ip`` 本身是指向 ``/bin/ip`` 的软链接
  - 但测试 rootfs 内部的 ``/bin`` 是真实目录，不是宿主机那样指向 ``/usr/bin`` 的层级
  - 旧版 staging 逻辑使用 ``cp -a`` 原样复制软链接，结果 guest 里留下了
    ``/usr/sbin/ip -> /bin/ip`` 的悬空链接
  - 同样的问题也会影响 ``ld-linux`` 等动态装载器路径，意味着此前“已经分发了完整二进制”
    这一前提并不成立

- 这解释了为什么前一阶段虽然调整了 ``PATH``，guest 仍然会落回 BusyBox ``ip`` 或出现能力缺失。

- 因此本阶段对测试基础设施做两点修正：

  - ``tools/testing/rust/nlmon/common.sh``

    - ``nlmon_copy_path_with_parents()`` 改为复制 ``readlink -f`` 解析后的真实文件，
      不再把宿主机的符号链接原样带进 guest rootfs

  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh``

    - ``ip`` 改为直接从 ``/usr/bin/ip`` 分发

  - ``tools/testing/rust/nlmon/nlmon-guest-runner.sh``

    - 新增 ``IP_BIN`` / ``ip_cmd()``
    - 全部 ``ip`` 调用统一走显式的完整 ``iproute2`` 路径
    - 在测试结果中额外记录 ``ip_binary`` 与 ``ip_version``，用于后续审计基线可信度

- 本阶段预期验证命令：

  - ``bash -n tools/testing/rust/nlmon/common.sh tools/testing/rust/nlmon/prepare-test-rootfs.sh``
  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - 重新生成 ``memory-debug`` / C 版 rootfs 并重跑 baseline

- 实际验证命令：

  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --implementation c --scenario baseline --rootfs-dir /home/lwz/rfl-dev/rootfs/nlmon-baseline-c-stage --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-baseline-c.cpio.gz``
  - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-baseline-c.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/memory-debug-c-baseline.log --timeout-seconds 600``

- 实际验证结果：

  - 重新生成后的 rootfs 中：

    - ``/usr/bin/ip`` 已为真实 ELF 二进制
    - ``/lib64/ld-linux-x86-64.so.2`` 已为真实 ELF 装载器，不再是悬空软链接

  - ``memory-debug`` / C 版 baseline 首次完整收尾：

    - 串口日志记录 ``NLMON_RESULT: ip_binary=/usr/bin/ip``
    - 串口日志记录 ``NLMON_RESULT: ip_version=ip utility, iproute2-5.15.0, libbpf 0``
    - ``NLMON_RESULT: observation_mode.dummy=full-iproute2``
    - ``NLMON_RESULT: observation_mode.baseline.nlmon0=full-iproute2``
    - ``NLMON_RESULT: status=ok``
    - ``=== nlmon automated test exit rc=0 ===``

  - 这说明 guest 里的 ``ip`` 路径污染问题已经被修复，C 版 baseline 现在终于具备“可信基线”的前提。

  - 同时也暴露出下一轮需要继续追查的两个剩余问题：

    - ``NLMON_RESULT: baseline.decoded.lines=0``，当前抓包结果仍未形成可解析报文清单
    - ``NLMON_RESULT: dmesg_anomaly.baseline=1``，仍需继续收集并分类异常来源，不能直接当成 ``nlmon`` 缺陷或通过项

- 下一步：

  - 先把这一轮“测试基础设施修复”作为单独任务提交
  - 然后继续跑 ``memory-debug`` / C 版 lifecycle，并补强结果采集，使 ``pcap`` 与 ``dmesg`` 的问题能够在主机侧被精确归因

22. 收紧 ``dmesg`` 异常规则并补强 ``tcpdump`` 结果回传
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在完成 ``memory-debug`` 的 C/Rust baseline + lifecycle 对照后，当前主机侧仍有两个盲点：

  - ``baseline.decoded.lines=0``，无法仅凭这一项判断是“没有抓到 netlink 报文”，还是
    ``tcpdump``/``pcap``/``decode`` 某个环节自身出了问题
  - ``dmesg_anomaly.*=1`` 的判定规则过宽，尤其旧规则直接匹配裸字符串 ``KASAN``，
    很可能把 ``KernelAddressSanitizer initialized`` 这类调试内核正常启动横幅误判为异常

- 因此本阶段要对 ``tools/testing/rust/nlmon/nlmon-guest-runner.sh`` 做两类收紧：

  - ``finalize_capture()``

    - 额外回传 ``pcap`` 字节数
    - 回传 ``tcpdump stdout/stderr`` 行数
    - 回传 ``decode.err`` 行数
    - 回传 ``tcpdump.stderr`` 与 ``decode.err`` 的前几行摘要

  - ``scan_dmesg()``

    - 将异常匹配从裸 ``KASAN`` / ``lockdep`` / ``RCU`` 等宽泛关键词，
      收紧为更像真实缺陷信号的模式，例如：

      - ``BUG:``
      - ``WARNING:``
      - ``KASAN:``
      - ``KFENCE:``
      - ``UBSAN:``
      - ``use-after-free``
      - ``double free`` / ``double-free``
      - ``lockdep:``
      - ``possible recursive locking detected``
      - ``suspicious RCU usage``
      - ``refcount_t:``
      - ``kmemleak``
      - ``DEBUG_OBJECTS``

    - 额外回传 ``dmesg`` 命中的行数与前几行摘要，方便主机侧直接判读

- 本阶段预期验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - 重新执行 ``memory-debug`` 下的 C/Rust baseline

- 预期目标：

  - 主机侧串口日志足以判断 ``pcap`` 为何仍为空
  - ``dmesg_anomaly`` 不再被调试内核自身的正常启动横幅误报污染

- 实际验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --implementation rust --scenario baseline --rootfs-dir /home/lwz/rfl-dev/rootfs/nlmon-baseline-rust-stage --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-baseline-rust.cpio.gz``
  - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-baseline-rust.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/memory-debug-rust-baseline.log --timeout-seconds 600``
  - ``scripts/config --file /home/lwz/rfl-dev/build-nlmon-kasan/.config -d NLMON_RUST`` 后重建 ``memory-debug`` / C 版内核与模块
  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --implementation c --scenario baseline --rootfs-dir /home/lwz/rfl-dev/rootfs/nlmon-baseline-c-stage --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-baseline-c.cpio.gz``
  - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-baseline-c.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/memory-debug-c-baseline.log --timeout-seconds 600``

- 实际验证结果：

  - 新摘要结果已成功回传到主机侧串口日志。

  - ``dmesg`` 误报问题已经被修正：

    - C 版 ``NLMON_RESULT: dmesg_anomaly.baseline=0``
    - Rust 版 ``NLMON_RESULT: dmesg_anomaly.baseline=0``
    - C/Rust 两侧 ``dmesg_anomaly_lines.baseline=0``

  - C/Rust baseline 的摘要在去掉 ``implementation=...`` 这一行之后一致，说明当前新增的诊断信号本身没有引入新的实现差异。

  - ``pcap`` 为空的问题也已被精确归因：

    - C/Rust 两侧都记录 ``baseline.pcap.bytes=24``
    - C/Rust 两侧都记录 ``baseline.decoded.lines=0``
    - C/Rust 两侧都记录 ``baseline.tcpdump.stderr.head=tcpdump: Couldn't find user 'tcpdump'``
    - C/Rust 两侧都记录 ``baseline.decode.err.head=... tcpdump: Couldn't find user 'tcpdump'``

  - 这说明当前 ``pcap`` 为空并不是 C/Rust 行为差异，而是测试 rootfs 缺少 ``tcpdump`` 默认降权所需的用户信息，
    导致 ``tcpdump`` 只留下 24 字节的 ``pcap`` 头后就失败。

  - 另外，这一轮也暴露了一个流程性注意点：

    - 当同一个 ``build-nlmon-kasan`` 目录在 C/Rust 之间切换后，旧实现的 ``.ko`` 可能残留但与新 ``bzImage`` 的
      ``vermagic`` 不匹配
    - 已在本轮通过重新切回 C 配置并完整重建修正这一问题
    - 后续对照时必须明确记录“当前 build 目录对应的是哪一个实现”

- 下一步：

  - 单独修复 guest rootfs 中的 ``tcpdump`` 用户环境
  - 让 ``pcap`` 能真正落下报文后，再重新执行 baseline / lifecycle 差分并更新阶段报告

23. 为 guest rootfs 补齐 ``tcpdump`` 运行身份
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 经过上一阶段的主机侧摘要增强，``pcap`` 为空的根因已经明确：

  - C/Rust 两侧都记录 ``baseline.pcap.bytes=24``
  - C/Rust 两侧都记录 ``baseline.tcpdump.stderr.head=tcpdump: Couldn't find user 'tcpdump'``

- 这说明当前问题不在 ``nlmon`` 本体，而在测试 rootfs 太“干净”：

  - ``tcpdump`` 在 root 身份启动后会默认降权到 ``tcpdump`` 用户
  - 但当前 guest rootfs 没有 ``/etc/passwd`` / ``/etc/group`` 中的对应条目
  - 因此它会在打开接口后直接报错，最终只留下 24 字节的 ``pcap`` 头

- 因此本阶段对 ``tools/testing/rust/nlmon/prepare-test-rootfs.sh`` 做最小修正：

  - 在生成 rootfs 时写入最小化的 ``/etc/passwd``

    - ``root``
    - ``tcpdump``

  - 同时写入最小化的 ``/etc/group``

    - ``root``
    - ``tcpdump``

- 本阶段预期验证命令：

  - 重新生成 C/Rust baseline rootfs
  - 重新执行 ``memory-debug`` / C baseline
  - 重新切回 Rust 实现并执行 ``memory-debug`` / Rust baseline

- 预期目标：

  - ``baseline.tcpdump.stderr.head`` 不再出现 ``Couldn't find user 'tcpdump'``
  - ``baseline.pcap.bytes`` 明显大于 24
  - ``baseline.decoded.lines`` 大于 0

- 实际验证命令：

  - ``bash -n tools/testing/rust/nlmon/prepare-test-rootfs.sh``
  - 重新生成并执行 ``memory-debug`` / C baseline
  - 切回 ``CONFIG_NLMON_RUST=y`` 并重建后，重新生成并执行 ``memory-debug`` / Rust baseline

- 实际验证结果：

  - ``tcpdump`` 用户缺失问题已经被修正：

    - C 版不再出现 ``Couldn't find user 'tcpdump'``
    - Rust 版不再出现 ``Couldn't find user 'tcpdump'``
    - C/Rust 两侧 ``baseline.decode.err.head`` 都只剩下正常的 ``reading from file ...`` 提示

  - 这说明 ``/etc/passwd`` / ``/etc/group`` 的最小补齐已经生效。

  - 但新的摘要也暴露出下一层问题：

    - C 版 ``baseline.tcpdump.stderr.head`` 变为
      ``... 0 packets captured 158 packets received by filter ...``
    - Rust 版 ``baseline.tcpdump.stderr.head`` 变为
      ``... 10 packets captured 158 packets received by filter ...``
    - C 版 ``baseline.pcap.bytes=24``，``baseline.decoded.lines=0``
    - Rust 版 ``baseline.pcap.bytes=1568``，``baseline.decoded.lines=101``

  - 因此，本阶段的结论应当谨慎表述为：

    - ``tcpdump`` 运行身份问题已经解决
    - 但当前 baseline 采样链路仍然存在明显的“捕获/写盘/收尾时序不稳定”
    - 在未继续收敛该时序问题之前，不能把这组 C/Rust 差异直接解读为功能差异

  - 同时两边仍然保持：

    - ``dmesg_anomaly.baseline=0``
    - ``observation_mode.baseline.nlmon0=full-iproute2``
    - ``status=ok``

- 下一步：

  - 单独修正 ``tcpdump`` 的缓冲与收尾时序
  - 目标是先让 C/Rust baseline 都稳定产出非空 ``pcap``，再继续 lifecycle 与阶段报告

24. 收敛 ``tcpdump`` 捕获缓冲与收尾时序
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在补齐 ``tcpdump`` 用户身份后，baseline 摘要已经不再报用户错误，但暴露出新的不稳定性：

  - C 版仍然出现 ``0 packets captured / 158 packets received by filter``
  - Rust 版则出现 ``10 packets captured / 158 packets received by filter``

- 这说明当前更像是测试 harness 的采样时序还不稳定，而不是可以直接据此断言 C/Rust 行为不同。

- 因此本阶段继续只调整 ``tools/testing/rust/nlmon/nlmon-guest-runner.sh`` 的抓包收尾行为：

  - ``start_capture()``

    - 为 ``tcpdump`` 增加 ``-U``，启用 packet-buffered 输出

  - ``stop_capture()``

    - 在发送 ``SIGINT`` 之前增加一个短暂 drain 窗口

- 本阶段预期验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - 重新执行 ``memory-debug`` / Rust baseline
  - 重新切回 C 实现并执行 ``memory-debug`` / C baseline

- 预期目标：

  - C/Rust baseline 都能稳定产出非空 ``pcap``
  - ``tcpdump.stderr.head`` 中的 captured 计数与 ``pcap``/decoded 结果一致

- 实际验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - 重新执行 ``memory-debug`` / Rust baseline
  - 切回 ``CONFIG_NLMON_RUST=n`` 并重建后，重新执行 ``memory-debug`` / C baseline

- 实际验证结果：

  - 这一轮 capture drain 调整有效收敛了 baseline 抓包链路：

    - Rust 版 ``baseline.pcap.bytes=38596``
    - Rust 版 ``baseline.decoded.lines=2474``
    - Rust 版 ``baseline.tcpdump.stderr.head=... 158 packets captured 158 packets received by filter ...``

    - C 版 ``baseline.pcap.bytes=38596``
    - C 版 ``baseline.decoded.lines=2474``
    - C 版 ``baseline.tcpdump.stderr.head=... 158 packets captured 158 packets received by filter ...``

  - 同时两边仍然保持：

    - ``dmesg_anomaly.baseline=0``
    - ``observation_mode.baseline.nlmon0=full-iproute2``
    - ``status=ok``

  - 当前 C/Rust baseline 还剩下一个待解释差异：

    - 原始 ``pcap`` 的 ``sha256`` 不同

  - 但在现有主机侧证据下，两边已经表现出：

    - 相同的 ``pcap`` 字节数
    - 相同的 decoded 行数
    - 相同的 captured/filter 计数

  - 因此更合理的临时判断是：

    - 当前差异更可能来自 ``pcap`` 二进制层面的时间戳或记录细节
    - 在拿到更强的主机侧内容摘要前，不能仅凭 ``sha256`` 不同就判定功能不等价

- 下一步：

  - 继续补跑 C/Rust lifecycle
  - 若 lifecycle 也收敛，再把 baseline + lifecycle 的差分结果写入阶段报告

25. 输出 ``memory-debug`` 强化差分阶段报告
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在完成当前一轮 baseline + lifecycle 对照后，已经具备一组比“第一版原型报告”更可靠的结论：

  - ``memory-debug`` 内核下：

    - C/Rust baseline 都可稳定完成
    - C/Rust lifecycle 都可稳定完成
    - ``dmesg_anomaly`` 在两边都为 0
    - baseline 的 ``pcap`` 字节数、decoded 行数与 captured/filter 计数已收敛

- 因此本阶段需要把这些结果从工作日志中提升为正式文档：

  - 新增一份中文强化差分报告
  - 将其加入 ``Documentation/rust/lwz-dev/index.rst``
  - 在旧的原型阶段报告顶部补充“已被二阶段结果补充”的提示，避免读者误读为最新结论

- 本阶段预期目标：

  - 后续接手者可以直接阅读正式报告，而不必先翻完整工作日志
  - 文档中明确写清：

    - 当前已经成立的证据
    - 当前仍未成立的证据
    - 还需要继续做哪些测试

- 实际验证结果：

  - 已新增：

    - ``Documentation/rust/lwz-dev/diff-test-report-2026-03-18-nlmon-memory-debug-zh_CN.rst``

  - 已更新：

    - ``Documentation/rust/lwz-dev/index.rst``
    - ``Documentation/rust/lwz-dev/report-2026-03-18-nlmon-rust-zh_CN.rst``

  - 新报告明确记录了：

    - ``memory-debug`` 下 ``baseline`` 已收敛
    - ``memory-debug`` 下 ``lifecycle`` 已收敛
    - 当前只能做“阶段性收敛”判断，不能提前写成最终工程验收

- 下一步：

  - 若继续推进本轮计划，应转入 ``matrix`` / ``stress`` 与其他调试内核配置

26. 输出 Rust 化流程指导手册
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在完成 ``memory-debug`` 强化差分报告后，还缺少一份更适合长期复用的“方法手册”：

  - 它不应该只复述 ``nlmon`` 某一轮测试结果
  - 而应把这次从选目标、扩 bindgen、补 helper、做抽象、审计 ``unsafe``、
    建调试内核、做差分测试，到修正测试基础设施的一整套顺序固定下来

- 因此本阶段新增：

  - ``Documentation/rust/lwz-dev/driver-rustification-playbook-zh_CN.rst``

- 手册明确固化了以下内容：

  - Rust 化内核模块的推荐顺序
  - 关键硬约束
  - bindgen / helper / 抽象 / 驱动的职责边界
  - 调试内核与差分测试的推进顺序
  - 本次 ``nlmon`` 实验里已经踩过的典型陷阱及规避方法
  - 交付检查表

- 同时更新：

  - ``Documentation/rust/lwz-dev/index.rst``

  将该手册加入本地中文文档索引，方便后续直接查阅。

- 本阶段预期目标：

  - 后续新开窗口时，不需要再重复梳理这次 ``nlmon`` 的经验
  - 下一次迁移更复杂模块时，可以直接以该手册为操作底稿

- 下一步：

  - 将当前分支上的文档与代码变更整体推送到 ``origin/feature/nlmon-rust``

27. 修正 ``stress`` 场景的 guest 空间耗尽
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在按计划推进 ``memory-debug`` / Rust ``stress`` 时，首次出现了新的测试基础设施瓶颈：

  - guest 最终以 ``rc=1`` 退出
  - 串口日志显示：

    - ``tcpdump: ... 1754012 packets captured ...``
    - ``tcpdump: Unable to write output: No space left on device``
    - ``sh: write error: No space left on device``

- 关键判断：

  - 这并不是 ``nlmon`` 本体失效
  - ``tcpdump`` 已经成功抓到大量报文
  - 真正把 guest 写爆的是 ``finalize_capture()`` 中那一步“把整份 ``pcap`` 再全量解码成文本文件”

- 因此本阶段继续只调整测试 harness：

  - ``tools/testing/rust/nlmon/nlmon-guest-runner.sh``

    - 不再把完整 decoded 文本落地到 guest rootfs
    - 改为“流式统计 decoded 总行数 + 仅保留前 20 行摘要”

- 这样做的理由是：

  - 对差分测试而言，长时 ``stress`` 更重要的是：

    - ``pcap`` 是否存在且大小合理
    - decoded 总行数
    - 头部内容摘要
    - ``dmesg`` 是否异常

  - 没必要在 guest 内保存整份超大 decoded 文本，后续若要深入比对，应改在主机侧对 ``pcap`` 做规范化处理

- 本阶段预期验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - 重新执行 ``memory-debug`` / Rust ``stress``

- 下一步：

  - 若 Rust ``stress`` 收敛，再切回 C 实现完成对应 ``matrix`` / ``stress``

28. 完成 ``memory-debug`` 下的 ``matrix`` / ``stress`` 对照
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 在完成上一阶段的 harness 修复后，继续按既定顺序推进 ``memory-debug`` 下更强的
  差分证据：

  - 先重跑 Rust ``stress``
  - 再切回 C 实现重建内核与模块
  - 再执行 C ``matrix``
  - 最后执行 C ``stress``

- 本阶段实际执行的关键命令包括：

  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --implementation rust --scenario stress --rootfs-dir /home/lwz/rfl-dev/rootfs/nlmon-stress-rust-stage --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-stress-rust.cpio.gz``
  - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-stress-rust.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/memory-debug-rust-stress.log --timeout-seconds 2400``
  - ``scripts/config --file /home/lwz/rfl-dev/build-nlmon-kasan/.config -d NLMON_RUST``
  - ``make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-kasan LLVM=1 olddefconfig``
  - ``make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-kasan LLVM=1 rustavailable``
  - ``make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-kasan LLVM=1 -j$(nproc) bzImage modules``
  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --implementation c --scenario matrix --rootfs-dir /home/lwz/rfl-dev/rootfs/nlmon-matrix-c-stage --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-matrix-c.cpio.gz``
  - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-matrix-c.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/memory-debug-c-matrix.log --timeout-seconds 1200``
  - ``tools/testing/rust/nlmon/prepare-test-rootfs.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --implementation c --scenario stress --rootfs-dir /home/lwz/rfl-dev/rootfs/nlmon-stress-c-stage --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-stress-c.cpio.gz``
  - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-kasan --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-stress-c.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/memory-debug-c-stress.log --timeout-seconds 2400``

- Rust ``stress`` 的实际结果：

  - ``status=ok``
  - ``dmesg_anomaly.stress=0``
  - ``stress.pcap.bytes=416143792``
  - ``stress.decoded.lines=26691293``
  - ``1703996 packets captured``
  - 说明上一阶段“guest 空间耗尽”已经被确认是测试基础设施问题，而不是 ``nlmon`` 本体故障

- C ``matrix`` 的实际结果：

  - ``status=ok``
  - ``dmesg_anomaly.matrix.dummy=0``
  - ``dmesg_anomaly.matrix.veth=0``
  - ``dmesg_anomaly.matrix.bridge=0``
  - ``dmesg_anomaly.matrix.route_rule=0``
  - ``dmesg_anomaly.matrix.netns=0``
  - 与当前已有 Rust ``matrix`` 结果对比：

    - ``dummy`` / ``veth`` / ``bridge`` / ``route_rule`` 的 ``pcap.bytes``、
      ``decoded.lines`` 与 captured/filter 计数完全一致
    - ``netns`` 仍存在公共噪声 ``Cannot find device "nlmon_ns_veth0"``
    - ``netns`` 的摘要出现轻微漂移：

      - C：``161608`` bytes / ``10464`` lines / ``1004`` captured
      - Rust：``160716`` bytes / ``10407`` lines / ``1002`` captured

- C ``stress`` 的实际结果：

  - ``status=ok``
  - ``dmesg_anomaly.stress=0``
  - ``stress.pcap.bytes=416168736``
  - ``stress.decoded.lines=26692887``
  - ``1704052 packets captured``

- 由此可以先得到一个更强但仍保守的阶段结论：

  - ``memory-debug`` 下，C/Rust 两边现在都已经完成 ``baseline`` / ``lifecycle`` /
    ``matrix`` / ``stress``
  - 两边都没有新增的 ``dmesg`` 异常
  - ``dummy`` / ``veth`` / ``bridge`` / ``route_rule`` 四类发生器在 ``matrix`` 下已完全对齐
  - 当前剩余的主要问题已经收敛到：

    - ``netns`` 发生器的公共噪声与轻微计数漂移
    - 原始 ``pcap.sha256`` 仍不足以作为最终等价判据

- 下一步：

  - 更新 ``memory-debug`` 强化差分报告，将 ``matrix`` / ``stress`` 结果补为正式文档
  - 以独立提交记录本阶段结果
  - 然后转入 ``concurrency-debug`` 与 ``leak-debug`` 的 ``baseline`` / ``lifecycle``

29. 完成 ``concurrency-debug`` 下的 ``baseline`` / ``lifecycle``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 继续按计划转入第二套调试内核 ``concurrency-debug``，目标是先验证：

  - ``PROVE_LOCKING``
  - ``PROVE_RCU``
  - ``DEBUG_ATOMIC_SLEEP``
  - ``DEBUG_SPINLOCK``
  - ``DEBUG_NET``

  这些并发/锁语义诊断器开启后，C/Rust 两个 ``nlmon`` 实现是否仍能通过
  ``baseline`` / ``lifecycle``。

- 本阶段实际执行：

  - ``tools/testing/rust/nlmon/configure-debug-kernel.sh concurrency-debug``

    - 生成了 ``/home/lwz/rfl-dev/build-nlmon-lockdep/.config``
    - 由于脚本默认从当前主 ``build/.config`` 复制，且主构建此时带有
      ``CONFIG_NLMON_RUST=y``，因此先按 Rust 实现完成本轮

  - Rust 版：

    - 编译：

      - ``make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-lockdep LLVM=1 rustavailable``
      - ``make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-lockdep LLVM=1 -j$(nproc) bzImage modules``

    - 测试：

      - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-lockdep --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-lockdep-baseline-rust.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/concurrency-debug-rust-baseline.log --timeout-seconds 900``
      - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-lockdep --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-lockdep-lifecycle-rust.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/concurrency-debug-rust-lifecycle.log --timeout-seconds 1200``

  - C 版：

    - 切换并重建：

      - ``scripts/config --file /home/lwz/rfl-dev/build-nlmon-lockdep/.config -d NLMON_RUST``
      - ``make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-lockdep LLVM=1 olddefconfig``
      - ``make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-lockdep LLVM=1 rustavailable``
      - ``make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-lockdep LLVM=1 -j$(nproc) bzImage modules``

    - 测试：

      - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-lockdep --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-lockdep-baseline-c.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/concurrency-debug-c-baseline.log --timeout-seconds 900``
      - ``tools/testing/rust/nlmon/run-qemu-test.sh --build-dir /home/lwz/rfl-dev/build-nlmon-lockdep --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-lockdep-lifecycle-c.cpio.gz --log-file /home/lwz/rfl-dev/test-results/nlmon/concurrency-debug-c-lifecycle.log --timeout-seconds 1200``

- 实际结果摘要：

  - Rust ``baseline``：

    - ``status=ok``
    - ``dmesg_anomaly.baseline=0``
    - ``baseline.pcap.bytes=39488``
    - ``baseline.decoded.lines=2531``
    - ``160 packets captured``

  - Rust ``lifecycle``：

    - ``status=ok``
    - ``dmesg_anomaly.lifecycle=0``

  - C ``baseline``：

    - ``status=ok``
    - ``dmesg_anomaly.baseline=0``
    - ``baseline.pcap.bytes=39488``
    - ``baseline.decoded.lines=2531``
    - ``160 packets captured``

  - C ``lifecycle``：

    - ``status=ok``
    - ``dmesg_anomaly.lifecycle=0``

- 本阶段还注意到一个只出现在 Rust ``baseline`` 串口日志中的调试信息：

  - ``ip (104) used greatest stack depth: 11104 bytes left``

  这条信息没有被 ``dmesg_anomaly`` 规则判为异常，也不属于 lockdep/RCU/atomic-sleep
  告警，当前先按普通调试统计信息记录，不将其视为 ``nlmon`` 回归。

- 阶段性结论：

  - 在 ``concurrency-debug`` 下，C/Rust 两边的 ``baseline`` / ``lifecycle`` 都已经通过
  - 当前没有观测到新增的 lockdep、RCU、atomic-sleep、spinlock 或 network debug 异常
  - ``baseline`` 的关键抓包数量级在两边再次对齐

- 本阶段同时新增正式报告：

  - ``Documentation/rust/lwz-dev/diff-test-report-2026-03-18-nlmon-concurrency-debug-zh_CN.rst``

- 下一步：

  - 转入 ``leak-debug``，至少先复跑 C/Rust 的 ``baseline`` / ``lifecycle``

30. 修正 ``leak-debug`` 误报并完成 ``baseline`` / ``lifecycle``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 进入 ``leak-debug`` 后，第一次执行 Rust ``baseline`` 时出现了一个新的测试口径问题：

  - ``status=ok``
  - 但 ``dmesg_anomaly.baseline=1``
  - 命中的并不是真实泄漏，而是两条启动横幅：

    - ``kmemleak: Kernel memory leak detector initialized ...``
    - ``kmemleak: Automatic memory scanning thread started``

- 这说明之前 ``scan_dmesg()`` 中把任意 ``kmemleak`` 文本都当成异常的做法过宽了。
  对 ``leak-debug`` 而言，真正应该保留的是：

  - ``kmemleak: <count> new suspected memory leaks``
  - ``unreferenced object``

  而不是初始化横幅。

- 因此本阶段先修测试 harness：

  - ``tools/testing/rust/nlmon/nlmon-guest-runner.sh``

    - 将 ``dmesg`` 异常匹配里的 ``kmemleak`` 粗匹配
      收紧为真正疑似泄漏的输出模式

- 本阶段验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``

- 在修正口径后，重新完成 ``leak-debug`` 下的 C/Rust ``baseline`` / ``lifecycle``：

  - 生成并使用 ``/home/lwz/rfl-dev/build-nlmon-kmem``
  - 与 ``concurrency-debug`` 一样，由于初始配置继承自主 ``build/.config``，所以先跑 Rust，
    再切到 C

- Rust 版结果：

  - ``baseline``：

    - ``status=ok``
    - ``dmesg_anomaly.baseline=0``
    - ``baseline.pcap.bytes=39488``
    - ``baseline.decoded.lines=2531``
    - ``160 packets captured``

  - ``lifecycle``：

    - ``status=ok``
    - ``dmesg_anomaly.lifecycle=0``

- C 版结果：

  - ``baseline``：

    - ``status=ok``
    - ``dmesg_anomaly.baseline=0``
    - ``baseline.pcap.bytes=39488``
    - ``baseline.decoded.lines=2531``
    - ``160 packets captured``

  - ``lifecycle``：

    - ``status=ok``
    - ``dmesg_anomaly.lifecycle=0``

- 在 ``leak-debug`` 的串口日志中，C/Rust 侧都可能出现：

  - ``ip (...) used greatest stack depth: ... bytes left``

  当前仍将其视为普通调试统计信息，而不是 ``nlmon`` 回归。

- 阶段性结论：

  - ``leak-debug`` 当前至少已经不再被启动横幅误报污染
  - C/Rust 两边的 ``baseline`` / ``lifecycle`` 都已经通过
  - 当前没有观测到新的 ``kmemleak: <count> new suspected memory leaks`` 或
    ``unreferenced object`` 输出

- 本阶段同时新增正式报告：

  - ``Documentation/rust/lwz-dev/diff-test-report-2026-03-18-nlmon-leak-debug-zh_CN.rst``

- 下一步：

  - 转入 ``pcap`` 去时间戳/规范化比较
  - 然后收紧 ``netns`` 发生器公共噪声

31. 补充 ``pcap`` 去时间戳/规范化比较
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

- 为了不再只盯着原始 ``pcap.sha256``，本阶段继续增强 guest 侧观测：

  - 修改 ``tools/testing/rust/nlmon/nlmon-guest-runner.sh``
  - 在不落地整份 decoded 文本的前提下，新增：

    - ``<tag>.normalized.sha256``
    - ``<tag>.normalized.head``

  - 具体做法是对 ``tcpdump -nn -r`` 的输出按行去掉最前面的时间戳字段，再对其做哈希
    和摘要采样

- 本阶段验证命令：

  - ``busybox sh -n tools/testing/rust/nlmon/nlmon-guest-runner.sh``

- 为了得到统一口径的数据，重新补采了三套调试内核下的 C/Rust ``baseline``：

  - ``memory-debug``
  - ``concurrency-debug``
  - ``leak-debug``

- 实际结果显示出一个非常重要的现象：

  - 三个 profile 下，C/Rust 两边都满足：

    - ``baseline.pcap.bytes`` 一致
    - ``baseline.decoded.lines`` 一致
    - ``baseline.tcpdump.stderr.head`` 一致
    - ``baseline.normalized.head`` 一致
    - ``status=ok``

  - 但三组 C/Rust ``baseline.normalized.sha256`` 仍然全部不同

- 这说明：

  - 原始 ``pcap.sha256`` 的差异，确实不能简单地解读为“功能不一致”
  - 但它也不只是 pcap 外层记录时间戳那么简单
  - 即使去掉了 ``tcpdump`` 文本解码前缀中的时间戳，整个解码流的哈希仍然不同
  - 更合理的解释是：

    - 被捕获的 netlink 报文本体里仍包含运行相关的可变字段
    - 单纯“去掉外层时间戳”还不足以构成字节级规范化

- 因此本阶段得到的研究性结论是：

  - 当前可以把“只看原始 ``pcap`` 哈希”这条判据正式降级
  - 当前较稳健的已对齐指标包括：

    - ``pcap.bytes``
    - decoded 行数
    - captured/filter 计数
    - 去时间戳后的头部摘要

  - 如果后续要追求更严格的字节级/内容级等价，需要：

    - 针对 netlink 报文字段做更细粒度的语义规范化
    - 或者引入真正理解 netlink 负载结构的比较器

- 本阶段同时新增正式报告：

  - ``Documentation/rust/lwz-dev/diff-test-report-2026-03-18-nlmon-normalized-pcap-zh_CN.rst``

- 下一步：

  - 收紧 ``netns`` 事件发生器里的公共噪声
  - 然后重跑受影响的 ``matrix`` / ``stress`` 场景
