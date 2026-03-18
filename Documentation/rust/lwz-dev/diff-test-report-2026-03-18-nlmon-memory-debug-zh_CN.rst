.. SPDX-License-Identifier: GPL-2.0

2026-03-18 nlmon ``memory-debug`` 强化差分报告
=============================================

摘要
----

本报告记录 ``nlmon`` Rust 化第二阶段中，基于 ``memory-debug`` 调试内核完成的
一轮强化差分结果。这里的目标不是宣称“工程验收已经完成”，而是明确回答：

- 在同一套 ``memory-debug`` 基线下，原始 C 版 ``drivers/net/nlmon.c`` 与
  Rust 版 ``drivers/net/nlmon_rust.rs`` 是否已经完成一阶行为对照？
- 当前哪些结论已经有证据支持？
- 哪些问题仍然悬而未决，不能过度解读？

本轮结论可以概括为：

- ``baseline`` 场景下，C/Rust 两边都已经稳定抓到非空 ``pcap``。
- ``lifecycle`` 场景下，C/Rust 两边都能稳定完成 100 轮 add/up/del。
- ``matrix`` 场景下，``dummy`` / ``veth`` / ``bridge`` / ``route_rule`` 四类发生器的
  摘要已经对齐，``netns`` 仍有公共噪声与轻微漂移。
- ``stress`` 场景下，C/Rust 两边都能稳定运行 1800 秒并完成大规模抓包。
- 当前 ``memory-debug`` 结果支持“强化行为对照已经基本成形”的阶段性判断。
- 但由于 ``netns`` 发生器尚未收紧、原始 ``pcap`` 的 ``sha256`` 仍不同，且尚未跑完
  ``concurrency-debug`` / ``leak-debug``，因此还不能把这轮结果写成最终工程验收通过。

范围与前提
----------

本报告只覆盖以下范围：

- 调试构建：``memory-debug``
- 实现场景：

  - C 版 ``nlmon.ko``
  - Rust 版 ``nlmon_rust.ko``

- 测试场景：

  - ``baseline``
  - ``lifecycle``
  - ``matrix``
  - ``stress``

本轮不覆盖：

- ``concurrency-debug``
- ``leak-debug``
- 最终工程验收结论

术语约定
--------

- DUT 始终只有 ``nlmon``。
- ``dummy``、``veth``、``bridge``、``route/rule``、``netns`` 只是 netlink 事件发生器。
- 当前 Rust 实现仍应被称为“驱动层零 ``unsafe``、抽象层已收紧并完成第一轮审计的原型”。
- 本报告不等同于“抽象层已经被完全证明健全”。

测试方法
--------

1. 使用同一套 ``memory-debug`` QEMU 环境分别构建并运行 C / Rust 两版。
2. ``baseline`` 场景中：

   - 创建并启动 ``nlmon0``
   - 后台运行 ``tcpdump``
   - 依次触发 ``dummy`` / ``veth`` / ``bridge`` / ``route_rule`` / ``netns`` 事件发生器
   - 记录 ``ip`` 观察结果、``pcap`` 摘要和 ``dmesg`` 摘要

3. ``lifecycle`` 场景中：

   - 连续 100 轮 ``add/up/del nlmon0``
   - 每 10 轮混入一次事件发生器流量
   - 检查 ``status``、``dmesg`` 与公共噪声是否同型

4. ``matrix`` 场景中：

   - 分别对 ``dummy`` / ``veth`` / ``bridge`` / ``route_rule`` / ``netns``
     做 50 轮事件发生器循环
   - 对每类发生器分别记录 ``pcap`` 摘要与 ``dmesg`` 摘要

5. ``stress`` 场景中：

   - 连续 1800 秒循环触发五类事件发生器
   - 检查长时抓包、摘要行数与 ``dmesg`` 是否稳定

6. 结果对比优先看：

   - ``status=ok``
   - ``dmesg_anomaly``
   - ``pcap`` 字节数
   - decoded 行数
   - ``tcpdump`` 的 captured/filter 计数
   - 公共噪声是否同型

关键结果
--------

1. ``baseline`` 已经收敛
~~~~~~~~~~~~~~~~~~~~~~~~~

当前 ``memory-debug`` 下，C/Rust 两边的 ``baseline`` 都已经稳定完成，并且具备以下共同点：

- ``status=ok``
- ``observation_mode.baseline.nlmon0=full-iproute2``
- ``dmesg_anomaly.baseline=0``
- ``baseline.pcap.bytes=38596``
- ``baseline.decoded.lines=2474``
- ``baseline.tcpdump.stderr.head`` 都显示：

  - ``158 packets captured``
  - ``158 packets received by filter``
  - ``0 packets dropped by kernel``

这说明在当前基线下，C/Rust 两边：

- 都确实从 ``nlmon0`` 抓到了大量 netlink 报文
- 抓包结果的数量级已经收敛
- 当前 baseline 不再受此前 ``tcpdump`` 用户缺失或缓冲时序问题污染

2. ``baseline`` 仍有一个未决差异
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

尽管 ``pcap`` 字节数与 decoded 行数已经一致，但原始 ``pcap`` 的 ``sha256`` 仍然不同。

当前阶段，对这一点的保守解释应当是：

