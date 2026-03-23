# `rnull` acceptance checklist

- 生成日期：2026-03-23
- 任务性质：`fresh workflow validation rerun`
- formal benchmark 配额：不自动计入

## 阶段顺序

- `bindgen / bindings coverage audit`：通过
- `helper audit`：通过
- `pattern selection + abstraction layer`：通过
- `blind-first Rust driver`：通过
- `hardening`：通过
- `runtime validation`：部分通过
- `official compare`：通过
- `documentation + acceptance + commit`：通过

## 冻结与范围

- fresh worktree / fresh branch / fixed build dir：通过
- `pipeline_mode=calibration` / `module_id=rnull` / `conversion_stage=blind` 冻结：通过
- `c_function_target=main.c core + configfs` 明确写入：通过
- 排除项 `zoned` / fault injection 显式列出：通过
- 官方 Rust lower-bound / mature 参考写入：通过

## 工程硬要求

- 驱动层零 `unsafe`：通过
- `unsafe` 尽量收敛到抽象层：通过
- 驱动层不直接碰 bindings：通过
- helper 不承载驱动私有 C 类型：通过
- 功能差异 / 工程差异 / 旧树约束显式区分：通过

## 构建与静态门

- blind-first 模块目录构建：通过
- memory-debug C baseline 全量构建：通过
- memory-debug Rust candidate 全量构建：通过
- `git diff --check`：通过
- `rg -n '\bunsafe\b' drivers/block/rnull`：通过

## runtime 证据

- C baseline `baseline`：通过
- C baseline `lifecycle`：通过
- C baseline `matrix-core`：通过
- C baseline `io-core`：通过，但 `mbps` 仅记录为 `runtime_gap`
- Rust candidate `baseline`：通过
- Rust candidate `lifecycle`：通过
- Rust candidate `matrix-core`：通过
- Rust candidate `io-core`：通过
- A/B 结论：部分通过
- 无新的 `WARNING/Oops/KASAN/lockdep`：通过

## official compare

- 仅在 runtime 之后读取官方 `rnull`：通过
- lower-bound 与 mature 参考都已对比：通过
- `calibration-delta.json` 已生成：通过

## 最终判定

- 能否宣称达到冻结 C 金标准：不能
- 原因：`mbps` 冻结目标在 C baseline 上仍是 `runtime_gap`，所以整轮 `runtime_state` 必须保持 `unvalidated`
- 能否宣称工程闭环完成：部分可以
- 说明：blind-first、hardening、memory-debug A/B、official compare 都已完成，但 helper 退役与附加 profile 仍未闭环
