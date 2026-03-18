.. SPDX-License-Identifier: GPL-2.0

2026-03-18 nlmon ``concurrency-debug`` 差分报告
==============================================

摘要
----

本报告记录 ``nlmon`` Rust 化第二阶段中，在 ``concurrency-debug`` 调试内核下完成的
一轮 C/Rust ``baseline`` / ``lifecycle`` 对照。该 profile 的关注点不是长时抓包吞吐，
而是并发与锁语义诊断器是否会暴露新的问题。

本轮可以成立的结论是：

- ``concurrency-debug`` 下，C/Rust 两边的 ``baseline`` 都稳定通过。
- ``concurrency-debug`` 下，C/Rust 两边的 ``lifecycle`` 都稳定通过。
- 当前没有观测到新增的 lockdep、RCU、atomic-sleep、spinlock 或 ``DEBUG_NET`` 异常。
- ``baseline`` 的关键数量级摘要在两边再次对齐。

本轮仍不能推出：

- 抽象层 ``unsafe`` 已被完全证明健全。
- 已完成最终工程验收。

范围
----

本报告只覆盖：

- 调试构建：``concurrency-debug``（``/home/lwz/rfl-dev/build-nlmon-lockdep``）
- 实现场景：

  - C 版 ``nlmon.ko``
  - Rust 版 ``nlmon_rust.ko``

- 测试场景：

  - ``baseline``
  - ``lifecycle``

本报告不覆盖：

- ``matrix``
- ``stress``
- ``leak-debug``
- 最终工程验收结论

诊断器配置
----------

本 profile 重点启用了以下调试能力：

- ``CONFIG_PROVE_LOCKING=y``
- ``CONFIG_PROVE_RCU=y``
- ``CONFIG_DEBUG_ATOMIC_SLEEP=y``
- ``CONFIG_DEBUG_SPINLOCK=y``
- ``CONFIG_DEBUG_NET=y``

测试方法
--------

1. 先创建 ``concurrency-debug`` 独立构建目录并完成内核构建。
2. 先运行 Rust 版 ``baseline`` / ``lifecycle``。
3. 再切换 ``CONFIG_NLMON_RUST=n``，重建后运行 C 版 ``baseline`` / ``lifecycle``。
4. 结果优先关注：

   - ``status=ok``
   - ``dmesg_anomaly``
   - baseline 的 ``pcap`` 字节数
   - baseline 的 decoded 行数
   - ``tcpdump`` captured/filter 计数

关键结果
--------

1. Rust ``baseline`` 通过
~~~~~~~~~~~~~~~~~~~~~~~~~

Rust ``baseline`` 的关键摘要为：

- ``status=ok``
- ``dmesg_anomaly.baseline=0``
- ``baseline.pcap.bytes=39488``
- ``baseline.decoded.lines=2531``
- ``160 packets captured``

串口日志中还出现了：

- ``ip (104) used greatest stack depth: 11104 bytes left``

当前将其记录为普通调试统计信息，而不是 ``nlmon`` 回归，因为：

- 它没有触发 ``dmesg_anomaly`` 规则
- 它不属于 lockdep / RCU / atomic-sleep / spinlock 报告

2. Rust ``lifecycle`` 通过
~~~~~~~~~~~~~~~~~~~~~~~~~~

Rust ``lifecycle`` 的关键摘要为：

- ``status=ok``
- ``dmesg_anomaly.lifecycle=0``
- ``dmesg_anomaly_lines.lifecycle=0``

3. C ``baseline`` 通过且与 Rust 对齐
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

C ``baseline`` 的关键摘要为：

- ``status=ok``
- ``dmesg_anomaly.baseline=0``
- ``baseline.pcap.bytes=39488``
- ``baseline.decoded.lines=2531``
- ``160 packets captured``

这说明在当前 ``concurrency-debug`` 下，C/Rust 两边的 baseline 至少在当前主机侧
可观测摘要上再次对齐。

4. C ``lifecycle`` 通过
~~~~~~~~~~~~~~~~~~~~~~~

C ``lifecycle`` 的关键摘要为：

- ``status=ok``
- ``dmesg_anomaly.lifecycle=0``
- ``dmesg_anomaly_lines.lifecycle=0``

阶段性判断
----------

基于当前证据，可以成立的判断是：

- ``concurrency-debug`` 下，C/Rust 两边的 ``baseline`` / ``lifecycle`` 都能稳定完成。
- 当前没有观测到新增的并发诊断器异常。
- 这说明 ``nlmon`` 当前实现至少没有在 lockdep / RCU / atomic-sleep 这一层立即暴露出
  新的问题。

当前不能成立的判断是：

- ``matrix`` / ``stress`` 在 ``concurrency-debug`` 下也已经完成验证
- ``unsafe`` 审计已经可以停止
- ``netns`` 发生器问题已经解决

下一步
------

建议继续按以下顺序推进：

1. 在 ``leak-debug`` 下复跑 C/Rust 的 ``baseline`` / ``lifecycle``。
2. 补充去时间戳/规范化后的 ``pcap`` 比较口径。
3. 收紧 ``netns`` 发生器并重跑受影响场景。
