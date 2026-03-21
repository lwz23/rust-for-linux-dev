# 面向内核驱动 C2Rust 的评价阶梯

- 版本：2026-03-21
- 适用范围：`/home/lwz/rfl-dev/worktrees/rnull-blind-rust`
- 核心目的：把“能编译、能跑、看起来没问题”拆成可审计的等级和状态轴。

## 使用原则

- 每个模块都必须同时报告：
  - `conversion_stage`
  - `style_alignment`
  - `runtime_state`
- `grade` 只能作为压缩投影，不能代替这三条状态轴。

## 三轴定义

### 1. `conversion_stage`

- `blind`
  - 刚完成 blind-first 或仅到最小可运行原型
- `hardening`
  - 已进入对象模型和 safe API 收紧阶段
- `accepted-candidate`
  - 静态门禁已经通过，适合进入更高层面的工程或上游决策

### 2. `style_alignment`

- `unreviewed`
  - 尚未完成官方风格校准
- `benchmark-calibrated`
  - 只在 benchmark 校准流水线中使用
  - 表示已在 hardening 后与官方 Rust 实现完成结构性对照，并把结论回灌成规则

### 3. `runtime_state`

- `unvalidated`
  - 没有可信 runtime harness，或 runtime 验证尚未完成
- `validated`
  - 已有可信 runtime 验证证据

## grade 作为三轴投影

### `prototype-grade`

允许状态：

- `conversion_stage=blind` 或 `hardening`
- `runtime_state=unvalidated`

含义：

- 原型已跑通，或 hardening 已开始，但尚未形成完整工程闭环

### `engineering-grade`

必须满足：

- `conversion_stage=accepted-candidate`
- `runtime_state=validated`
- 无未分类功能差异

说明：

- 没有可信 runtime harness 时，不得宣称 `engineering-grade`

### `mainline-grade`

必须满足：

- `conversion_stage=accepted-candidate`
- `runtime_state=validated`
- 对象模型已经完成 hardening
- 若是 benchmark 校准流水线，还需要 `style_alignment=benchmark-calibrated`

说明：

- `mainline-grade` 不再等同于“仅静态上看起来像官方风格”

### `upstreamable-grade`

必须满足：

- 已达到 `mainline-grade`
- 形成对维护者可解释的理由链
- 有跨样本规则稳定性证据

## 关键规则

- 生产流水线不读取官方 Rust 实现，因此生产任务也必须报告三轴，但：
  - `style_alignment` 默认只能是 `unreviewed`
- benchmark 校准流水线在 hardening 后才允许把 `style_alignment` 升到
  `benchmark-calibrated`
- 历史校准样本若只有静态证据，即使已完成 official compare，也仍只能停留在
  `prototype-grade`
- toolchain 兼容补丁不直接提升或降低 grade；它们只进入 `旧树约束`

## 结论表述模板

- 若 `conversion_stage=blind`：
  - 只允许写“blind-first 原型已跑通”
- 若 `conversion_stage=hardening` 且 `style_alignment=benchmark-calibrated`：
  - 允许写“已完成官方风格校准，但尚未形成 runtime 工程闭环”
- 若 `conversion_stage=accepted-candidate` 且 `runtime_state=validated`：
  - 允许写“已通过工程级验证闭环”
