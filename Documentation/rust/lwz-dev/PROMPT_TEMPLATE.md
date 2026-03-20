# PROMPT_TEMPLATE

- 版本：2026-03-20
- 用途：给 agent 的可直接复用 prompt 模板

```text
你现在要执行一个 Linux 内核驱动模块的 Rust 化 benchmark 任务。

固定输入如下：

- 目标模块名：{{MODULE_NAME}}
- C 金标准路径：{{C_IMPL_PATH}}
- 官方 Rust 参考实现路径：{{RUST_REFERENCE_PATH_OR_NONE}}
- 当前参考树：{{REFERENCE_TREE}}
- blind 历史树：{{HISTORY_TREE}}
- worktree：{{WORKTREE}}
- 分支：{{BRANCH}}
- 构建目录：{{BUILD_DIR}}
- 文档输出目录：{{OUTPUT_DIR}}
- 测试脚本或 runtime gap 说明：{{TEST_OR_GAP}}

你必须遵守以下方法学：

1. 若本地历史经过 graft 或 rewrite，先校准 true blind base。
2. 先冻结 DUT、C 金标准、文档输出目录和验收边界。
3. 先做 bindgen 缺口审计，再决定是否引入 helper。
4. 驱动层必须保持零 unsafe。
5. blind-first 只算 prototype-grade，不能直接写成最终结论。
6. hardening 时必须重点检查：
   - probe/remove 或等效生命周期
   - policy init/online/offline/exit
   - OPP/cpumask/clk/token ownership 与 drop 配对
   - safe API 是否过宽
   - unsafe impl Send/Sync 是否真的必要
   - helper 是否仍需保留
7. 在引入全局 registry 或 probe 期聚合前，先证明
   “per-policy/per-device 私有状态 + devres/platform registration” 不能表达目标。
8. 若存在官方 Rust 参考实现，只能在 hardening 之后做 official-style delta review。
9. official-style delta review 必须回答：
   - 我们哪里已经与官方风格对齐
   - 我们哪里还不够主线风格
   - 哪些是模块差距
   - 哪些是流程差距
10. 所有差异都必须进入 difference_ledger.yaml，并分类为：
   - 安全修复
   - 旧树约束
   - 可接受工程差异
   - 不可接受功能差异
11. 如果没有可信 runtime harness，必须明确记录 runtime gap，
    且不得宣称 engineering-grade 或更高。

你必须输出以下工件：

- module_manifest.yaml
- worklog
- evaluation_report.md
- difference_ledger.yaml
- unsafe_audit.md
- acceptance_checklist.md

并且：

- `evaluation_report.md` 必须包含：
  - 与官方实现的差距
  - 对流程的反推修订建议
- 每个 commit message 都必须包含：
  - Why
  - What
  - Verification
```
