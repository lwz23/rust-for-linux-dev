# PROMPT_TEMPLATE

- 版本：2026-03-20
- 用途：给 agent 的可直接复用 prompt 模板

## 使用说明

把下面模板里的占位符替换掉，然后把本文件、`AGENT_PLAYBOOK.md`、
`HARD_RULES.md`、`ACCEPTANCE_CHECKLIST.md` 一起交给 agent。

## 模板

```text
你现在要执行一个 Linux 内核驱动模块的 Rust 化 benchmark 任务。

固定输入如下：

- 目标模块名：{{MODULE_NAME}}
- C 金标准路径：{{C_IMPL_PATH}}
- 当前 Rust 参考实现路径：{{RUST_REFERENCE_PATH_OR_NONE}}
- 当前工作树：{{KERNEL_TREE}}
- 工作区根目录：{{WORKSPACE_ROOT}}
- 构建脚本：{{BUILD_SCRIPT}}
- 测试脚本或测试说明：{{TEST_SCRIPT_OR_TEST_GAP}}
- 文档输出目录：{{OUTPUT_DIR}}
- 当前目标等级：{{TARGET_GRADE}}

你必须遵守以下方法学：

1. 先探索，不猜路径，不猜 config，不猜参考实现边界。
2. 先冻结 DUT、事件发生器、C 金标准和验收标准。
3. 先做 bindgen 缺口审计，再决定是否引入 helper。
4. 驱动层必须保持零 unsafe。
5. blind-first 只算 prototype-grade，不能直接写成最终结论。
6. hardening 时必须重点检查：
   - intrusive / registered / callback-owned 对象是否使用 Pin + Opaque + state machine
   - private data 是否仍暴露宽泛 &mut Private
   - unsafe impl Send/Sync 是否真的必要
   - helper 是否仍需保留
7. 若当前树存在 Rust 参考实现，只能在 hardening 之后再做 reference-based compare。
8. 所有差异都必须进入 difference_ledger.yaml，并分类为：
   - 安全修复
   - 旧树约束
   - 可接受工程差异
   - 不可接受功能差异

你必须输出以下工件：

- module_manifest.yaml
- worklog
- evaluation_report.md
- difference_ledger.yaml
- unsafe_audit.md
- acceptance_checklist.md
- 分阶段 commit 计划

每个工件至少包含以下信息：

- module_manifest.yaml：
  - DUT
  - C 金标准
  - Rust 参考实现
  - 子系统
  - 构建开关
  - 测试入口
  - 当前等级
- evaluation_report.md：
  - 当前等级
  - 主要证据
  - 阻塞项
  - 是否能升级到下一等级
- difference_ledger.yaml：
  - 每项差异的分类
  - 证据来源
  - 是否已解决
- unsafe_audit.md：
  - 前置条件
  - 后置条件
  - 所有权转移
  - 生命周期边界
  - 析构配对
  - safe API 破坏面
- acceptance_checklist.md：
  - 构建
  - 运行验证
  - diff classification
  - 文档
  - 提交纪律

输出风格要求：

- 先给事实，再给结论。
- 不把第一次跑通写成“工程验收通过”。
- 不把历史研究校准流程混入 current-tree benchmark 正式结论。
- 如果测试失败，先分类原因，再决定是改代码还是记为旧树约束。
- 如果样本缺乏 runtime 脚本，必须显式记录“测试缺口”，不能伪造通过。
```
