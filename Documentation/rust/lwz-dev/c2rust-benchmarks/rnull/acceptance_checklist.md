# `rnull` acceptance checklist

- 生成日期：2026-03-21
- 当前等级：`prototype-grade`

## 输入冻结

- [x] 已确认工作树、分支、reference tree、构建目录
- [x] 已确认真实 `pre-rnull` 语义基线与 `rnull` 引入提交
- [x] 已冻结 DUT 为 `drivers/block/null_blk/main.c`
- [x] 已冻结官方 Rust 参考为 current-tree `rnull.rs + configfs.rs`
- [x] 已写明这是 calibration rerun，不自动计入新的 formal benchmark 配额

## 构建

- [x] C / Rust 模式可切换构建
- [x] Rust 模式对象构建通过
- [x] C 模式对象构建通过
- [x] 无新增 `git diff --check` 问题
- [x] 驱动层仍为零 `unsafe`

## blind-first

- [x] 已完成 blind-first bootstrap
- [x] blind-first 阶段未提前读取官方 Rust 参考
- [x] 已生成 `c-structure-map.json`
- [x] 已生成 `binding-gap-audit.json`
- [x] 已生成 `helper-audit.json`

## hardening

- [x] 已检查 block-mq request / completion ownership
- [x] 已检查 configfs object lifecycle
- [x] 已检查 intrusive / registered / callback-owned state 建模
- [x] 已检查 safe API 是否过宽
- [x] 已检查 `unsafe impl Send/Sync`
- [x] 已检查 helper 是否仍必要
- [x] 已检查 `GenDisk` / `TagSet` / configfs child state 的 drop 配对

## `unsafe` 审计

- [x] 已生成标准化 `unsafe_audit.md`
- [x] 已生成 `unsafe-obligations.json`
- [x] 已写明 safe API 破坏面

## official compare

- [x] official compare 发生在 hardening 之后
- [x] 已生成 `calibration-delta.json`
- [x] 已明确回答范围扩大或缩小问题
- [x] 已明确回答 configfs 建模是否一致
- [x] 已明确回答 request completion / queue data ownership 是否弱化

## 差异分类

- [x] 已生成 `difference_ledger.yaml`
- [x] 已把所有已知差异分类
- [x] 已明确哪些属于 `旧树约束`
- [x] 已明确哪些属于 `不可接受功能差异`

## 运行验证

- [ ] 已建立 configfs create/delete 的可信 runtime 证据
- [ ] 已建立 power on/off 生命周期的可信 runtime 证据
- [ ] 已建立 blocksize / size / rotational / irqmode 配置约束的可信 runtime 证据
- [ ] 已建立基本 I/O completion 路径的可信 runtime 证据
- [x] 已明确记录 runtime gap，且没有伪造 `validated`

## 结论边界

- [x] 允许写“已完成 calibration rerun 并完成 official compare”
- [x] 不允许写“已达到 validated / engineering-grade / mainline-grade”
- [x] 不允许写“本次自动计入新的 formal benchmark 配额”
