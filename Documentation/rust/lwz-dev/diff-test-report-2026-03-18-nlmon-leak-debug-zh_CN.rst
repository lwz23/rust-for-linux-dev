.. SPDX-License-Identifier: GPL-2.0

2026-03-18 nlmon ``leak-debug`` 差分报告
=======================================

摘要
----

本报告记录 ``nlmon`` Rust 化第二阶段中，在 ``leak-debug`` 调试内核下完成的
一轮 C/Rust ``baseline`` / ``lifecycle`` 对照，以及为使该 profile 可用而做的一次
测试口径修正。

本轮可以成立的结论是：

- ``leak-debug`` 下，C/Rust 两边的 ``baseline`` 都稳定通过。
- ``leak-debug`` 下，C/Rust 两边的 ``lifecycle`` 都稳定通过。
- 收紧 ``kmemleak`` 相关异常匹配后，启动横幅不再被误判为异常。
- 在当前测试窗口内，没有观测到 ``kmemleak: <count> new suspected memory leaks`` 或
  ``unreferenced object`` 这类真实泄漏信号。

本轮仍不能推出：

- 已经完成真正的长期泄漏压力验证。
- 最终工程验收已经通过。

范围
----

本报告只覆盖：

- 调试构建：``leak-debug``（``/home/lwz/rfl-dev/build-nlmon-kmem``）
- 实现场景：

  - C 版 ``nlmon.ko``
  - Rust 版 ``nlmon_rust.ko``

- 测试场景：

  - ``baseline``
  - ``lifecycle``

本报告不覆盖：

- ``matrix``
- ``stress``
- 真实长时 ``kmemleak`` 扫描压力
- 最终工程验收结论

口径修正
--------

在第一次 Rust ``baseline`` 中，自动化框架把以下启动横幅误判为异常：

- ``kmemleak: Kernel memory leak detector initialized ...``
- ``kmemleak: Automatic memory scanning thread started``

这表明旧规则中过于宽泛的 ``kmemleak`` 粗匹配不适用于 ``leak-debug``。

本轮修正后的策略是只匹配真正的疑似泄漏信号：

- ``kmemleak: <count> new suspected memory leaks``
- ``unreferenced object``

而不再把初始化横幅视为异常。

测试方法
--------

1. 创建 ``leak-debug`` 独立构建目录并完成内核构建。
2. 在修正后的异常匹配规则下，先运行 Rust 版 ``baseline`` / ``lifecycle``。
3. 再切换 ``CONFIG_NLMON_RUST=n``，重建后运行 C 版 ``baseline`` / ``lifecycle``。
4. 结果优先关注：

   - ``status=ok``
   - ``dmesg_anomaly``
   - baseline 的 ``pcap`` 字节数
   - baseline 的 decoded 行数
   - ``tcpdump`` captured/filter 计数
   - 是否出现真实 ``kmemleak`` 疑似泄漏信号

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

4. C ``lifecycle`` 通过
~~~~~~~~~~~~~~~~~~~~~~~

C ``lifecycle`` 的关键摘要为：

- ``status=ok``
- ``dmesg_anomaly.lifecycle=0``
- ``dmesg_anomaly_lines.lifecycle=0``

5. 当前 ``leak-debug`` 结论的边界
~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~~

当前可以确定的是：

- 启动横幅误报已经被消除
- baseline/lifecycle 窗口内没有出现真实 ``kmemleak`` 疑似泄漏输出

当前还不能确定的是：

- ``matrix`` / ``stress`` 下是否仍然完全干净
- 更长时间窗口下是否会出现延迟暴露的泄漏

阶段性判断
----------

基于当前证据，可以成立的判断是：

- ``leak-debug`` 下，C/Rust 两边的 ``baseline`` / ``lifecycle`` 都能稳定完成。
- 当前自动化框架已经能区分 ``kmemleak`` 启动横幅和真实疑似泄漏信号。
- 在当前测试窗口内，没有看到新的 ``kmemleak`` 异常。

下一步
------

建议继续按以下顺序推进：

1. 为 ``pcap`` 增加去时间戳/规范化后的比较口径。
2. 收紧 ``netns`` 发生器，去掉公共噪声并缩小 ``matrix`` / ``stress`` 漂移。
3. 在完成上述两点后，重跑受影响场景并输出最终工程验收文档。
