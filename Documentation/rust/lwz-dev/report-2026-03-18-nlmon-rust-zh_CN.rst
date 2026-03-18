.. SPDX-License-Identifier: GPL-2.0

2026-03-18 工作报告：nlmon Rust 化阶段总结
==========================================

摘要
----

今天的工作目标是把 ``drivers/net/nlmon.c`` 的 Rust 化计划真正落地成一个可构建、
可测试、可回滚的工程原型，而不是只停留在“流程设计”层面。以 2026-03-18
这份阶段报告对应的事实为准，最终结果是：

- 已在 ``feature/nlmon-rust`` 分支完成 ``nlmon`` 的第一版 Rust 实现。
- 原始 ``drivers/net/nlmon.c`` 完整保留，未删除、未覆盖。
- C / Rust 版本支持通过 Kconfig 切换构建。
- Rust 驱动层 ``drivers/net/nlmon_rust.rs`` 保持零 ``unsafe``。
- 所有 ``unsafe``、FFI、回调桥接、私有数据访问、结构体字段写入都被下沉到
  ``rust/kernel/net/*`` 和极小的 ``rust/helpers/net.c`` 中。
- 已在 QEMU 中完成一轮基础 C / Rust 差分测试，当前观测结果一致。
- 但 ``rust/kernel/net/*`` 中的 ``unsafe`` 仍未完成研究级契约审计，因此当前实现
  只能被称为“驱动层零 ``unsafe``、抽象层待审计的第一版原型”，不能直接宣称
  已满足工程级安全性。

如果只用一句话概括今天的进展，就是：

``nlmon`` 已经从“计划要 Rust 化”推进到了“具备 Rust 实现、并完成第一轮基础差分验证”的状态。

术语修正
--------

为避免后续读者误解，本报告统一采用下面的口径：

- DUT 始终只有 ``nlmon``。
- ``dummy``、``veth``、``bridge``、``addr/route/rule`` 与 ``netns`` 只用于制造
  netlink/rtnetlink 事件，它们是测试事件发生器，不是本轮 Rust 化目标模块。
- ``drivers/net/nlmon_rust.rs`` 的确保持零 ``unsafe``，但这并不自动推出
  ``rust/kernel/net/*`` 的安全封装已经被证明健全。
- 因此，本报告记录的是“第一版原型 + 基础验证”状态，而不是“最终工程验收通过”状态。

今日完成的工作
--------------

1. 修正执行顺序与开发基线
~~~~~~~~~~~~~~~~~~~~~~~~~~

先修正了此前日志中“helper/抽象先行”的顺序偏差，明确本项目必须遵循：

1. Kbuild 切换
2. 主 bindgen 暴露
3. bindings 缺口审计
4. helper 最小补齐
5. 安全抽象层
6. Rust 驱动层
7. 差分测试

这一步很重要，因为它直接决定后续实现是不是符合 Rust-for-Linux 的真实工程方式。

2. 建立 C / Rust 二选一构建开关
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

修改了：

- ``drivers/net/Kconfig``
- ``drivers/net/Makefile``

引入 ``CONFIG_NLMON_RUST`` 作为 Rust 实现选择器，使当前树能够在：

- ``CONFIG_NLMON_RUST=n`` 时构建原 C 版 ``nlmon.ko``
- ``CONFIG_NLMON_RUST=y`` 时构建 Rust 版 ``nlmon_rust.ko``

这样做满足了两个关键需求：

- 原始 C 版永远保留，始终可回退
- C / Rust 可以在同一基线上直接做差分测试

3. 扩展主 bindgen，完成接口可见性审计
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

修改了：

- ``rust/bindings/bindings_helper.h``
- ``rust/bindgen_parameters``

补齐了 ``nlmon`` 所需的网络头文件与常量，确认主 bindings 已经暴露：

- ``struct net_device``
- ``struct net_device_ops``
- ``struct rtnl_link_ops``
- ``struct netlink_tap``
- ``rtnl_link_register()`` / ``rtnl_link_unregister()``
- ``netlink_add_tap()`` / ``netlink_remove_tap()``
- ``dev_lstats_read()``
- ``ARPHRD_NETLINK``
- ``IFF_NO_QUEUE``
- ``IFF_NOARP``
- ``NETDEV_PCPU_STAT_LSTATS``
- ``NETDEV_TX_OK``
- ``NLMSG_GOODSIZE``
- ``NETIF_F_SG`` / ``NETIF_F_FRAGLIST`` / ``NETIF_F_HIGHDMA``

审计后确认只剩两个 helper 盲区：

- ``netdev_priv()``
- ``dev_lstats_add()``

这一步验证了一个关键事实：

当前仓库的 helper bindings 只能导出函数，不能承载新的 C 类型。因此“先主 bindgen，
后 helper”不是风格问题，而是硬约束。

4. 只为盲区补最小 helper
~~~~~~~~~~~~~~~~~~~~~~~~~

新增了：

- ``rust/helpers/net.c``

只封装了两个必要 wrapper：

- ``rust_helper_netdev_priv()``
- ``rust_helper_dev_lstats_add()``

这一步保持得比较克制，没有把业务逻辑塞进 helper，也没有引入新的驱动私有 C 结构。

5. 建立 ``rust/kernel/net`` 安全抽象层
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

新增或修改了：

- ``rust/kernel/net.rs``
- ``rust/kernel/net/skbuff.rs``
- ``rust/kernel/net/netdevice.rs``
- ``rust/kernel/net/rtnl.rs``

抽象层分成了两层推进：

第一层是基础对象：

- ``SkBuff``：拥有 ``struct sk_buff *``，``Drop`` 自动释放
- ``NetlinkTapHandle``：封装 ``netlink_add_tap/remove_tap``
- ``LStats``：封装 ``dev_lstats_add/read``
- ``LinkStats64``：封装 ``rtnl_link_stats64`` 写入

第二层是 netdev / rtnl 桥接：

- ``DeviceRef<T>``
- ``NetDevice<T>``
- ``SetupContext<T>``
- ``Driver`` trait
- ``Registration<T>``
- ``AttrTable`` / ``ExtAck`` / ``TxStatus``

这层桥接的作用，是把：

- ``net_device_ops``
- ``ethtool_ops``
- ``rtnl_link_ops``
- C ABI 回调函数

全部收敛到抽象内部，让驱动层只面对安全 Rust trait。

6. 实现零 ``unsafe`` Rust 驱动
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

新增：

- ``drivers/net/nlmon_rust.rs``

Rust 驱动层实现了：

- ``module!`` 模块元数据
- ``alias: ["rtnl-link-nlmon"]``
- 与 C 版一致的 ``setup`` 语义
- ``validate()``
- ``get_link()``
- ``open()``
- ``stop()``
- ``start_xmit()``
- ``get_stats64()``

其中最关键的一点是：

``drivers/net/nlmon_rust.rs`` 本身没有任何 ``unsafe``。

这说明当前实现满足了项目开始时设定的硬性边界：
驱动层只写业务逻辑，不直接碰裸指针、FFI、bitfield、回调表安装和私有数据原始地址。

7. 完成 C / Rust 差分测试
~~~~~~~~~~~~~~~~~~~~~~~~~~

先后在 QEMU 中跑了两轮：

- C 基线轮：``nlmon.ko``
- Rust 对照轮：``nlmon_rust.ko``

两轮都使用同一套测试思路：

- 加载事件发生器模块 ``dummy.ko``
- 加载对应 ``nlmon`` 模块
- 创建并启动 ``nlmon0``
- 通过 ``dummy0`` 制造 netlink 活动
- 用 ``tcpdump`` 在 ``nlmon0`` 上抓包
- 比较链路属性、sysfs 统计、pcap 解码结果和 ``dmesg``

差分测试结果
------------

1. 总体结论
~~~~~~~~~~~

当前 Rust 版 ``nlmon_rust`` 与原 C 版 ``nlmon`` 在本轮关键外部行为上是一致的。
这说明当前原型已经达到“功能层面对齐”的第一步，但还不能据此推出抽象层已经完成
内存安全证明。

2. 两轮一致的关键观察值
~~~~~~~~~~~~~~~~~~~~~~~~

事件发生器 ``dummy0``：

- 都是成功可用的 dummy 设备
- 都能 ``up``
- 都能成功配置 ``192.0.2.1/24``
- ``tx_packets = 1``

``nlmon0``：

- 都能 ``ip link add nlmon0 type nlmon``
- 都能 ``ip link set nlmon0 up``
- ``type = 824``，对应 ``ARPHRD_NETLINK``
- ``flags = 0x81``
- ``mtu = 3776``
- ``rx_packets = 21``
- ``rx_bytes = 30532``

抓包结果：

- C / Rust 两轮的 ``tcpdump -nn -r`` 输出都非空
- 文本总行数同为 ``494``
- 说明抓包路径、报文数量级与统计结果一致

稳定性：

- Rust 轮额外通过了连续两次 ``add/up/del nlmon0`` 的重复创建删除测试
- 严格 ``dmesg`` 检查未见 ``warning/oops/lockdep/KASAN/use-after-free``

3. 对原始验证命令的修正结论
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

你最初给出的验证序列里，有两点在当前环境下需要修正理解：

第一，``ip link add dummy0 type dummy`` 不是总能成功。

原因不是事件发生器 ``dummy`` 驱动有问题，而是当前环境中：

- ``insmod /lib/modules/dummy.ko`` 后会直接出现 ``dummy0``

因此首次再执行 ``ip link add dummy0 type dummy`` 会报：

- ``RTNETLINK answers: File exists``

但当接口被删除后，再执行该命令可以成功，说明事件发生器 ``dummy`` 本身工作正常。

第二，BusyBox 自带的 ``ip`` 不支持：

- ``ip -d link show ...``
- ``ip -s link show ...``

因此测试中改用：

- ``ip link show ...``
- ``/sys/class/net/<if>/type``
- ``/sys/class/net/<if>/flags``
- ``/sys/class/net/<if>/mtu``
- ``/sys/class/net/<if>/statistics/*``

来完成等价观测。

对 Rust-for-Linux 工程要求的符合度评估
--------------------------------------

1. 已满足的要求
~~~~~~~~~~~~~~~~

从 Rust-for-Linux 当前项目风格和已有案例看，今天的实现已经满足以下关键原型要求：

- 原始 C 驱动保留，未被删除
- Rust 与 C 通过配置切换，而不是直接替换源码
- 先 bindgen，再 helper，再抽象，再驱动，顺序正确
- helper 只处理 inline/宏盲区，范围最小化
- 驱动层零 ``unsafe``
- ``unsafe`` 下沉至 ``rust/kernel/net/*`` 与最小 helper
- 回调桥接放在抽象层，不散落在驱动文件里
- 差分测试以 C 版为金标准

这意味着当前实现已经不是“只做语法翻译的 demo”，而是一个遵循
Rust-for-Linux 分层方式构建出来的第一版工程原型。

2. 仍然存在的保守点
~~~~~~~~~~~~~~~~~~~~

虽然已经满足基本原型要求，但距离“可以放心宣称工程级安全”还有几个关键缺口：

- ``rust/kernel/net`` 新增抽象目前是“足够支撑 nlmon”的最小版，还没有经过更多
  网络 Rust 驱动复用验证
- 还没有为这些抽象补 KUnit 或更系统化的单元测试
- 还没有对 ``rust/kernel/net/*`` 中的全部 ``unsafe`` 做逐点契约审计，尤其是
  前置条件、后置条件、生命周期、别名与析构配对
- 当前差分测试主要覆盖功能路径和稳定性冒烟，还不是强化矩阵与调试内核压力测试
- Rust 版本当前产物名是 ``nlmon_rust.ko``，虽然功能上没问题，但如果未来要追求
  更强的用户态无感替换，还要进一步讨论模块命名与集成策略

所以更准确的评价是：

当前实现已经满足“研究原型 + 工程化第一版”的要求，但尚未完成研究级安全性论证。

开发过程中遇到的问题、困难与解决办法
--------------------------------------

1. 最大的技术困难：不是翻译语法，而是补齐子系统抽象
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

真正困难的部分，不是把 ``nlmon.c`` 改写成 Rust 语法，而是：

- Rust 侧原本没有现成的 ``netdev/rtnl-link`` 安全抽象

这意味着核心问题变成了：

- 先设计什么放进 ``rust/kernel/net``
- 哪些内容应该留在 helper
- 哪些能力应该直接来自主 bindings
- 如何让驱动层最终没有 ``unsafe``

解决办法是：

- 严格先做 bindings 审计
- 把 helper 限制在最小盲区
- 把所有回调桥接统一收敛到 ``Registration<T>``

这一步难，但不是不可解；它的难点更多是“子系统建模”，不是“语法转换”。

2. bindgen 相关困难
~~~~~~~~~~~~~~~~~~~

实际开发中发现：

- 不是所有 ``nlmon`` 依赖都能直接由 bindgen 完美给出
- inline 函数和复杂宏必须靠 helper
- helper bindings 不能引入新的类型，只能导出函数

这逼着实现必须先做主 bindings 设计，再决定 helper 范围。

解决办法：

- 对所有依赖做缺口表
- 明确把 ``netdev_priv``、``dev_lstats_add`` 放入 helper
- 避免“先写 helper，后想类型”的反向开发

3. 构建与环境问题
~~~~~~~~~~~~~~~~~

构建中发现：

- 不先 ``source /home/lwz/rfl-dev/env.sh``，就会误用宿主机旧版 Clang
- 某些单目标 ``make ... bindings_generated.rs`` 不会可靠刷新旧产物

解决办法：

- 全程统一走既有脚本
- 核心构建命令固定为 ``/home/lwz/rfl-dev/scripts/build-kernel.sh``

4. QEMU 测试环境问题
~~~~~~~~~~~~~~~~~~~~

测试中遇到三类环境噪声：

- BusyBox ``ip`` 功能不完整
- ``dummy0`` 的自动存在行为与“手工 add”命令冲突
- 旧的 ``rust_out_of_tree.ko`` 在 initramfs 中因为 version magic 不匹配而报错

解决办法：

- 改用 ``ip link show`` + ``sysfs`` 做等价观测
- 把 ``dummy0`` 看作“可能预存在”的对象，先检测再决定是否 add
- 明确把 ``rust_out_of_tree.ko`` 错误识别为独立噪声，不混入 ``nlmon`` 差异判断

这些问题都解决了，但也提醒我们：
未来如果要把这套流程工具化，测试环境探测必须自动化，否则会出现大量“不是驱动 bug 的环境误报”。

对未来博士课题的启发：如果做面向内核的 C2Rust 工具，应该怎么优化
--------------------------------------------------------------

今天的工作非常适合作为“面向内核驱动 Rust 化工具”的需求样本。最大启发是：

``内核 C -> Rust`` 不是单纯的源码翻译，而是一条分阶段、分语义层次的迁移流水线。

如果未来要把这个方向做成博士课题，我认为工具至少要覆盖以下能力。

1. 自动依赖盘点
~~~~~~~~~~~~~~~~

输入一个 C 驱动后，工具应先自动输出：

- 用到了哪些类型
- 哪些函数是 extern 可绑定
- 哪些函数是 inline / 宏
- 哪些字段写入涉及 bitfield / union
- 哪些回调表需要桥接

这一步如果自动化，能极大减少人工“摸清依赖”的时间。

2. 自动生成 bindgen 缺口报告
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

工具不应该直接盲目生成 Rust 代码，而应该先给出：

- 已被主 bindings 覆盖的能力
- 需要 helper 的能力
- 需要新增安全抽象的能力

也就是说，第一阶段产物应该是“迁移计划报告”，而不是“粗糙 Rust 代码”。

3. 自动生成 helper 候选
~~~~~~~~~~~~~~~~~~~~~~~

对 inline/宏盲区，工具可以自动生成候选 helper：

- 只包装最小原语
- 自动加 ``rust_helper_*`` 前缀
- 自动避免引入新类型

人工只需要做语义审查，而不是手写所有 wrapper。

4. 自动生成抽象层骨架，而不是直接生成驱动终稿
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

这是最关键的一点。

对内核驱动 Rust 化来说，更有价值的自动化产物不是直接的 Rust 驱动，而是：

- ``SkBuff`` 风格的拥有型对象
- ``Registration`` 风格的桥接器
- ``DeviceRef`` / ``NetDevice`` 风格的生命周期包装
- helper 到安全 trait 的抽象层骨架

也就是说，工具应该优先帮助开发者“搭桥”，而不是直接“翻译驱动函数体”。

5. 自动生成差分测试脚本
~~~~~~~~~~~~~~~~~~~~~~~~

工具应该把原 C 版当金标准，自动生成：

- C 轮测试脚本
- Rust 轮测试脚本
- 输出对比脚本
- ``dmesg`` 关键错误过滤规则

如果没有差分测试自动化，Rust 化很容易退化成“能编译就算完成”。

6. 自动识别环境问题
~~~~~~~~~~~~~~~~~~~

今天最典型的例子就是：

- BusyBox ``ip`` 不支持 ``-d/-s``
- ``dummy0`` 可能预存在

工具如果能在测试前先做环境探测，并自动切换到合适的观测方法，会极大减少误判。

当前仍然难以自动解决的部分
--------------------------

即使未来做工具，有一部分问题仍然很难完全自动化：

- 安全抽象的边界应如何设计
- 某个 C 结构到底该映射为拥有型、借用型还是 RAII 句柄
- 某个 callback 的生命周期和并发语义是否足够稳定
- 某个 helper 应该只是过渡 glue，还是应该进入长期抽象层

这类问题本质上不是“语法转换”，而是“内核 API 设计”和“Rust 安全模型建模”。

所以更现实的课题目标应当是：

做一个 **人机协同的 kernel C2Rust 工具链**，而不是一个试图完全自动替代内核开发者判断的黑盒翻译器。

对后续工作的建议
----------------

短期建议：

- 继续补 ``rust/kernel/net`` 抽象的可复用性与注释
- 给新增抽象补更系统的测试
- 继续增加长时间和更高负载的差分测试
- 处理 ``rootfs`` 中旧 ``rust_out_of_tree.ko`` 的版本魔数噪声

中期建议：

- 以 ``dummy`` 或其他更简单 netdev 为对照，再验证当前抽象是否通用
- 评估是否要把 ``nlmon_rust.ko`` 的集成方式进一步向“用户无感”方向收敛

课题方向建议：

- 把今天的流程抽象成“依赖盘点 -> bindings 审计 -> helper 候选 -> 抽象骨架 ->
  驱动草案 -> 差分测试脚本”的半自动流水线
- 把“抽象层生成”和“差分测试生成”作为工具的核心创新点

结论
----

今天这项工作的价值，不只是“把 ``nlmon`` 写出了一个 Rust 版本”，更重要的是：

- 证明了 ``nlmon`` 这类 rtnl-link 网络驱动可以沿着 Rust-for-Linux 的工程路线
  被真正 Rust 化
- 证明了驱动层零 ``unsafe`` 是可实现的
- 证明了 C 版金标准 + 差分测试的开发模式是有效的
- 暴露出了未来做面向内核的 C2Rust 工具时最值得攻克的几个关键环节

所以，今天的结论可以概括为：

``nlmon`` 的 Rust 化已经进入“可运行、可验证、可继续扩展”的阶段，
而这个过程本身也为后续博士课题中的内核 C2Rust 工具设计提供了非常具体的实证样本。
