# `nlmon` acceptance checklist

- 生成日期：2026-03-20
- 当前等级：`mainline-grade`

## 输入冻结

- [x] DUT 已冻结为 `drivers/net/nlmon.c`
- [x] 事件发生器已与 DUT 区分
- [x] C 金标准已固定

## 构建

- [x] C / Rust 可切换构建
- [x] 默认构建通过
- [x] 无新增 `git diff --check` 问题
- [x] 驱动层仍为零 `unsafe`

## blind-first

- [x] 已完成 blind-first bootstrap
- [x] 已明确 blind-first 不是最终结论

## hardening

- [x] 已完成 private pinned access 收缩
- [x] 已完成 pinned tap / pinned registration 收缩
- [x] 已完成 helper 保留项记录
- [x] 已完成 `Send/Sync` 保留项记录

## `unsafe` 审计

- [x] 已生成标准化 `unsafe_audit.md`
- [x] 已保留详细 `rst` 审计
- [x] 已写明 safe API 破坏面

## 运行验证

- [x] `memory-debug` baseline / lifecycle / matrix 通过
- [x] `concurrency-debug` baseline / lifecycle 通过
- [x] `leak-debug` baseline / lifecycle 通过
- [x] 无新增 `WARNING/Oops/KASAN/lockdep/kmemleak/use-after-free/double-free/refcount` 异常

## 差异分类

- [x] 已生成 `difference_ledger.yaml`
- [x] 无未分类差异
- [x] 已明确哪些属于 `安全修复`
- [x] 已明确哪些属于 `旧树约束`

## 结论边界

- [x] 允许写“`nlmon` 达到 current-tree `mainline-grade`”
- [x] 不允许写“通用 netdev 抽象已可直接上游”
- [x] 不允许写“已经达到 `upstreamable-grade`”
