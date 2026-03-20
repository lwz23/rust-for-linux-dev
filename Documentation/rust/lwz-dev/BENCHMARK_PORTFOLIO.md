# 当前树 benchmark 样本池

- 版本：2026-03-20
- 样本来源边界：只使用当前 `/home/lwz/rfl-dev/linux` 工作树
- 上游基线：`upstream/rust-next`
- 目的：为内核驱动 C2Rust 工具提供“current-tree、可追溯、可分层”的研究样本池

## 样本选择原则

- 必须优先选择“当前树里同时存在 C 金标准与 Rust 实现”的对象。
- 必须覆盖不同抽象形态，而不是只重复 `netdev`。
- 必须显式记录样本规模与测试缺口，避免把大样本和小样本混在一起。

## 正式样本组合

| 模块 | 路径 | 类型 | 样本规模 | 当前状态 | 作用 |
|---|---|---|---|---|---|
| `nlmon` | `drivers/net/nlmon.c` ↔ `drivers/net/nlmon_rust.rs` | full driver | 小 | 已完成 | current-tree 小样本闭环基线 |
| `ax88796b` | `drivers/net/phy/ax88796b.c` ↔ `drivers/net/phy/ax88796b_rust.rs` | full driver | 小 | 待执行 | 验证 `phylib` 回调与等价性 |
| `cpufreq-dt` | `drivers/cpufreq/cpufreq-dt.c` ↔ `drivers/cpufreq/rcpufreq_dt.rs` | full driver | 中 | 待执行 | 验证 `platform + policy + OPP` 风格 |
| `drm_panic_qr` | `drivers/gpu/drm/drm_panic.c` 中 QR 逻辑 ↔ `drivers/gpu/drm/drm_panic_qr.rs` | component | 大组件 | 待执行 | 验证 panic-context 组件级 Rust 化 |

## 已有校准锚点

- `nlmon`
  - 已完成 blind-first、hardening、差分验证和工程验收。
  - 适合作为标准化工件示例目录。
- `rnull`
  - 继续作为历史研究校准锚点使用。
  - 不计入当前树 benchmark 的正式样本配额。

## 当前明确排除的对象

### `qt2025.rs`

- 原因：只有 vendor C 参考，不是当前树内同级 C 金标准。
- 结论：可作为后续“vendor-reference”研究样本，但不进入当前阶段正式样本池。

### `pwm_th1520.rs`

- 原因：当前树里没有对应的同级 C 移植对象。
- 结论：不适合做 blind-write ↔ current-tree compare。

### `binder`

- 原因：规模过大，且当前树里没有同级“一对一 C 移植对象”。
- 结论：后置为大型系统样本，不进入本轮。

### `nova / tyr`

- 原因：Rust-first 对象，不符合“C 金标准 ↔ Rust 实现”的 benchmark 条件。

## 执行顺序

1. `nlmon`
   - 作为 schema 与目录结构示例，先标准化已有证据。
2. `ax88796b`
   - 先跑一个第二个小样本，验证 `net/phy` 子系统是否会引入新的硬规则。
3. `cpufreq-dt`
   - 用中样本验证 `probe/init/online/opp` 这类非 `netdev` 形态。
4. `drm_panic_qr`
   - 最后处理 panic-context 组件级样本，验证“完整驱动之外”的研究边界。

## 每个样本必须重复的统一协议

1. 冻结 DUT、C 金标准和测试边界。
2. 先做 blind-first bootstrap，不提前宣称主线级。
3. 再做 mainline-grade hardening。
4. 若当前树存在 Rust 参考实现，则最后做 reference-based compare。
5. 生成或刷新：
   - `module_manifest.yaml`
   - `evaluation_report.md`
   - `difference_ledger.yaml`
   - `unsafe_audit.md`
   - `acceptance_checklist.md`

## 当前样本不足时的处理规则

- 不在本轮里临时放宽到历史树。
- 不在本轮里临时引入 vendor-only C 参考。
- 若四个样本做完仍不足以归纳新规则，再单开“样本来源扩展”计划。
