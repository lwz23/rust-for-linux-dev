# 当前树 benchmark 样本池

- 版本：2026-03-21
- 样本来源边界：优先使用当前树；必要时单开历史校准 rerun
- 目的：为内核驱动 C2Rust 工具提供“可追溯、可分层、可校准”的样本池

## 样本选择原则

- 必须优先选择“当前树里同时存在 C 金标准与 Rust 实现”的对象。
- 必须覆盖不同抽象形态，而不是只重复 `netdev`。
- 必须显式记录样本规模与测试缺口，避免把大样本和小样本混在一起。

## 正式样本组合

| 模块 | 路径 | 类型 | 样本规模 | 当前状态 | 作用 |
|---|---|---|---|---|---|
| `nlmon` | `drivers/net/nlmon.c` ↔ `drivers/net/nlmon_rust.rs` | full driver | 小 | 已完成 | 校准 `rtnl/netdevice registered module` 模式 |
| `ax88796b` | `drivers/net/phy/ax88796b.c` ↔ `drivers/net/phy/ax88796b_rust.rs` | full driver | 小 | 待执行 | 校准 `phy callback-table driver` 模式 |
| `cpufreq-dt` | `drivers/cpufreq/cpufreq-dt.c` ↔ `drivers/cpufreq/rcpufreq_dt.rs` | full driver | 中 | 待执行 | 校准 `platform + devres + per-policy ownership` 模式 |
| `drm_panic_qr` | `drivers/gpu/drm/drm_panic.c` 中 QR 逻辑 ↔ `drivers/gpu/drm/drm_panic_qr.rs` | component | 大组件 | 待执行 | 校准 `panic-context pure component` 模式 |

## 历史校准锚点

- `rnull`
  - 已完成 calibration rerun。
  - 不计入新的 formal benchmark 配额。
  - 这次 rerun 回灌了两条额外规则：
    - configfs child 持有 registered object 时必须同时审计 `drop_item` 与 final release；
    - 历史基线在现代 toolchain 上的兼容修复必须与 DUT correctness 分离。

## 每个样本必须重复的统一协议

1. 冻结 DUT、C 金标准和测试边界。
2. 先做 blind-first bootstrap，不提前宣称主线级。
3. 再做 hardening。
4. 若存在官方 Rust 参考实现，则最后做 reference-based compare。
5. 把对照结果回灌到：
   - `tooling-spec/rule-library.yaml`
   - `tooling-spec/pattern-library.yaml`
   - `tooling-spec/validator-matrix.yaml`
6. 生成或刷新：
   - `module_manifest.yaml`
   - `evaluation_report.md`
   - `difference_ledger.yaml`
   - `unsafe_audit.md`
   - `acceptance_checklist.md`
