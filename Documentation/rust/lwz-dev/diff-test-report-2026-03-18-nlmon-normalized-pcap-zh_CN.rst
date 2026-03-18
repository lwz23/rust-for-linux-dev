.. SPDX-License-Identifier: GPL-2.0

2026-03-18 nlmon ``pcap`` 规范化比较说明
========================================

摘要
----

本说明记录 ``nlmon`` 第二阶段测试中，为解释 ``pcap.sha256`` 差异所做的一轮
去时间戳/规范化比较。

核心结论如下：

- 只看原始 ``pcap.sha256`` 不可靠，不能直接推出 C/Rust 功能不一致。
- 但仅仅把 ``tcpdump`` 文本解码前缀中的时间戳去掉，也还不足以得到相同哈希。
- 在 ``memory-debug``、``concurrency-debug``、``leak-debug`` 三套 profile 中，
  C/Rust 两边虽然 ``normalized.sha256`` 仍不同，但以下指标都保持一致：

  - ``pcap.bytes``
  - decoded 行数
  - ``tcpdump`` captured/filter 计数
  - 去时间戳后的头部摘要

这说明：

- 原始 ``pcap`` 差异不应再作为最终判据。
- 但想要做更严格的内容级比较，还需要更语义化的 canonicalizer。

方法
----

本轮在 guest 侧新增了两个结果字段：

- ``baseline.normalized.sha256``
- ``baseline.normalized.head``

处理方式是：

1. 对 ``tcpdump -nn -r`` 的输出逐行处理。
2. 若某一行以 ``HH:MM:SS.xxxxxx`` 时间戳开头，则去掉该前缀。
3. 对去时间戳后的整条文本流做 ``sha256``。
4. 同时记录去时间戳后的头部摘要，作为人工复核样本。

结果概览
--------

1. ``memory-debug``
~~~~~~~~~~~~~~~~~~~

- C：

  - ``pcap.bytes=38596``
  - ``decoded.lines=2474``
  - ``normalized.sha256=b20a9afcdbaead084c82c5c93194643ad6805009a2fff364d7aa2e97c1cd3b83``

- Rust：

  - ``pcap.bytes=38596``
  - ``decoded.lines=2474``
  - ``normalized.sha256=12f4504d04a477f66cfcf7c86b580e868009b5fbf6d274c422af141dabc9885b``

- 对照结论：

  - ``pcap.bytes`` 一致
  - ``decoded.lines`` 一致
  - ``normalized.head`` 一致
  - ``normalized.sha256`` 不一致

2. ``concurrency-debug``
~~~~~~~~~~~~~~~~~~~~~~~~

- C：

  - ``pcap.bytes=39488``
  - ``decoded.lines=2531``
  - ``normalized.sha256=bb2a4e425daf0f19a9bafb26fb2aa4674a50d94ca7f6b68c8e32fda44c4060a2``

- Rust：

  - ``pcap.bytes=39488``
  - ``decoded.lines=2531``
  - ``normalized.sha256=3f66002bafe73dfb9940c1b28a54029d32d42fb77c52d7dd066d622831b57bd4``

- 对照结论：

  - ``pcap.bytes`` 一致
  - ``decoded.lines`` 一致
  - ``normalized.head`` 一致
  - ``normalized.sha256`` 不一致

3. ``leak-debug``
~~~~~~~~~~~~~~~~~

- C：

  - ``pcap.bytes=39488``
  - ``decoded.lines=2531``
  - ``normalized.sha256=1abe34cfc90e498bf9558785caed26ad9aeb0eeef538ae93634530c91ae37cc9``

- Rust：

  - ``pcap.bytes=39488``
  - ``decoded.lines=2531``
  - ``normalized.sha256=c754ab75065e59b1ad375263b5f88e57c749c0dc65ffbb1578e7b4a1ae654aa0``

- 对照结论：

  - ``pcap.bytes`` 一致
  - ``decoded.lines`` 一致
  - ``normalized.head`` 一致
  - ``normalized.sha256`` 不一致

解释
----

这一轮结果说明，原始 ``pcap`` 的差异来源至少分成两层：

1. **外层记录差异**

   - 原始 ``pcap`` 天生带有记录时间戳
   - 因此 raw hash 本来就不适合作为第一判据

2. **更深层的运行相关字段差异**

   - 即使去掉 ``tcpdump`` 文本的时间戳前缀，整体哈希仍不同
   - 说明捕获到的 netlink 报文负载里还存在运行相关的可变字段

因此，当前最合理的判断是：

- C/Rust 在当前黑盒摘要层面已经高度对齐
- 但要证明“内容级完全一致”，还需要更强的语义规范化工具

当前建议的比较口径
------------------

在新的语义化比较器出现之前，建议把以下指标作为更稳健的阶段判据：

- ``status=ok``
- ``dmesg_anomaly=0``
- ``pcap.bytes``
- decoded 行数
- ``tcpdump`` captured/filter 计数
- ``normalized.head`` 是否一致

而不应继续把以下条目单独当成最终判据：

- 原始 ``pcap.sha256``
- 仅去掉外层时间戳后的 ``normalized.sha256``

下一步
------

建议继续按以下顺序推进：

1. 收紧 ``netns`` 事件发生器里的公共噪声。
2. 重跑受影响的 ``matrix`` / ``stress``。
3. 在最终验收文档里明确区分：

   - 当前已经对齐的行为级证据
   - 仍需更语义化 canonicalizer 才能解决的内容级比较问题
