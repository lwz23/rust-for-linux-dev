# AGENT_PLAYBOOK

- 版本：2026-03-20
- 角色：给 agent 的执行手册
- 本次修订来源：`cpufreq-dt` blind-write / hardening / official-style compare

## 目标

给定一个 Linux 内核 C 驱动模块后，agent 必须输出：

- Rust 实现；
- hardening 结果；
- official-style delta review；
- 标准 benchmark 工件；
- 方法学回写。

## 固定输入

每次任务至少要拿到：

- 模块名
- DUT 与 C 金标准路径
- 参考树路径
- blind base 或需要校准 blind base 的历史源
- worktree、分支、构建目录
- 文档输出目录
- 测试脚本或明确的 runtime gap 说明

## 固定阶段

### 阶段 0：环境冻结与 blind base 校准

必须做：

- 确认 worktree、分支、remote、构建目录
- 冻结 DUT、C 金标准、文档输出目录
- 若历史经过 graft / rewrite，先校准 true blind base
- 在开始编码前记录 blind base 与引入提交

禁止做：

- 用无法追溯的本地 SHA 直接充当 blind 结论依据
- 在 DUT 未冻结前开始写驱动

### 阶段 1：Kbuild / Kconfig

必须做：

- 保留 C 实现
- 用 config 选择器切换 C / Rust
- 记录 build config 组合

### 阶段 2：bindgen 缺口审计

必须做：

- 先查主 bindings
- 再记录真正缺失的 helper 候选

### 阶段 3：抽象层设计

必须做：

- 把 raw FFI 和 `unsafe` 压进 `rust/kernel/*` 或极小 helper
- 对 intrusive / registered / callback-owned 对象优先使用
  `Pin + Opaque + state machine`
- 在引入全局 registry 之前，先回答：
  “能否用 per-policy 或 per-device 的私有状态 + devres/platform registration 表达？”

禁止做：

- 为了快把 `unsafe` 留在驱动层
- 默认暴露宽泛 `&mut Private`
- 因为 C 恰好这么写，就先复制 probe 期全局聚合结构

### 阶段 4：blind-first bootstrap

必须做：

- 先得到驱动层零 `unsafe` 的最小可工作版本
- 明确写出 blind-first 只达到 `prototype-grade`

### 阶段 5：hardening

必须做：

- 收紧 safe API
- 检查 policy / private data / probe-remove 生命周期
- 检查 OPP、cpumask、clk、token、registration 的所有权与 drop 配对
- 检查 helper 是否仍必要
- 检查 `unsafe impl Send/Sync` 是否真的必要

### 阶段 6：official-style delta review

若当前树存在官方 Rust 参考实现，必须做：

- 只在 hardening 后阅读它
- 显式比较：
  - 对象模型
  - 注册边界
  - safe API 宽度
  - helper 位置与数量
  - `Send/Sync`
  - 生命周期表达
- 把结论拆成：
  - 模块实现差距
  - 工作流程差距

禁止做：

- 只写“和官方差异不大”
- 只比功能，不比风格和安全边界

### 阶段 7：验证与等级结论

必须做：

- 默认构建
- `git diff --check`
- 驱动层零 `unsafe` 扫描
- runtime 验证，或明确 runtime gap

强制规则：

- 没有可信 runtime harness 时，不得宣称 `engineering-grade` 或更高

### 阶段 8：文档与提交

必须做：

- 生成标准 benchmark 工件
- 在 `evaluation_report.md` 中单列：
  - `与官方实现的差距`
  - `对流程的反推修订建议`
- 按可回滚阶段拆 commit
- 每个 commit message 都写：
  - `Why`
  - `What`
  - `Verification`
