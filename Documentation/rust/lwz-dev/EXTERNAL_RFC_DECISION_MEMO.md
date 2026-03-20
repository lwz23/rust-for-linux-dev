# EXTERNAL_RFC_DECISION_MEMO

- 版本：2026-03-20
- 当前结论：暂不直接发送“工具自动生成驱动补丁”的外部邮件

## 为什么现在不直接发代码补丁

- 当前最成熟的闭环证据主要来自 `nlmon`。
- `rnull` 提供了重要研究校准，但它不是当前树正式 benchmark 配额的一部分。
- 还没有形成足够多的 current-tree 样本，来证明方法能迁移到多个子系统。

## 当前阶段更合适的外部沟通形式

优先级最高的是“方法论 / RFC 型邮件”，而不是直接要求别人 review 大片自动生成代码。

建议的主题顺序：

1. 先介绍研究问题：
   - 我们不是在做自由翻译器，而是在做受规则约束的内核 Rust 化流水线。
2. 再介绍证据形式：
   - blind-first
   - hardening
   - reference-based compare
   - unsafe audit
   - diff classification
3. 最后再介绍当前样本与尚未解决的问题。

## 进入外部 RFC 阶段的硬门槛

- 当前树内至少 3 个 benchmark 证据包完成
- 至少 1 个新模块达到 `mainline-grade`
- 无未解释功能差异
- 已有一份面向维护者的理由链摘要

## 对外沟通目标对象

- Rust-for-Linux 维护者
- 对应子系统维护者
- 对方法学和验证流程感兴趣的内核开发者

## 对外邮件不应做的事

- 不要把本地 blind-first 结果直接宣称为“已经可上游”
- 不要先扔大段代码再请求别人免费审校整个工具
- 不要把 `旧树约束` 和 `安全修复` 混在一起

## 当前推荐动作

1. 先完成 `ax88796b`、`cpufreq-dt`、`drm_panic_qr` 三个 current-tree benchmark。
2. 用统一 schema 整理 `nlmon` 的示例证据包。
3. 再准备一份对外 RFC 摘要，而不是直接投递工具生成代码。
