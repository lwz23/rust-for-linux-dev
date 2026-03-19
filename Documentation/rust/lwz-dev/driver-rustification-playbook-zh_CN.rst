.. SPDX-License-Identifier: GPL-2.0

Linux 内核模块 Rust 化完整复现手册（以 ``nlmon`` 为例）
=======================================================

摘要
----

本文档把本轮 ``drivers/net/nlmon.c`` Rust 化过程中已经跑通的真实流程，整理成一份
可以从零复现的操作手册。它不是“自动把 C 翻译成 Rust 的教程”，而是当前
Rust-for-Linux 仓库里一条可执行、可回滚、可验证的工程路线。

本文档适用于当前长期开发环境：

- 工作区根目录：``/home/lwz/rfl-dev``
- 内核源码树：``/home/lwz/rfl-dev/linux``
- 旧目录 ``/home/lwz/linux`` 只作历史参考，不参与本流程
- 默认长期开发分支：``dev``
- 当前示例功能分支：``feature/nlmon-rust``

最终目标不是“把 C 文件改成 Rust 语法”，而是完成下面四件事：

1. 原始 C 驱动保留，并作为功能金标准。
2. Rust 驱动层保持零 ``unsafe``。
3. 所有 ``unsafe`` 下沉到 ``rust/kernel/*`` 与极小 ``rust/helpers/*``。
4. 用调试内核和差分测试证明 Rust 版不是玩具，而是当前阶段可运行、可验证、可回退的实现。

经过 ``rnull@v6.10`` 的研究校准后，本手册新增一个更严格的总目标：

5. ``blind-first`` 只算 ``prototype-grade``，后续还必须经过
   ``mainline-grade conformance hardening``，才能接近主线工程要求。

先读结论
--------

以 ``nlmon`` 为例，当前已经验证过、并经 ``rnull`` 复盘修订后的顺序是：

1. 先建立 Kbuild/Kconfig 切换，不碰原 C 文件。
2. 先扩主 bindgen，再做缺口审计。
3. 只有 bindgen 无法覆盖的 inline/宏盲区，才用 helper 补最小桥接。
4. 先写 ``rust/kernel/net/*`` 安全抽象，再写驱动层。
5. 驱动层 ``drivers/net/nlmon_rust.rs`` 保持零 ``unsafe``。
6. 先在 ``memory-debug`` 下跑强行为对照，再用 ``concurrency-debug`` /
   ``leak-debug`` 补并发与泄漏证据。
7. 再进入 ``mainline-grade conformance hardening``，按主线规则收缩对象模型与 safe API。
8. 只有完成 ``hardening`` 后，才允许写工程验收与“接近主线级”的结论。
9. 如果当前任务是研究型 benchmark，再额外做 ``reference-based evaluation``。
10. 每一步单独提交；每次提交前先更新工作日志。

先区分两条流程
--------------

从 ``rnull`` 之后，本文统一区分两类流程：

1. ``reference-free production pipeline``

- 面向真实工具场景
- 输入是只有 C 版本、没有现成 Rust 版本的驱动
- 这是未来工具真正要执行的标准流程

2. ``reference-based research calibration pipeline``

- 只在研究阶段使用
- 目标是拿一个已有官方 Rust 版本的驱动做 benchmark
- 用来校准规则、约束和验证器

因此，真实生产流程里不存在“必须和官方 Rust 版本对齐”这一步。
``rnull`` 里的 upstream 对比，属于研究校准而不是生产流程。

先明确流程等级
--------------

从 ``rnull`` 开始，任何新模块 Rust 化都必须固定分成两个阶段：

1. ``blind-first bootstrap``

- 目标是先在旧树上独立跑通：

  - Kbuild/Kconfig
  - 主 bindings
  - 缺口审计
  - 最小 helper
  - 最小抽象
  - 零 ``unsafe`` 驱动
  - 第一轮功能验证

- 这一阶段通过后，只能说明“这条路径在当前旧树上可行”。

2. ``mainline-grade conformance hardening``

- 这是未来真实工具流程中的标准第二阶段。
- 目标不是补更多功能，而是把原型期 safe API 收紧到更主线化的对象模型：

  - 默认优先 ``Pin + Opaque``
  - 显式状态机
  - builder 前置校验
  - 收缩 ``unsafe impl Send/Sync``
  - helper 退役审计

- 只有这一步完成后，才允许宣称“接近主线工程要求”。

如果当前任务是研究型 benchmark，再额外追加：

3. ``reference-based evaluation``

- 只在目标模块已经有官方 Rust 版本时使用
- 用于对照官方初始 upstream 的对象模型和 API 形态
- 它的产物是新规则、新约束和新验证器，而不是未来生产流程的新步骤

不要做的事
----------

1. 不要删除 ``drivers/net/nlmon.c``。
2. 不要在驱动目录私下生成零散 bindgen 文件。
3. 不要把大量 raw bindings 直接暴露给驱动层。
4. 不要为了省事把 ``unsafe`` 留在驱动层。
5. 不要跳过工作日志、差分测试、调试内核验证。
6. 不要把 ``blind-first`` 版本直接写成“主线级实现”。
7. 不要默认生成 ``unsafe impl Send/Sync``。
8. 不要在主 bindings 缺口还没审计清楚前先写 helper。

阶段 0：准备环境
----------------

1. 加载环境变量
~~~~~~~~~~~~~~~

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   cd /home/lwz/rfl-dev/linux

2. 确认仓库与分支
~~~~~~~~~~~~~~~~~

.. code-block:: bash

   git status --short --branch
   git remote -v

预期：

- 当前工作目录是 ``/home/lwz/rfl-dev/linux``
- ``origin`` 指向 ``git@github.com:lwz23/rust-for-linux-dev.git``
- ``upstream`` 指向 ``https://github.com/Rust-for-Linux/linux.git``

3. 创建或切换功能分支
~~~~~~~~~~~~~~~~~~~~~

.. code-block:: bash

   git checkout dev
   git pull --ff-only origin dev
   git checkout -b feature/<topic>

4. 立即创建/更新工作日志
~~~~~~~~~~~~~~~~~~~~~~~~

文档位置统一放在：

- ``Documentation/rust/lwz-dev/``

本轮示例使用：

- ``worklog-2026-03-18-zh_CN.rst``

提交纪律：

- 每一个可回滚小目标单独 commit
- 每次 commit 前先更新工作日志
- commit message 必须包含 ``Why`` / ``What`` / ``Verification``

阶段 1：先确认 DUT 与金标准
--------------------------

以 ``nlmon`` 为例，必须先统一口径：

- DUT 只有 ``drivers/net/nlmon.c``
- ``dummy``、``veth``、``bridge``、``route/rule``、``netns`` 都只是事件发生器
- 原始 C 版始终是功能金标准

如果这一步不写清楚，后续文档很容易把事件发生器误写成“第二目标模块”。

阶段 2：Kbuild / Kconfig 集成
----------------------------

目标：

- 保留 C 版
- 增加 Rust 版
- 同一时刻只编译其中一个实现

要修改的文件：

- ``drivers/net/Kconfig``
- ``drivers/net/Makefile``

推荐模式：

- 保留 ``CONFIG_NLMON``
- 新增 ``CONFIG_NLMON_RUST`` 作为选择器
- ``CONFIG_NLMON_RUST=n`` 时编译 C 版
- ``CONFIG_NLMON_RUST=y`` 时编译 Rust 版

验证命令：

.. code-block:: bash

   scripts/config --file /home/lwz/rfl-dev/build/.config -e NLMON
   scripts/config --file /home/lwz/rfl-dev/build/.config -d NLMON_RUST
   make O=/home/lwz/rfl-dev/build LLVM=1 olddefconfig

阶段 3：先扩主 bindgen，再做缺口审计
------------------------------------

核心原则：

- 先主 bindings
- 后 helper
- 最后才是抽象与驱动

要改的主入口通常是：

- ``rust/bindings/bindings_helper.h``
- 必要时 ``rust/uapi/uapi_helper.h``

``nlmon`` 实例中至少补了：

- ``linux/netdevice.h``
- ``linux/netlink.h``
- ``net/rtnetlink.h``
- ``linux/if_arp.h``

统一生成方式：

.. code-block:: bash

   make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build LLVM=1 rustavailable
   make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build LLVM=1 -j"$(nproc)" bzImage modules

然后审计主 bindings 是否已经暴露：

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
- ``NLMSG_GOODSIZE``

经验规则：

- 如果主 bindings 能给，就不要写 helper
- helper 只服务于 bindgen 无法直接表达的 inline/宏盲区

阶段 4：只为盲区补最小 helper
-----------------------------

``nlmon`` 实例里最后只保留了两个 helper：

- ``rust_helper_netdev_priv()``
- ``rust_helper_dev_lstats_add()``

文件位置：

- ``rust/helpers/net.c``

helper 设计规则：

1. 只导出 ``rust_helper_*`` 函数。
2. 不定义长期对外的驱动私有 C 结构。
3. 不承载业务逻辑。
4. 语义边界仍归属于 ``rust/kernel/*`` 抽象，而不是驱动层。

阶段 5：在 ``rust/kernel/*`` 里建立安全抽象
------------------------------------------

``nlmon`` 这一例子的正确拆法是：

1. 先做基础对象
~~~~~~~~~~~~~~~~

- ``rust/kernel/net/skbuff.rs``
- ``rust/kernel/net/netdevice.rs`` 中的统计对象和 tap 句柄

至少需要：

- ``SkBuff``
- ``DeviceRef``
- ``NetDevice``
- ``SetupContext``
- ``NetlinkTapHandle``
- ``LStatsHandle``
- ``LinkStats64``

2. 再做回调桥接
~~~~~~~~~~~~~~~

- ``rust/kernel/net/rtnl.rs``

至少需要：

- ``Driver`` trait
- ``AttrTable``
- ``ExtAck``
- ``TxStatus``
- ``Registration<T>``
- ``net_device_ops`` / ``ethtool_ops`` / ``rtnl_link_ops`` 到 Rust trait 的桥接

硬规则：

- 所有 ``unsafe`` 都必须封在这些抽象里
- 每个 ``unsafe`` 点都要写局部不变式注释
- 驱动层只拿能力型接口，不拿近似裸 ``net_device`` 访问权
- 只要 ``Private`` 里会放 registered / intrusive / callback-owned pinned 对象，
  就禁止再给 safe 驱动公开宽泛 ``&mut Private``；必须改成
  ``in-place pinned init + pinned access``

阶段 6：先做 ``unsafe`` 审计，再写最终结论
------------------------------------------

抽象能编译后，不要立刻宣布“已经安全”。

必须单独输出审计文档，至少检查：

1. 原始指针来源
2. 前置条件
3. 后置条件
4. 生命周期边界
5. 所有权转移
6. 别名/可变性约束
7. 失败回滚路径
8. safe API 是否会把证明责任外推给驱动层

``nlmon`` 实例中的一个重要经验是：

- 旧版审计文档会很快过时
- 一旦抽象层收紧，就要刷新审计文档
- 否则工程验收结论会直接和源码现实冲突

阶段 7：最后才写零 ``unsafe`` 驱动层
-----------------------------------

``nlmon`` 驱动层文件：

- ``drivers/net/nlmon_rust.rs``

当前驱动层只负责：

- ``module!`` 元数据
- 驱动私有状态
- ``setup/open/stop/start_xmit/get_stats64/validate/get_link`` 业务语义

当前驱动层不负责：

- raw bindings 细节
- 设备字段裸写
- 回调桥接
- 裸指针操作
- ``unsafe``

对照检查：

.. code-block:: bash

   rg -n "unsafe" drivers/net/nlmon_rust.rs

预期为空。

抽象层附加硬规则
~~~~~~~~~~~~~~~~

经过 ``rnull`` 验证后，新增三条跨子系统默认约束：

1. 对 intrusive / registered / callback-owned C 对象，默认优先：

   - ``Pin + Opaque``
   - 显式状态机
   - builder 前置校验

2. 如果某个 safe API 仍然要求驱动自己记住生命周期、唯一可变性或析构配对，它就还只是
   原型 API，不应直接进入最终抽象。

3. ``blind-first`` 阶段允许存在临时聚合对象；进入 ``hardening`` 阶段后，必须检查这些
   对象是否把多个生命周期揉得过宽，并在必要时拆解。

4. 只要驱动私有区里承载 registered / intrusive / callback-owned pinned 对象，就不能再
   保留宽泛 ``&mut Private`` 出口；必须：

   - 把私有区初始化改成 ``in-place pinned init``
   - 把可变私有访问改成 ``Pin<&mut Private>``
   - 必要时补 current-device capability，把“只能绑定当前设备自身”编码进 API

阶段 8：构建三套调试内核
------------------------

本轮固定使用三套 build 输出，不混用：

- ``/home/lwz/rfl-dev/build-nlmon-kasan``
- ``/home/lwz/rfl-dev/build-nlmon-lockdep``
- ``/home/lwz/rfl-dev/build-nlmon-kmem``

对应脚本：

- ``tools/testing/rust/nlmon/configure-debug-kernel.sh``

用法：

.. code-block:: bash

   tools/testing/rust/nlmon/configure-debug-kernel.sh memory-debug
   tools/testing/rust/nlmon/configure-debug-kernel.sh concurrency-debug
   tools/testing/rust/nlmon/configure-debug-kernel.sh leak-debug

然后在对应输出目录构建：

.. code-block:: bash

   make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-kasan LLVM=1 rustavailable
   make -C /home/lwz/rfl-dev/linux O=/home/lwz/rfl-dev/build-nlmon-kasan LLVM=1 -j"$(nproc)" bzImage modules

其他两个 profile 同理。

阶段 9：准备 rootfs 与自动化测试脚本
----------------------------------

本轮不靠手工敲命令，而是统一用脚本驱动：

- ``tools/testing/rust/nlmon/prepare-test-rootfs.sh``
- ``tools/testing/rust/nlmon/nlmon-guest-runner.sh``
- ``tools/testing/rust/nlmon/run-qemu-test.sh``

1. 准备 rootfs
~~~~~~~~~~~~~~

示例：

.. code-block:: bash

   tools/testing/rust/nlmon/prepare-test-rootfs.sh \
     --build-dir /home/lwz/rfl-dev/build-nlmon-kasan \
     --implementation rust \
     --scenario baseline \
     --rootfs-dir /home/lwz/rfl-dev/rootfs/nlmon-baseline-rust-stage \
     --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-baseline-rust.cpio.gz

2. 运行 QEMU 自动测试
~~~~~~~~~~~~~~~~~~~~~

示例：

.. code-block:: bash

   tools/testing/rust/nlmon/run-qemu-test.sh \
     --build-dir /home/lwz/rfl-dev/build-nlmon-kasan \
     --rootfs-image /home/lwz/rfl-dev/rootfs/initramfs-nlmon-baseline-rust.cpio.gz \
     --log-file /home/lwz/rfl-dev/test-results/nlmon/memory-debug-rust-baseline.log \
     --timeout-seconds 900

日志输出目录统一放在：

- ``/home/lwz/rfl-dev/test-results/nlmon/``

阶段 10：测试顺序固定
--------------------

本轮已经证明有效的顺序是：

1. ``memory-debug``：

   - ``baseline``
   - ``lifecycle``
   - ``matrix``
   - ``stress``

2. ``concurrency-debug``：

   - 至少先跑 ``baseline`` / ``lifecycle``

3. ``leak-debug``：

   - 至少先跑 ``baseline`` / ``lifecycle``

为什么这样排：

- ``memory-debug`` 最适合先把功能和生命周期行为拉齐
- ``concurrency-debug`` 用来补锁/RCU/atomic-sleep 证据
- ``leak-debug`` 用来补 ``kmemleak`` 口径

注意：

- 上面这组测试顺序主要服务于 ``blind-first`` 的可运行验证与调试内核证据。
- 对真实生产流程而言，测试通过后还必须回到抽象层做
  ``mainline-grade conformance hardening``，然后刷新审计和关键测试结论。
- 如果当前任务是研究型 benchmark，再在 hardening 之后追加
  ``reference-based evaluation``。

阶段 11：差分测试判读规则
------------------------

优先看这些指标：

- ``status=ok``
- ``dmesg_anomaly=0``
- ``pcap.bytes``
- decoded 行数
- ``tcpdump`` captured/filter 计数
- ``normalized.head``

不要把下面这个判据当成唯一结论：

- 原始 ``pcap.sha256``

因为 ``nlmon`` 实例已经证明：

- 原始哈希不同，不代表功能不等价
- 去掉外层时间戳后，``normalized.sha256`` 仍可能不同
- netlink 负载本身带运行相关字段

因此更合理的口径是：

- baseline / lifecycle / matrix 用来判断行为是否收敛
- stress 用来判断长时稳定性与“无新增调试异常”

阶段 12：`netns` 发生器经验教训
------------------------------

本轮一个很实际的坑是：

- ``gen_netns()`` 在 ``ip netns del`` 之后又删了一次外部 ``veth``
- 这会制造 ``Cannot find device "nlmon_ns_veth0"`` 公共噪声
- 也会额外引入失败型 netlink 事件，污染 ``matrix`` / ``stress`` 摘要

经验结论：

- 测试发生器的公共噪声必须当真修复，不能只在报告里“解释过去”
- 差分测试框架本身也是工程对象，要单独调试和提交

阶段 13：如何写工程验收结论
--------------------------

工程验收文档必须明确区分三层结论：

1. DUT 本身是否通过当前项目阶段验收
2. 当前抽象层是否足以支撑这个 DUT
3. 这套抽象是否已经能被夸成“通用成熟子系统抽象”

``nlmon`` 的最终写法是：

- ``nlmon_rust`` 通过当前阶段工程验收
- 当前抽象足以支撑 ``nlmon`` 用例
- 通用化/上游化结论暂不做过度承诺

这种写法既不保守到否认成果，也不夸大到越过证据边界。

``rnull`` 之后新增一条强制前提：

- 如果还没有完成 ``mainline-grade conformance hardening``，就不要写工程验收。
- 在那之前，最准确的表述只能是“原型已跑通并完成基础验证”。

阶段 14：提交与 push 规则
------------------------

推荐提交切分：

1. Kbuild 切换
2. 主 bindgen 暴露
3. helper 最小补齐
4. 基础抽象
5. 桥接抽象
6. 驱动层骨架
7. 驱动行为补齐
8. ``unsafe`` 审计
9. 调试内核测试基础设施
10. 差分测试增强
11. 工程验收文档
12. 完整复现手册

如果当前任务属于研究型 benchmark，且存在官方初始 upstream Rust 参考实现，
还要再插入两步：

13. ``reference-based evaluation`` 与差异总账
14. 基于对比结论刷新规则与审计

每次提交前必须：

- 更新工作日志
- 确保当前步骤可回滚
- 写清 ``Why`` / ``What`` / ``Verification``

最后 push：

.. code-block:: bash

   git push origin feature/<topic>

复现完成检查表
--------------

如果后续你要拿这套流程去做第二个模块，至少要确认下面这些都具备：

- 原始 C 驱动保留
- Rust 驱动层零 ``unsafe``
- 主 bindgen 先补齐
- helper 只补盲区
- ``unsafe`` 审计文档存在且与当前源码一致
- 三套调试 profile 至少有基础对照结果
- ``memory-debug`` 下有更强的行为/稳定性证据
- 已经完成 ``mainline-grade conformance hardening``
- 如果当前任务属于研究型 benchmark，已经完成 ``reference-based evaluation``
- 差分报告存在
- 工程验收文档存在
- 完整复现手册存在
- 工作日志完整

如果上面这些还没齐，就不要急着把项目写成“彻底完成”。
