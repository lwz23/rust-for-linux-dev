# `drm-panic-qr` acceptance checklist

- 生成日期：2026-03-21
- `pipeline_mode=calibration`
- `conversion_stage=hardening`
- `style_alignment=benchmark-calibrated`
- `runtime_state=unvalidated`
- `stage ceiling=hardening`
- `validation gap`：
  - 无可信 panic runtime harness
  - 本机 `libclang 14.0.0` 阻塞 Rust-enabled build / KUnit
  - blind lane 发生 pre-S9 参考污染

## 输入冻结

- [x] 工作树、分支、输出目录、构建目录已冻结
- [x] DUT、C 金标准、组件边界已冻结
- [x] 路径级 blind base / intro 已重新校准

## 静态前端

- [x] 已生成 `dut-spec.json`
- [x] 已生成 `c-structure-map.json`
- [x] 已完成 `panic-context-pure-component` 证据化选择
- [x] 已说明为什么这不是完整驱动 benchmark

## 构建与集成面

- [x] 已冻结 `CONFIG_DRM_PANIC_SCREEN_QR_CODE` 的 C / Rust 集成面
- [x] 驱动层保持零 `unsafe`
- [ ] 默认 build 通过
- [ ] Rust-enabled KUnit 通过

## blind-first 与 hardening

- [x] 已完成 blind-first 组件实现
- [x] 已明确 blind-first 不能当工程结论
- [x] 已完成 helper 审计
- [x] 已完成 no-allocation / ownership / drop-pair 静态审计
- [x] 未新增 `Send/Sync`

## `unsafe`

- [x] 已生成 `unsafe_audit.md`
- [x] 已生成 `unsafe-obligations.json`
- [x] 所有 `unsafe` 均封在 `rust/kernel/drm/panic_qr.rs`

## 校准流水线

- [x] 已生成 `calibration-delta.json`
- [x] 已拆出 aligned / misaligned / old_tree_constraints / workflow_gaps
- [x] `evaluation_report.md` 已单列“与官方实现的差距”
- [x] `evaluation_report.md` 已单列“对流程的反推修订建议”

## 差异分类

- [x] 已生成 `difference_ledger.yaml`
- [x] 所有已识别行为差异都已分类
- [x] 已明确存在 1 项 `不可接受功能差异`

## 结论边界

- [x] 允许写“已完成 blind-first / hardening / 官方风格校准”
- [x] 不允许写“已经形成 runtime 工程闭环”
- [x] 不允许写 `engineering-grade`
- [x] 不允许把这次结果描述成完全 reference-free blind-first
