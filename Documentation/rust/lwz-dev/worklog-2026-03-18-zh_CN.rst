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
2. 先补 Kconfig/Makefile 切换开关，保证 C/Rust 实现可以二选一构建。
3. 先扩展 ``rust/bindings/bindings_helper.h``，通过主 bindgen 暴露 ``nlmon`` 所需 C 类型与接口。
4. 对生成结果做缺口审计，确认哪些能力已由主 bindings 覆盖，哪些能力仍需额外处理。
5. 仅对 ``bindgen`` 无法直接处理的 inline 函数和复杂宏补最小 ``rust/helpers`` 包装。
6. 在 ``rust/kernel/net`` 下建立安全抽象层，把 ``unsafe`` 与 FFI 细节收敛到抽象内部。
7. 在抽象之上实现 ``drivers/net/nlmon_rust.rs``，保证驱动层零 ``unsafe``。
8. 最后执行构建、启动、差分测试，并根据结果继续迭代。

当前仓库基线（修正后）
----------------------

在修正本文档顺序时，再次确认当前仓库状态如下：

- 当前工作分支为 ``feature/nlmon-rust``。
- 当前 ``HEAD`` 为提交 ``60689628b``，主题为 ``Documentation: rust: add 2026-03-18 nlmon work log``。
- 目前除本文档与索引入口外，还没有任何 ``bindings/helper/抽象/驱动`` 代码提交落地。
- 从这一点开始，后续所有改动都遵循“单阶段、单提交、同步更新工作日志”的规则。

阶段记录
--------

1. 初始化阶段
~~~~~~~~~~~~~

- 从 ``dev`` 切出功能分支 ``feature/nlmon-rust``。
- 建立本日志文件，作为本轮所有提交的同步记录入口。

2. 流程顺序修正
~~~~~~~~~~~~~~~

- 重新核对 ``Documentation/rust/general-information.rst`` 与 ``rust/Makefile`` 中的实际机制。
- 确认本仓库必须遵循 ``Kbuild -> 主 bindgen -> 缺口审计 -> helper -> 抽象 -> 驱动 -> 差分测试`` 的顺序。
- 确认 ``rust/helpers`` 的绑定生成规则只允许导出 ``rust_helper_*`` 函数，不能承载新的 C 类型定义，因此 helper 不能替代主 bindings 阶段。

后续阶段会继续在本文件中追加记录。
