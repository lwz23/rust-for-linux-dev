# ACCEPTANCE_CHECKLIST

- 版本：2026-03-20
- 用途：给 benchmark 和 agent 统一验收口径

## A. 输入冻结

- [ ] 已确认工作树、分支、remote、构建目录
- [ ] 已确认 DUT、事件发生器、C 金标准、模块边界
- [ ] 已确认目标 grade

## B. 构建

- [ ] C / Rust 模式可切换构建
- [ ] 默认构建通过
- [ ] 无新增 `git diff --check` 问题
- [ ] 驱动层仍为零 `unsafe`

## C. blind-first

- [ ] 已完成最小可运行原型
- [ ] 明确标注当前只达到 `prototype-grade`
- [ ] 未把 blind-first 误写成最终结论

## D. hardening

- [ ] 已检查 intrusive / registered / callback-owned 对象建模
- [ ] 已检查 private data 访问模型
- [ ] 已检查 `Send/Sync`
- [ ] 已检查 helper 是否仍必要

## E. `unsafe` 审计

- [ ] 已生成 `unsafe_audit.md`
- [ ] 每个 `unsafe` 点都覆盖前置条件和后置条件
- [ ] 已写明 safe API 调用者如何被限制

## F. 运行验证

- [ ] 已执行目标子系统对应的 runtime 验证
- [ ] 已复用现有脚本或明确记录测试缺口
- [ ] 无新增 `WARNING/Oops/KASAN/lockdep/kmemleak/use-after-free/double-free/refcount` 异常

## G. 差异分类

- [ ] 已生成 `difference_ledger.yaml`
- [ ] 与 C 金标准相比无未分类差异
- [ ] 如存在官方 Rust 参考实现，已完成结构性对照

## H. 文档工件

- [ ] `module_manifest.yaml` 已刷新
- [ ] `evaluation_report.md` 已刷新
- [ ] `difference_ledger.yaml` 已刷新
- [ ] `unsafe_audit.md` 已刷新
- [ ] `acceptance_checklist.md` 已刷新
- [ ] 工作日志已刷新

## I. 提交纪律

- [ ] 提交已按可回滚阶段拆分
- [ ] 每个 commit message 都写了 `Why / What / Verification`
- [ ] 结论表述没有越级

## J. 进入外部 RFC 的额外门槛

- [ ] 当前树内至少 3 个 benchmark 证据包已经完成
- [ ] 至少 1 个新模块达到 `mainline-grade`
- [ ] 已形成面向维护者的理由链摘要
