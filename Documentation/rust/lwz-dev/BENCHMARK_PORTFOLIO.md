# 当前树 benchmark 样本池

- 版本：2026-03-20
- 目的：记录 current-tree / blind-write benchmark 的状态与样本价值

## 正式样本

| 模块 | 类型 | 样本规模 | 当前状态 | 作用 |
|---|---|---|---|---|
| `nlmon` | full driver | 小 | 已完成 | benchmark schema 小样本闭环 |
| `ax88796b` | full driver | 小 | 待执行 | 验证 `phy` 风格回调边界 |
| `cpufreq-dt` | full driver | 中 | 已完成但有 runtime gap | 验证 `platform + policy + OPP` 风格，并逼出官方风格对齐规则 |
| `drm_panic_qr` | component | 大组件 | 待执行 | 验证 panic-context 组件级 Rust 化 |

## `cpufreq-dt` 新增的组合价值

- 它证明了非 `netdev` 中样本会逼出新的对象模型问题：
  - probe/remove 与 policy init/exit 并不天然共享同一个 ownership 形状；
  - OPP table、cpumask、clk、token 更适合 per-policy 归属；
  - 官方风格对齐不能只看功能。
- 它新增了三条必须纳入后续 benchmark 的规则：
  - true blind base 校准；
  - hardening 后的 official-style delta review；
  - runtime gap 直接封顶等级。

## 统一协议

1. 冻结 DUT、C 金标准和 blind base。
2. 先做 blind-first，不提前宣称主线级。
3. 再做 hardening。
4. 若存在官方 Rust 参考实现，则最后做 official-style delta review。
5. 生成或刷新：
   - `module_manifest.yaml`
   - `evaluation_report.md`
   - `difference_ledger.yaml`
   - `unsafe_audit.md`
   - `acceptance_checklist.md`
   - `worklog`
