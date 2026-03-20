# ACCEPTANCE_CHECKLIST

- 版本：2026-03-20
- 用途：给 benchmark 和 agent 的统一验收口径

## A. 输入冻结

- [ ] 已确认 worktree、分支、remote、构建目录
- [ ] 已冻结 DUT、C 金标准、模块边界、文档输出目录
- [ ] 若历史不可靠，已校准 true blind base

## B. 构建

- [ ] C / Rust 模式可切换构建
- [ ] 默认构建通过
- [ ] 无新增 `git diff --check` 问题
- [ ] 驱动层仍为零 `unsafe`

## C. blind-first

- [ ] 已完成最小可工作原型
- [ ] 明确标注当前只达到 `prototype-grade`
- [ ] 未把 blind-first 误写成最终结论

## D. hardening

- [ ] 已检查 intrusive / registered / callback-owned 对象建模
- [ ] 已检查 private data / policy 生命周期
- [ ] 已检查 helper 是否仍必要
- [ ] 已检查 `Send/Sync`
- [ ] 已检查 safe API 是否过宽

## E. official-style delta review

- [ ] 若存在官方 Rust 参考实现，已在 hardening 后完成对照
- [ ] 已比较对象模型、注册边界、safe API、helper、`Send/Sync`
- [ ] 已区分“模块实现差距”和“工作流程差距”
- [ ] 已把 compare 结论写入正式文档

## F. `unsafe` 审计

- [ ] 已生成 `unsafe_audit.md`
- [ ] 每个 `unsafe` 点都覆盖前置条件和后置条件
- [ ] 已写明 safe API 调用者如何被限制

## G. 运行验证

- [ ] 已执行可信 runtime 验证
- [ ] 若没有可信 runtime harness，已明确记录 runtime gap
- [ ] 若存在 runtime gap，结论没有越级

## H. 差异分类

- [ ] 已生成 `difference_ledger.yaml`
- [ ] 与 C 金标准相比无未分类差异
- [ ] 如存在官方 Rust 参考实现，已记录官方风格差距

## I. 文档工件

- [ ] `module_manifest.yaml` 已刷新
- [ ] `evaluation_report.md` 已刷新
- [ ] `difference_ledger.yaml` 已刷新
- [ ] `unsafe_audit.md` 已刷新
- [ ] `acceptance_checklist.md` 已刷新
- [ ] `worklog` 已刷新

## J. 提交纪律

- [ ] 提交已按可回滚阶段拆分
- [ ] 每个 commit message 都写了 `Why / What / Verification`
- [ ] 结论表述没有越级