- 这可能来自二进制层面的时间戳或记录细节差异
- 仅凭 ``pcap`` 原始哈希不同，还不足以断言 C / Rust 行为不等价
- 若后续需要更强证据，应继续增加：

  - decoded 内容摘要
  - 去时间戳后的规范化比较
  - 更强的 ``matrix`` / ``stress`` 对照

3. ``lifecycle`` 已经收敛
~~~~~~~~~~~~~~~~~~~~~~~~~~

当前 ``memory-debug`` 下，C/Rust 两边的 ``lifecycle`` 摘要在去掉 ``implementation=...``
后保持一致：

- ``status=ok``
- ``dmesg_anomaly.lifecycle=0``
- ``dmesg_anomaly_lines.lifecycle=0``
- 每 10 轮插入的 ``dummy`` 观察结果都为 ``full-iproute2``

这说明至少在当前 100 轮 add/up/del 基线上：

- C/Rust 两边都没有暴露新增的调试内核异常
- ``nlmon`` 生命周期路径已经通过一轮重复性验证

4. 当前仍存在公共测试噪声
~~~~~~~~~~~~~~~~~~~~~~~~~~

在 C/Rust 两边的 ``baseline`` / ``lifecycle`` 日志中，仍然都会出现：

- ``Cannot find device "nlmon_ns_veth0"``

由于该现象在两边完全同型，目前更适合作为“事件发生器实现尚待收紧”的公共噪声，
而不应归因到 ``nlmon`` C/Rust 实现差异上。

5. ``matrix`` 已经完成首轮强化对照
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

当前 ``memory-debug`` 下，C/Rust 两边都已经完成 ``matrix`` 场景，其中：

- ``dummy``：

  - 两边均为 ``600424`` bytes / ``38300`` decoded lines / ``1900`` captured

- ``veth``：

  - 两边均为 ``492824`` bytes / ``31600`` decoded lines / ``1900`` captured

- ``bridge``：

  - 两边均为 ``542824`` bytes / ``34800`` decoded lines / ``2300`` captured

- ``route_rule``：

  - 两边均为 ``132824`` bytes / ``8650`` decoded lines / ``800`` captured

- ``netns``：

  - C：``161608`` bytes / ``10464`` decoded lines / ``1004`` captured
  - Rust：``160716`` bytes / ``10407`` decoded lines / ``1002`` captured
  - 两边的 ``dmesg_anomaly.matrix.netns`` 都为 ``0``
  - 两边日志都带有 ``Cannot find device "nlmon_ns_veth0"`` 公共噪声

这说明：

- 四类较稳定事件发生器已经完成逐类 50 轮后的摘要对齐。
- 当前 ``matrix`` 里剩余的主要不确定性，已经收敛到 ``netns`` 发生器本身的清理顺序与
  事件边界上，而不是 ``nlmon`` 整体功能路径都在漂移。

6. ``stress`` 已经完成长时抓包对照
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

当前 ``memory-debug`` 下，C/Rust 两边都已经完成默认 ``1800`` 秒 ``stress``：

- Rust：

  - ``status=ok``
  - ``dmesg_anomaly.stress=0``
  - ``416143792`` bytes
  - ``26691293`` decoded lines
  - ``1703996`` captured

- C：

  - ``status=ok``
  - ``dmesg_anomaly.stress=0``
  - ``416168736`` bytes
  - ``26692887`` decoded lines
  - ``1704052`` captured

这一轮结果可以支持以下判断：

- 两边都已经通过长时抓包与高频事件发生器混合负载。
- 上一阶段出现的 ``No space left on device`` 已被确认是测试 harness 问题，而不是
  ``nlmon`` 功能失败。
- 当前 ``stress`` 里仍存在轻微摘要漂移，因此还不能把它写成“逐字节完全一致”。
  结合 ``matrix.netns`` 的现象，更合理的解释仍然是：

  - ``netns`` 发生器存在公共噪声
  - 原始 ``pcap`` 受时间戳/记录细节影响，不能只看原始哈希

阶段性判断
----------

基于当前证据，可以成立的判断是：

- 当前 ``memory-debug`` 下的 ``baseline``、``lifecycle``、``matrix``、``stress``
  都已经完成一轮 C/Rust 强化对照。
- Rust 版 ``nlmon`` 至少没有在这四类场景里引入额外的调试内核异常。
- 在当前主机侧可见指标下，C/Rust 外部行为已经高度接近，且差异面已被压缩到
  ``netns`` 发生器与原始 ``pcap`` 口径。

当前不能成立的判断是：

- “已经完成最终工程验收”
- “抽象层的 ``unsafe`` 已被完全证明健全”
- “C/Rust 已经逐字节完全一致”

下一步建议
----------

建议按照以下顺序继续推进：

1. 在 ``concurrency-debug`` 与 ``leak-debug`` 内核下，至少先复跑 C/Rust 的
   ``baseline`` / ``lifecycle``。
2. 为 ``pcap`` 增加去时间戳/规范化后的比较口径，不再只盯原始 ``sha256``。
3. 收紧 ``netns`` 发生器的清理顺序，去掉
   ``Cannot find device "nlmon_ns_veth0"`` 这类公共噪声。
4. 在完成上述两点后，重跑受影响的 ``matrix`` / ``stress`` 场景。
5. 在上述步骤全部完成后，再输出最终工程验收结论文档与完整复现流程文档。
