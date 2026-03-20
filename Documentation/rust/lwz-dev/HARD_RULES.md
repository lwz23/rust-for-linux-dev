# HARD_RULES

- 版本：2026-03-20
- 目的：把 `cpufreq-dt` 这次新增坐实的规则写成不可违反的约束

## Rule 1

驱动层必须保持零 `unsafe`。

## Rule 2

先扩主 bindings，再做 helper 审计。不得因为 helper 更快就跳过 bindgen 缺口审查。

## Rule 3

`blind-first` 只算 `prototype-grade`。第一次跑通永远不是最终工程结论。

## Rule 4

原始 C 实现始终是功能金标准。Rust 版不能自行定义“这也算等价”。

## Rule 5

对 intrusive / registered / callback-owned 对象，默认优先
`Pin + Opaque + state machine`。

## Rule 6

只要 private data 里可能包含 pinned registered state，就禁止继续向 safe 驱动暴露宽泛
`&mut Private`。

## Rule 7

若某个 safe API 仍允许调用者移动已注册对象，这个抽象就还没有完成 hardening。

## Rule 8

`unsafe impl Send/Sync` 默认禁止。只有写出结构性证明后才允许保留。

## Rule 9

所有 `unsafe` 审计都必须同时覆盖：

- 前置条件
- 后置条件
- 所有权转移
- 生命周期边界
- 析构配对
- safe API 破坏面

## Rule 10

helper 不仅要做引入审计，还要做退役审计。过渡 helper 不允许无限期遗留。

## Rule 11

如果本地历史经过 graft / rewrite，必须先校准 true blind base。未校准前禁止开始 blind-write。

## Rule 12

若当前树里存在官方 Rust 版本，对比它属于 `reference-based evaluation`，只能放在
hardening 之后。

## Rule 13

hardening 之后必须新增 `official-style delta review`。这一步必须回答：

- 我们哪里已经与官方风格对齐
- 我们哪里仍不对齐
- 哪些不对齐属于旧树约束
- 哪些不对齐属于流程不够严格

## Rule 14

safe API 审计不能只和 C 金标准比，还必须显式比较官方当前树是否暴露了同样能力。

## Rule 15

在引入全局 registry、probe 期聚合或宽 helper 前，必须先证明
“per-policy / per-device 私有状态 + devres/platform registration” 不能表达目标对象模型。

## Rule 16

所有行为差异都必须进入 `difference_ledger.yaml`，并分类为：

- `安全修复`
- `旧树约束`
- `可接受工程差异`
- `不可接受功能差异`

未分类差异不能进入最终验收。

## Rule 17

没有可信 runtime harness 时，结论等级不得超过 `prototype-grade`。
