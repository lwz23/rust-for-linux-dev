.. SPDX-License-Identifier: GPL-2.0

面向内核驱动 C2Rust 工具的研究工件入口
======================================

摘要
----

本页汇总 ``2026-03-20`` 起新增的一组研究工件。它们不再继续采用“大而全叙述文档”的组织方式，
而是拆成适合 agent 执行与 benchmark 归档的 ``md`` / ``yaml`` 文件。

这些工件的目标是：

- 固定评价阶梯与外部沟通节奏；
- 固定 benchmark 样本池与样本边界；
- 固定 agent 执行手册、硬规则、验收口径与 prompt 模板；
- 为后续模块 rustification 提供统一的结构化输出目录。

根手册
------

以下文件位于 ``Documentation/rust/lwz-dev/``：

- ``EVALUATION_LADDER.md``
- ``BENCHMARK_PORTFOLIO.md``
- ``AGENT_PLAYBOOK.md``
- ``HARD_RULES.md``
- ``ACCEPTANCE_CHECKLIST.md``
- ``PROMPT_TEMPLATE.md``
- ``EXTERNAL_RFC_DECISION_MEMO.md``

benchmark 工件目录
-----------------

标准化 benchmark 工件位于：

- ``Documentation/rust/lwz-dev/c2rust-benchmarks/nlmon/``
- ``Documentation/rust/lwz-dev/c2rust-benchmarks/ax88796b/``
- ``Documentation/rust/lwz-dev/c2rust-benchmarks/cpufreq-dt/``
- ``Documentation/rust/lwz-dev/c2rust-benchmarks/drm-panic-qr/``

其中 ``nlmon`` 目录已经提供一套完整示例工件：

- ``module_manifest.yaml``
- ``evaluation_report.md``
- ``difference_ledger.yaml``
- ``unsafe_audit.md``
- ``acceptance_checklist.md``

其余目录当前先提供 ``module_manifest.yaml``，用于冻结样本边界、构建开关、
参考实现路径、测试缺口与后续执行顺序。

使用建议
--------

建议按下面顺序使用这些文件：

1. 先读 ``EVALUATION_LADDER.md`` 明确等级含义；
2. 再读 ``BENCHMARK_PORTFOLIO.md`` 明确当前样本池；
3. 接着把 ``AGENT_PLAYBOOK.md``、``HARD_RULES.md``、
   ``ACCEPTANCE_CHECKLIST.md``、``PROMPT_TEMPLATE.md`` 一起交给 agent；
4. 最后在对应 benchmark 目录下生成或刷新模块级工件。
