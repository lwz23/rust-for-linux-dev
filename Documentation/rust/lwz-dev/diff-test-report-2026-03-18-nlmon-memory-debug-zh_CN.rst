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
- ``matrix`` 场景下，五类事件发生器都已经在主观测指标上对齐，``netns`` 公共噪声已移除。
- ``stress`` 场景下，C/Rust 两边都能稳定运行 1800 秒并完成大规模抓包，且不再被
  ``netns`` 发生器公共噪声污染。
- 当前 ``memory-debug`` 结果已经能作为更强的一条行为证据链，但仍不能单独替代
  ``unsafe`` 审计与其他调试 profile 的结论。

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

4. ``netns`` 公共噪声已被收紧
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

本轮早期的 ``matrix`` / ``stress`` 日志里，C/Rust 两边都会出现：

- ``Cannot find device "nlmon_ns_veth0"``

后续核对脚本后确认，这不是 ``nlmon`` 差异，而是 ``gen_netns()`` 在
``ip netns del nlmonns0`` 之后又多执行了一次 ``ip link del nlmon_ns_veth0``，
从而人为制造了额外失败事件。

在删除这条冗余清理命令后，重新补采的 ``memory-debug`` ``matrix`` / ``stress`` 日志中：

- C/Rust 两边都不再出现这条噪声
- ``dmesg_anomaly`` 计数保持为 ``0``
- ``netns`` 发生器的摘要也不再继续被这条公共失败路径扰动

因此，从本报告此版本开始，这条现象不再被视为当前 ``memory-debug`` 结果里的保留问题。

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

  - C：``142224`` bytes / ``9200`` decoded lines / ``800`` captured
  - Rust：``142224`` bytes / ``9200`` decoded lines / ``800`` captured
  - 两边的 ``dmesg_anomaly.matrix.netns`` 都为 ``0``
  - 两边都不再出现 ``Cannot find device "nlmon_ns_veth0"``
  - 两边的 ``normalized.head`` 与 ``tcpdump`` captured/filter 计数也再次对齐

这说明：

- 当前五类事件发生器都已经完成逐类 50 轮后的摘要对齐。
- ``matrix`` 结果不再被 ``netns`` 的公共清理噪声污染。

6. ``stress`` 已经完成长时抓包对照
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

当前 ``memory-debug`` 下，C/Rust 两边都已经完成默认 ``1800`` 秒 ``stress``：

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

这一轮结果可以支持以下判断：

- 两边都已经通过长时抓包与高频事件发生器混合负载。
- 上一阶段出现的 ``No space left on device`` 已被确认是测试 harness 问题，而不是
  ``nlmon`` 功能失败。
- 在收紧 ``netns`` 清理顺序后，``stress`` 日志中也不再出现
  ``Cannot find device "nlmon_ns_veth0"`` 公共噪声。
- 当前 ``stress`` 里仍存在摘要差异，因此它更适合被解释为“长时稳定性证据”，而不是
  “逐字节完全一致”证据。结合规范化 ``pcap`` 报告，更合理的解释仍然是：

  - 长时运行里五类事件发生器的交错顺序本身就是运行相关的
  - 被捕获的 netlink 报文本体中仍包含运行相关的可变字段
  - 原始 ``pcap`` 或简单去时间戳后的哈希，不能单独充当最终等价判据

阶段性判断
----------

基于当前证据，可以成立的判断是：

- 当前 ``memory-debug`` 下的 ``baseline``、``lifecycle``、``matrix``、``stress``
  都已经完成一轮 C/Rust 强化对照。
- Rust 版 ``nlmon`` 至少没有在这四类场景里引入额外的调试内核异常。
- 在当前主机侧可见指标下，C/Rust 外部行为已经高度接近，且 ``netns`` 公共噪声
  已经被清除。
- ``matrix`` 现在可作为更强的行为等价证据；``stress`` 主要作为稳定性与
  “无新增调试异常”证据。

当前不能成立的判断是：

- “已经完成最终工程验收”
- “抽象层的 ``unsafe`` 已被完全证明健全”
- “C/Rust 已经逐字节完全一致”

下一步建议
----------

建议将本报告与以下材料合并阅读：

1. ``diff-test-report-2026-03-18-nlmon-concurrency-debug-zh_CN.rst``
2. ``diff-test-report-2026-03-18-nlmon-leak-debug-zh_CN.rst``
3. ``diff-test-report-2026-03-18-nlmon-normalized-pcap-zh_CN.rst``
4. 更新后的 ``unsafe`` 审计文档
5. 最终工程验收结论文档
