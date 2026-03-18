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
2. 先补 ``rust/helpers`` 与 ``rust/kernel/net`` 的安全抽象。
3. 再接入 ``drivers/net/nlmon_rust.rs`` 与 Kconfig/Makefile 切换逻辑。
4. 最后执行构建、启动、差分测试，并根据结果继续迭代。

阶段记录
--------

1. 初始化阶段
~~~~~~~~~~~~~

- 从 ``dev`` 切出功能分支 ``feature/nlmon-rust``。
- 建立本日志文件，作为本轮所有提交的同步记录入口。

后续阶段会继续在本文件中追加记录。
