# HARD_RULES

- 版本：2026-03-20
- 目的：把已经被 `nlmon / rnull / nlmon hardening` 反复坐实的规则写成不可违反的约束

## Rule 1

驱动层必须保持零 `unsafe`。

## Rule 2

先扩主 bindings，再做 helper 审计。不得因为写 helper 更快，就跳过 bindgen 缺口审查。

## Rule 3

`blind-first` 只算 `prototype-grade`。第一次跑通永远不是最终工程结论。

## Rule 4

原始 C 实现始终是功能金标准。Rust 版不能自行定义“这也算等价”。

## Rule 5

对 intrusive / registered / callback-owned 对象，默认优先
`Pin + Opaque + state machine`。

## Rule 6

只要 `Private` 里可能包含 pinned registered state，就禁止继续向 safe 驱动暴露宽泛 `&mut Private`。

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

若当前树里存在官方 Rust 版本，对比它属于 `reference-based evaluation`，只能放在 hardening 之后。

## Rule 12

所有行为差异必须进入 `difference_ledger.yaml`，并分类为：

- `安全修复`
- `旧树约束`
- `可接受工程差异`
- `不可接受功能差异`

未分类差异不能进入最终验收。
