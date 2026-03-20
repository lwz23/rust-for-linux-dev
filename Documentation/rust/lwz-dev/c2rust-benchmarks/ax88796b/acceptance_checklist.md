# `ax88796b` acceptance checklist

- 生成日期：2026-03-20
- 当前等级：`engineering-grade`

## 输入冻结

- [x] DUT 已冻结为 `drivers/net/phy/ax88796b.c`
- [x] blind-write 输出目录已固定
- [x] 只读参考树与隔离 worktree 已分离

## 构建

- [x] C / Rust 可切换构建
- [x] `git diff --check` 通过
- [x] 定向 C 构建通过
- [x] 定向 Rust 构建通过
- [x] 完整 `bzImage modules` C 构建通过
- [x] 完整 `bzImage modules` Rust 构建通过
- [x] 完整 debug C 构建通过
- [x] 完整 debug Rust 构建通过
- [x] 驱动层仍为零 `unsafe`

## blind-first

- [x] 已完成 blind-first bootstrap
- [x] blind-first 期间未读取禁读参考驱动
- [x] bindgen 缺口与 helper 决策已记录

## hardening

- [x] 已审查 phylib callback object model
- [x] 已收紧过宽 safe setter
- [x] 已审查 helper 必要性并确认本轮无需 helper
- [x] 已审查 `Send/Sync` 并收缩到最小必要对象
- [x] 已记录 ownership / lifetime / drop 配对
- [x] 已把 `phy_device` bitfield 访问切换到 bindgen accessor

## `unsafe` 审计

- [x] 已生成标准化 `unsafe_audit.md`
- [x] 已写明 safe API 破坏面
- [x] 已记录 reference compare 后的 unsafe 差异
- [x] 未留下阻塞 `engineering-grade` 的未证明 soundness 缺口

## reference compare

- [x] 已在 hardening 后读取参考 Rust 驱动和 phylib 抽象
- [x] 已在 runtime 对齐后刷新 reference compare
- [x] 已生成 `difference_ledger.yaml`
- [x] 无未分类差异
- [x] 已明确 `安全修复`、`旧树约束`、`可接受工程差异`

## 运行验证

- [x] benchmark-only KUnit harness 已落地且默认关闭
- [x] fake Clause 22 bus 支持寄存器回放、顺序 trace 和单次故障注入
- [x] C mode runtime 通过
- [x] Rust mode runtime 通过
- [x] `runtime_diff.yaml` 为零差异
- [x] `module_manifest.yaml` 已记录 `runtime_status: passed`
- [x] 未伪造 runtime 通过

## 结论边界

- [x] 允许写“当前达到 `engineering-grade`”
- [x] 不允许写“已达到 `mainline-grade`”
- [x] 不允许写“已完成真实硬件 bring-up”
- [x] 不允许写“当前参考树 phylib 抽象已可直接视为最终上游形态”
