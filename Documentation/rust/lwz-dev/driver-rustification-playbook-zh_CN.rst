.. SPDX-License-Identifier: GPL-2.0

Linux 内核模块 Rust 化流程指导手册
=================================

摘要
----

本文档总结了当前工作区 ``/home/lwz/rfl-dev`` 中，围绕 ``drivers/net/nlmon.c``
完成 Rust 化原型、抽象层收缩、调试内核验证和差分测试时形成的一套可复现流程。

它的目标不是给出“自动把 C 驱动翻译成 Rust”的工具，而是提供一条符合当前
Rust-for-Linux 工程实践的人工迁移路线，帮助后续把更复杂的内核模块逐步 Rust 化。

本文档适用于当前仓库：

- 源码树：``/home/lwz/rfl-dev/linux``
- 默认长期开发分支：``dev``
- 当前功能分支示例：``feature/nlmon-rust``
- 远端：

  - ``origin = git@github.com:lwz23/rust-for-linux-dev.git``
  - ``upstream = https://github.com/Rust-for-Linux/linux.git``

核心原则
--------

在当前工作区中，任何内核模块 Rust 化都必须遵守以下硬约束：

1. 原始 C 实现永远保留，不删除、不覆盖。
2. Rust 驱动层文件必须保持零 ``unsafe``。
3. 所有 ``unsafe``、裸指针、FFI、回调桥接、结构体字段写入，都必须下沉到
   ``rust/kernel/*`` 或极小的 ``rust/helpers/*``。
4. helper 只处理 bindgen 无法自动生成的 inline/宏盲区，不承载长期业务逻辑。
5. bindgen 统一走内核主构建流程，不为单个驱动私下生成零散绑定文件。
6. 差分测试以原始 C 版为金标准。
7. 每个阶段单独提交，每次提交前先更新工作日志。
8. 任何阶段报告都必须区分：

   - 已被证据支持的结论
   - 仍未被证据支持的结论

推荐目录与文档落点
------------------

建议所有与 Rust 化相关的中文文档统一放在：

- ``Documentation/rust/lwz-dev/``

推荐至少维护以下几类文档：

- 工作日志：``worklog-YYYY-MM-DD-zh_CN.rst``
- 阶段报告：``report-YYYY-MM-DD-<topic>-zh_CN.rst``
- ``unsafe`` 审计：``unsafe-audit-YYYY-MM-DD-<topic>-zh_CN.rst``
- 差分测试报告：``diff-test-report-YYYY-MM-DD-<topic>-zh_CN.rst``
- 流程手册：本文档

推荐总流程
----------

1. 建立开发分支与日志基线
~~~~~~~~~~~~~~~~~~~~~~~~~

先从 ``dev`` 切出功能分支，并立即创建工作日志入口。

示例：

.. code-block:: bash

   source /home/lwz/rfl-dev/env.sh
   git -C "$KERNEL_SRC" checkout dev
   git -C "$KERNEL_SRC" pull --ff-only origin dev
   git -C "$KERNEL_SRC" checkout -b feature/<topic>

此时应先记录：

- 分支名
- 最近提交
- 当前目标模块
- 当前尚未开始的阶段

不要一开始就直接写抽象层或 helper。

2. 明确迁移目标与金标准
~~~~~~~~~~~~~~~~~~~~~~~

在开始编码前，先确定：

- DUT 只有哪个模块
- 哪些其他模块/命令只是事件发生器
- 哪个 C 文件是金标准
- 外部行为用什么命令验证

以 ``nlmon`` 为例：

- DUT：``drivers/net/nlmon.c``
- 事件发生器：``dummy``、``veth``、``bridge``、``route/rule``、``netns``

这一点必须写进日志和报告，避免后续把事件发生器误写成“第二目标模块”。

3. 先做 Kbuild/Kconfig 集成
~~~~~~~~~~~~~~~~~~~~~~~~~~~~

在写 Rust 驱动前，先让构建系统知道将来会存在 Rust 版本。

推荐做法：

- 保留原始 ``CONFIG_<DRIVER>``
- 增加 ``CONFIG_<DRIVER>_RUST`` 作为选择器
- 在 ``Makefile`` 中只编译 C 或 Rust 之一

这样可以保证：

- 原始 C 版始终可用
- Rust 版可以独立切换测试
- 不会同时注册两个同名设备实现

4. 先扩主 bindgen，再做缺口审计
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

这是最重要的顺序约束之一。

必须先修改：

- ``rust/bindings/bindings_helper.h``
- 必要时 ``rust/uapi/uapi_helper.h``

然后通过 Kbuild 触发统一 bindgen 生成，再审计缺口。

不推荐：

- 在驱动目录私下跑独立 bindgen
- 为单个模块生成零散的绑定文件

审计时要把依赖项分成三类：

- 主 bindings 已满足
- 需要 helper
- 需要新增安全抽象

5. 只为 bindgen 盲区补最小 helper
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

helper 只处理这类对象：

- ``static inline`` 函数
- 宏展开后的语义包装
- bindgen 规则无法直接导出的访问入口

典型例子：

- ``netdev_priv()``
- ``dev_lstats_add()``

helper 的设计规则：

- 只导出 ``rust_helper_*`` 函数
- 不新定义驱动私有 C 结构作为长期接口
- 不承载业务逻辑
- 语义归属于 ``rust/kernel/*`` 抽象，而不是驱动层

6. 在 ``rust/kernel/*`` 中建立最小安全抽象
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

正确做法不是“把 C 代码逐行翻成 Rust 语法”，而是先把驱动所依赖的内核能力抽象出来。

推荐顺序：

1. 基础资源对象

   - 例如 ``SkBuff``、统计对象、tap 句柄

2. 子系统桥接抽象

   - 例如 ``net_device_ops``、``rtnl_link_ops``、注册生命周期

3. 只保留驱动真正需要的能力型接口

   - 避免把“接近裸 ``struct net_device``”的能力直接暴露给驱动层

当前从 ``nlmon`` 经验总结出的关键要求：

- ``DeviceRef`` 这类借用对象必须带明确生命周期
- ``SetupContext`` 只负责 setup 阶段，不应泄漏运行期能力
- 私有数据访问要尽量收缩到回调上下文内部
- ``Registration`` 需要形成封闭的构造/注册/注销/析构模型

7. 对全部 ``unsafe`` 做契约审计
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

当抽象层初版能编译后，不应立刻宣称“已经安全”，而应先建立 ``unsafe`` 审计文档。

每个 ``unsafe`` 点至少要回答：

- 原始指针从哪里来
- 前置条件是什么
- 后置条件是什么
- 所有权如何转移
- 生命周期到哪里结束
- 别名/可变性约束是什么
- 是否依赖锁或特定上下文
- 失败路径如何回滚
- safe API 调用者能否破坏这些不变式

如果某个 safe API 无法给出完整契约，应优先收缩 API，而不是把证明责任外推给驱动层。

8. 最后才写零 ``unsafe`` 驱动层
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

只有在抽象层足够后，才开始写：

- ``drivers/<subsystem>/<driver>_rust.rs``

驱动层应只做：

- 模块元数据
- 私有状态定义
- 业务语义实现
- 与 C 金标准的一一对应

驱动层不应做：

- 直接操作 ``bindings::*`` 完成核心逻辑
- 直接写 ``struct net_device`` 字段
- 出现任何 ``unsafe``

9. 建调试内核，不只用普通构建
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

普通构建只能证明“能跑”，不能证明“调试器没发现明显问题”。

推荐至少维护三套独立输出目录：

- ``build-nlmon-kasan``
- ``build-nlmon-lockdep``
- ``build-nlmon-kmem``

推荐分别覆盖：

- memory debug：``KASAN``、``SLUB_DEBUG_ON``、``DEBUG_OBJECTS``
- concurrency debug：``PROVE_LOCKING``、``PROVE_RCU``、``DEBUG_ATOMIC_SLEEP``
- leak debug：``DEBUG_KMEMLEAK``

注意事项：

- 不建议把所有调试器都塞进同一个构建
- 每套构建的结果都要单独记录

10. 差分测试从弱到强逐步推进
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

推荐顺序：

1. ``baseline``
2. ``lifecycle``
3. ``matrix``
4. ``stress``

每一步都保持：

- 同一套测试脚本
- 只切换 C / Rust 实现
- 先看主机侧可比摘要，再决定是否深入原始产物

当前 ``nlmon`` 经验中，``baseline`` 至少要比较：

- ``status=ok``
- ``dmesg_anomaly``
- ``observation_mode.baseline.nlmon0``
- ``pcap`` 字节数
- decoded 行数
- ``tcpdump`` 的 captured/filter 计数

如果原始 ``pcap`` 哈希不同，但：

- 字节数一致
- decoded 行数一致
- captured/filter 计数一致

应先把它视为“待解释差异”，而不是直接宣称功能不等价。

11. 常见陷阱与规避建议
~~~~~~~~~~~~~~~~~~~~~~~

本次 ``nlmon`` Rust 化中，已经踩过以下坑：

1. BusyBox ``ip`` 污染测试结果

   - 现象：``ip netns``/``ip -d`` 等能力缺失
   - 解决：rootfs 中显式分发完整 ``iproute2``，并优先使用该路径

2. rootfs 复制宿主机软链接导致 guest 内悬空

   - 现象：``/usr/sbin/ip -> /bin/ip``，但 guest 里没有对应真实文件
   - 解决：staging 时复制 ``readlink -f`` 解析后的真实文件

3. ``tcpdump`` 默认降权用户不存在

   - 现象：``Couldn't find user 'tcpdump'``
   - 解决：在最小 rootfs 里补 ``/etc/passwd`` 与 ``/etc/group`` 条目

4. ``tcpdump`` 缓冲与收尾时序不稳定

   - 现象：``received by filter`` 非 0，但 ``captured`` 为 0
   - 解决：使用 ``-U``，并在发送 ``SIGINT`` 前给短暂 drain 窗口

5. 同一 build 目录在 C/Rust 间来回切换导致 ``vermagic`` 污染

   - 现象：旧 ``.ko`` 与新 ``bzImage`` 不匹配
   - 解决：每次切实现后完整重建，并明确记录当前 build 目录对应哪种实现

12. 建议的提交切分
~~~~~~~~~~~~~~~~~~

推荐每步一个提交，类似：

1. 日志与术语修正
2. Kbuild 切换
3. 主 bindgen 暴露
4. helper 最小补齐
5. 基础抽象
6. 桥接抽象
7. Rust 驱动骨架
8. Rust 驱动行为补齐
9. ``unsafe`` 审计
10. 调试内核测试基础设施
11. 差分测试增强
12. 阶段报告
13. 流程手册

每个提交正文建议包含：

- ``Why``
- ``What``
- ``Verification``

13. 当前阶段后，下一类模块怎么选
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

如果下一步要继续做更复杂模块，建议按以下顺序加难度：

1. 同一子系统中接口相近、状态机稍复杂一些的模块
2. 已有部分 Rust 抽象可复用的模块
3. 需要新增少量子系统抽象、但生命周期清晰的模块
4. 最后再碰高度并发、强锁耦合、复杂内存所有权的模块

原因很简单：

- 当前最昂贵的工作不在“写 Rust 语法”，而在“建立并证明抽象边界”
- 先复用已有抽象，可以把研究重点逐渐收敛到更难的问题上

交付检查表
----------

当一个模块 Rust 化完成到可阶段验收时，至少应满足：

- 原始 C 版保留
- Rust 驱动层零 ``unsafe``
- ``unsafe`` 审计文档存在
- baseline / lifecycle 差分已完成
- 调试内核下无新增异常
- 工作日志完整
- 阶段报告完整
- 流程文档已更新

如果这些条件还没全部满足，就不应把项目写成“最终完成”。
