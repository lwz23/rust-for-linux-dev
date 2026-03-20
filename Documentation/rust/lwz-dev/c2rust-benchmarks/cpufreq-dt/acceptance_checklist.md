# `cpufreq-dt` acceptance checklist

- 生成日期：2026-03-20
- 当前等级：`prototype-grade`

## 输入冻结

- [x] 已确认隔离 worktree、分支、remote、构建目录
- [x] 已冻结 DUT 为 `drivers/cpufreq/cpufreq-dt.c`
- [x] 已记录文档输出目录
- [x] 已校准 true blind base，而不是直接采用 grafted tree 中的给定 SHA

## 构建

- [x] C / Rust 模式可切换构建
- [x] `drivers/cpufreq/rcpufreq_dt.o` 默认构建通过
- [x] 无新增 `git diff --check` 问题
- [x] 驱动层仍为零 `unsafe`

## blind-first

- [x] 已完成 blind-first bootstrap
- [x] blind-first 已明确只算 `prototype-grade`
- [x] blind-first 期间未读取官方 Rust 参考实现

## hardening

- [x] 已完成 probe/remove 生命周期审查
- [x] 已完成 policy init/online/offline/exit 生命周期审查
- [x] 已完成 OPP table / cpumask / clk / config token ownership 审查
- [x] 已完成 safe API 宽度审查
- [x] 已完成 helper 必要性审查
- [x] 已完成 `Send/Sync` 复查

## official compare

- [x] 官方 compare 发生在 hardening 之后
- [x] 已比较对象模型、safe API、helper、`Send/Sync` 和生命周期表达
- [x] 已把差距拆成“模块实现差距”和“工作流程差距”
- [x] 已把 compare 结论回写到 benchmark 文档

## `unsafe` 审计

- [x] 已生成 `unsafe_audit.md`
- [x] 已写明前置条件、后置条件、所有权转移和析构配对
- [x] 已写明 safe API 破坏面

## 运行验证

- [ ] 已执行可信 DT-backed runtime harness
- [x] 已明确记录 runtime gap
- [x] 已明确由于 runtime gap 不得宣称 `engineering-grade` 或更高

## 差异分类

- [x] 已生成 `difference_ledger.yaml`
- [x] 无未分类差异
- [x] 已区分 `安全修复`
- [x] 已区分 `旧树约束`
- [x] 已区分 `可接受工程差异`
- [x] 没有已知 `不可接受功能差异`

## 文档工件

- [x] `module_manifest.yaml` 已生成
- [x] `evaluation_report.md` 已生成
- [x] `difference_ledger.yaml` 已生成
- [x] `unsafe_audit.md` 已生成
- [x] `acceptance_checklist.md` 已生成
- [x] `worklog` 已生成

## 结论边界

- [x] 允许写“blind-write、hardening、official-style delta review 已完成”
- [x] 不允许写“已达到 engineering-grade”
- [x] 不允许写“已达到 mainline-grade”
- [x] 不允许写“已建立可信 DT runtime 等价”
