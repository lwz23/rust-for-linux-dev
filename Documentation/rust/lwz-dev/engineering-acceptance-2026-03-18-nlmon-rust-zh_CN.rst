.. SPDX-License-Identifier: GPL-2.0

2026-03-18 nlmon Rust 化工程验收结论（2026-03-19 hardening 刷新版）
==================================================================

摘要
----

本文档给出 ``nlmon`` Rust 化第二阶段的工程验收结论，并在 ``2026-03-19`` 的
``nlmon`` hardening 完成后做了刷新。这里的“工程验收”不是问
“是否已经形式化证明绝对安全”，而是问：

- 当前分支里的 ``nlmon_rust`` 是否已经满足本项目定义的阶段性工程要求；
- 当前证据是否足以支持“它不是玩具原型，而是具备可重复构建、可切换、可调试、可差分验证的
  Rust 版本”；
- 哪些结论已经成立，哪些还不能过度延伸。

本轮最终结论分三层给出：

1. 对 ``nlmon`` 这个具体 DUT 而言：``通过当前阶段工程验收``。
2. 对支撑它的当前 ``rust/kernel/net/*`` 抽象而言：``已足以支撑 nlmon 用例通过验收``。
3. 对“把这套抽象直接视为通用 netdev/rtnl Rust 抽象并立即上游化”而言：``暂不做此结论``。

也就是说，当前可以说：

- 这版 ``nlmon_rust`` 已经符合本项目当前阶段的工程目标；
- 不能说它已经自动等价于“通用网络驱动 Rust 抽象已经定型”。

证据来源
--------

本结论综合以下材料：

- ``report-2026-03-18-nlmon-rust-zh_CN.rst``
- ``diff-test-report-2026-03-18-nlmon-memory-debug-zh_CN.rst``
- ``diff-test-report-2026-03-18-nlmon-concurrency-debug-zh_CN.rst``
- ``diff-test-report-2026-03-18-nlmon-leak-debug-zh_CN.rst``
- ``diff-test-report-2026-03-18-nlmon-normalized-pcap-zh_CN.rst``
- ``nlmon-hardening-ledger-2026-03-19-zh_CN.rst``
- ``unsafe-audit-2026-03-18-nlmon-rust-zh_CN.rst``
- ``worklog-2026-03-18-zh_CN.rst``

硬性要求核对
------------

本项目起初明确的硬性要求，当前满足情况如下：

1. ``drivers/net/nlmon.c`` 永远保留

   - 已满足。
   - 当前树里原始 C 实现完整保留，未被删除、未被覆盖。

2. Rust 驱动层零 ``unsafe``

   - 已满足。
   - ``drivers/net/nlmon_rust.rs`` 当前不包含 ``unsafe``。

3. 所有 ``unsafe`` 下沉到抽象层/极小 helper

   - 已满足。
   - ``unsafe`` 集中在 ``rust/kernel/net/*`` 与 ``rust/helpers/net.c``。

4. C / Rust 可切换构建

   - 已满足。
   - ``CONFIG_NLMON_RUST`` 已实现单实现切换，避免同名实现并存注册。

5. 以 C 版为金标准做差分测试

   - 已满足。
   - 所有自动化验证都按“同一测试框架下切 C / Rust 实现”的方式执行。

6. 需要有工作日志、报告、可回滚提交

   - 已满足。
   - 当前分支已按阶段提交，并同步更新中文日志与报告。

行为等价证据
------------

1. ``baseline`` 已在三套调试 profile 下收敛
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

以下 profile 中，C/Rust ``baseline`` 都成功通过：

- ``memory-debug``
- ``concurrency-debug``
- ``leak-debug``

截至 ``2026-03-19`` hardening refresh，本轮又重跑了：

- Rust：

  - ``memory-debug`` ``baseline`` / ``lifecycle`` / ``matrix``
  - ``concurrency-debug`` ``baseline`` / ``lifecycle``
  - ``leak-debug`` ``baseline`` / ``lifecycle``

- C：

  - ``memory-debug`` ``baseline`` / ``matrix``

本轮新的共同基线指标是：

- ``status=ok``
- ``dmesg_anomaly.baseline=0``
- ``baseline.pcap.bytes=38244``
- ``baseline.decoded.lines=2451``
- ``154 packets captured``

这里要把日期说清楚：

- ``2026-03-18`` 历史 ``concurrency-debug`` / ``leak-debug`` baseline 曾记录
  ``39488`` bytes / ``2531`` lines / ``160`` packets；
- ``2026-03-19`` hardening refresh 中，C 与 Rust 的同日 baseline 同步收敛为
  ``38244`` bytes / ``2451`` lines / ``154`` packets。

因为这种变化在同日 C/Rust 对照里同步出现，所以本轮把它归类为：

- ``可接受的同日环境/运行漂移``
- 而不是 ``Rust 特有功能差异``

2. ``lifecycle`` 已在三套调试 profile 下收敛
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

以下 profile 中，C/Rust ``lifecycle`` 都成功通过：

- ``memory-debug``
- ``concurrency-debug``
- ``leak-debug``

共同结论为：

- ``status=ok``
- ``dmesg_anomaly.lifecycle=0``
- 连续 100 轮 ``add/up/del nlmon0`` 未触发新增异常

这说明当前 Rust 版至少已经通过了“不是一次性幸运启动”的生命周期重复验证。

3. ``memory-debug`` 下的 ``matrix`` 已形成强行为证据
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``memory-debug`` 是本轮最强的行为对照 profile。当前 C/Rust 两边都已经完成：

- ``dummy``
- ``veth``
- ``bridge``
- ``route_rule``
- ``netns``

五类事件发生器各 50 轮的 ``matrix`` 对照。

其中最关键的更新是：

- ``netns`` 发生器的公共噪声已经被修复；
- 修复后，C/Rust 两边的 ``matrix.netns`` 摘要已经对齐：

  - ``142224`` bytes
  - ``9200`` decoded lines
  - ``800`` packets captured
  - ``dmesg_anomaly.matrix.netns=0``

因此，当前 ``memory-debug`` ``matrix`` 可以作为“当前 DUT 外部行为已高度收敛”的
强证据，而不再只是“除了 netns 之外都差不多”。

4. ``memory-debug`` 下的 ``stress`` 证明了稳定性
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

``memory-debug`` 下的 C/Rust ``stress`` 都完成了默认 ``1800`` 秒长时运行：

- Rust：

  - ``status=ok``
  - ``dmesg_anomaly.stress=0``
  - ``419044104`` bytes
  - ``26872764`` decoded lines
  - ``1688456`` captured

- C：

  - ``status=ok``
  - ``dmesg_anomaly.stress=0``
  - ``425312184`` bytes
  - ``27274728`` decoded lines
  - ``1713712`` captured

这里的结论边界要说清楚：

- ``stress`` 现在足以证明“长时运行稳定、无新增调试异常、抓包路径持续有效”；
- ``stress`` 不是逐字节等价证明，因为事件交错顺序和报文本体本身都带有运行相关字段。

5. ``pcap`` 哈希差异已被正确降级解释
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

当前证据已经明确说明：

- 原始 ``pcap.sha256`` 不同，不能直接解读为功能不等价；
- 即使去掉 ``tcpdump`` 文本前缀中的时间戳，``normalized.sha256`` 仍然不同；
- 但在多个 profile 的 baseline 中，以下指标又是稳定一致的：

  - ``pcap.bytes``
  - decoded 行数
  - ``tcpdump`` captured/filter 计数
  - ``normalized.head``

因此，本轮工程验收采纳的口径是：

- 原始或简单规范化 ``sha256`` 只保留为研究现象；
- 行为等价以多维摘要和调试内核稳定性为主，不再被单一哈希绑架。

安全性与抽象层证据
------------------

1. 驱动层安全边界成立
~~~~~~~~~~~~~~~~~~~~~

当前 ``nlmon_rust`` 驱动层完全通过安全抽象编写，具备以下特征：

- 无 ``unsafe``
- 不直接操作 raw bindings 完成核心业务
- 不直接操作 ``struct net_device`` 字段
- 不直接桥接 C ABI 回调

这与 Rust-for-Linux 现有“驱动层尽量纯安全 Rust、把 ``unsafe`` 封在
``rust/kernel/*``”的工程方向一致。

2. 抽象层当前已足以支撑 ``nlmon``
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

更新后的 ``unsafe`` 审计已经表明：

- 早期最明显的 safe API 过宽问题已经被修正；
- 当前没有再发现“驱动层不写 ``unsafe`` 也能轻易破坏抽象边界”的明显路径；
- ``DeviceRef`` 生命周期、``SetupContext`` 阶段边界、``SkBuff`` 所有权模型、
  ``AttrTable`` 边界检查都比第一版清晰得多。

结合调试内核结果，目前可以成立的判断是：

- 对当前 ``nlmon`` 用例，这套抽象层已经足以支撑工程验收；
- 并且 ``2026-03-19`` hardening 已把
  “已注册 ``netlink_tap`` 不能再被 safe code 移动”编码进了类型边界；
- 但它仍然更接近“面向当前用例收紧后的最小抽象”，而不是“已经定型的通用网络抽象”。

3. 仍保留的工程边界
~~~~~~~~~~~~~~~~~~~

以下点仍应被视为保留项，而不是被忽略：

- ``Registration<T>`` 的 ``Send/Sync`` 依赖当前注册对象契约与注释说明；
- vtable 的零初始化尾部仍依赖当前内核 ABI 语义；
- ``rust/helpers/net.c`` 仍是当前旧树里两个入口的实际绑定来源，还要单独做 helper 退役审计；
- 当前尚未引入“理解 netlink 负载语义的规范化比较器”，因此没有逐字段语义比较。

这些保留项不会否定本轮 ``nlmon`` 验收，但会限制我们对“通用化/上游化程度”的表述。

最终验收结论
------------

本轮正式结论如下：

1. ``nlmon_rust`` 通过当前阶段工程验收
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

理由：

- C 版保留且可随时切回；
- Rust 驱动层零 ``unsafe``；
- ``unsafe`` 已被限制在抽象层与极小 helper；
- ``2026-03-19`` hardening refresh 规定的
  ``memory-debug baseline/lifecycle/matrix``、
  ``concurrency-debug baseline/lifecycle``、
  ``leak-debug baseline/lifecycle`` 均已通过；
- 当 ``2026-03-19`` 的 ``memory-debug baseline`` 摘要相对 ``2026-03-18`` 历史日志发生变化时，
  同日 C ``baseline/matrix`` 已补跑并证明这不是 Rust 特有差异；
- ``memory-debug`` 下的 ``matrix`` / ``stress`` 已补强，并且 ``netns`` 公共噪声已清除；
- 当前没有观测到新增的 KASAN、lockdep、RCU、``kmemleak``、double-free、UAF 等异常；
- 中文日志、差分报告、审计报告、流程手册都已纳入仓库。

因此，对本项目定义的“研究级可运行、可验证、可回退的 Rust 版 ``nlmon``”目标而言，
当前结果已经达标。

2. 不采纳的过度结论
~~~~~~~~~~~~~~~~~~~

本轮明确不采纳以下说法：

- “已经得到逐字节完全一致证明”
- “已经完成形式化内存安全证明”
- “当前 ``rust/kernel/net/*`` 已经可以直接视为成熟通用网络子系统抽象”

3. 对 Rust-for-Linux 工程要求的回答
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

如果按你最关心的两个口径来回答：

- 驱动本身是否符合 Rust-for-Linux 的工程方向？

  - 是。
  - 驱动层零 ``unsafe``，抽象层承接 FFI 和回调桥接，这一点与现有 Rust 驱动实践一致。

- 封装抽象是否已经“完全正确到可以不再怀疑”？

  - 不能这么说。
  - 更准确的说法是：对 ``nlmon`` 当前用例，它已经足够正确到支持通过本轮工程验收；
    对更通用的推广目标，还应继续收紧与扩证。

后续建议
--------

如果下一步要继续把这个方向做成更强的博士课题/工具链基础，优先建议做三件事：

1. 做一个“理解 netlink 负载结构”的语义比较器，替代当前仅靠 ``tcpdump`` 文本和哈希的比较。
2. 把当前已经在 ``NetlinkTapHandle`` 上落地的 capability typing 经验继续扩展到更多
   registered / intrusive API，而不是只局限在 ``nlmon``。
3. 在此流程上再选一个稍复杂的网络模块，验证这套“bindgen -> helper -> 抽象 -> 驱动 ->
   差分测试”的方法能否稳定迁移到第二个真实案例。
