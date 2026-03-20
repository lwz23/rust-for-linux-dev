# 面向内核驱动 C2Rust 的评价阶梯

- 版本：2026-03-20
- 适用范围：`/home/lwz/rfl-dev/worktrees/cpufreq-dt-blind-rust`

## 使用原则

- 每个模块都只能声明自己已经达到的最高等级。
- 任何等级结论都必须有证据文件路径。
- 若缺少可信 runtime harness，等级上限立刻封顶。

## 四级定义

### 1. `prototype-grade`

定义：

- C / Rust 实现已经能切换构建。
- 驱动层保持零 `unsafe`。
- blind-first 与 hardening 结论已经分开记录。

允许存在：

- runtime gap
- official-style compare 已完成但仍无法升级

### 2. `engineering-grade`

定义：

- 已形成可重复的工程验证闭环。

必须满足：

- build / diff / unsafe / runtime 四类证据齐全
- 无未分类差异
- runtime 证据可信而非口头推断

升级阻塞项：

- 没有可信 runtime harness
- 仍依赖人工解释补足核心行为路径

### 3. `mainline-grade`

定义：

- 模块已经经历 hardening，safe API 和对象模型不再停留在 blind-first 原型期。

必须新增：

- official-style delta review
- 对齐项与未对齐项分开记录
- 明确哪些差距属于旧树约束，哪些属于流程问题

前置条件：

- 已先达到 `engineering-grade`

### 4. `upstreamable-grade`

定义：

- 模块不仅在本地闭环里成立，而且已经具备对维护者可解释的理由链。

前置条件：

- 已先达到 `mainline-grade`
- 已有跨样本方法稳定性证据

## 关键补充规则

- official-style compare 很强，也不能替代 runtime evidence。
- “更安全” 不自动等于 “更像官方风格”。
- 若当前树官方实现与 C 金标准本身存在偏移，报告必须分别写清：
  - 与 C 金标准的关系
  - 与官方实现的关系
