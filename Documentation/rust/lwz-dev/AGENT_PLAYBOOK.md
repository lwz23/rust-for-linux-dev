# AGENT_PLAYBOOK

- 版本：2026-03-20
- 角色：给 agent 的执行手册
- 使用方式：把本文件、`HARD_RULES.md`、`ACCEPTANCE_CHECKLIST.md`、`PROMPT_TEMPLATE.md` 一起提供给 agent

## 目标

在给定一个 Linux 内核 C 驱动模块后，agent 必须沿着受约束的 Rust 化流程工作，
输出结构化工件，而不是只产出“能编译的 Rust 文件”。

## 固定输入

agent 每次任务至少要拿到：

- 目标模块名
- C 金标准路径
- 当前内核树路径
- 构建脚本路径
- 测试脚本或测试缺口说明
- 文档输出目录

## 固定阶段

### 阶段 0：环境冻结

必须做：

- 确认工作树、分支、remote、构建目录
- 记录 DUT、事件发生器、C 金标准、目标 grade
- 创建或刷新 `module_manifest.yaml` 与工作日志

禁止做：

- 还没确认 DUT 就开始写代码
- 把参考实现、测试发生器、辅助脚本误当成主目标

### 阶段 1：Kbuild / Kconfig

必须做：

- 保留 C 实现
- 让 Rust 实现以 config 选择器切换构建
- 记录 C / Rust 模式的配置组合

输出：

- `module_manifest.yaml` 中的 `build` 部分

### 阶段 2：bindgen 缺口审计

必须做：

- 先查主 bindings 能否覆盖需求
- 只把真正缺失的 inline / 宏盲区记为 helper 候选

输出：

- 缺口列表
- helper 引入理由

### 阶段 3：抽象层设计

必须做：

- 把 raw FFI 和 `unsafe` 封进 `rust/kernel/*` 或极小 helper
- 写出局部不变式
- 对 intrusive / registered / callback-owned 对象优先使用
  `Pin + Opaque + state machine`

禁止做：

- 为了快而在驱动层留下 `unsafe`
- 默认暴露宽泛 `&mut Private`

### 阶段 4：blind-first bootstrap

必须做：

- 先得到驱动层零 `unsafe` 的可运行版本
- 记录这一步只达到 `prototype-grade`

禁止做：

- 把第一次跑通直接写成最终结论

### 阶段 5：mainline-grade hardening

必须做：

- 收紧 safe API
- 检查 private data 是否仍暴露过宽可变访问
- 检查 helper 是否可以退役
- 检查 `unsafe impl Send/Sync` 是否真的必要

输出：

- `difference_ledger.yaml`
- `unsafe_audit.md`

### 阶段 6：验证

必须做：

- 默认构建
- 目标子系统对应 runtime 验证
- 与 C 金标准做 diff classification

如果当前树存在 Rust 参考实现：

- 在 hardening 之后再做 reference-based compare
- 比较对象模型、safe API、`Pin/Opaque/state machine`、helper 残留、
  `Send/Sync` 和 private data 访问模型

### 阶段 7：文档与提交

必须做：

- 更新 `evaluation_report.md`
- 更新 `acceptance_checklist.md`
- 更新工作日志
- 按可回滚阶段拆 commit

提交正文必须写：

- `Why`
- `What`
- `Verification`

## 固定输出

每个模块目录都必须至少产出：

- `module_manifest.yaml`
- `evaluation_report.md`
- `difference_ledger.yaml`
- `unsafe_audit.md`
- `acceptance_checklist.md`

## 失败处理

如果构建或测试失败，agent 必须先分类：

- 代码 bug
- helper / bindings 缺口
- 旧树约束
- 测试基础设施缺口

然后再决定：

- 改代码
- 改抽象
- 记录为阻塞项

不允许直接跳过失败并继续宣称通过。
