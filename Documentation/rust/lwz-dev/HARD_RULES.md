# HARD_RULES

- 版本：2026-03-21
- 目的：把已经被 benchmark 反复坐实的规则写成不可违反的约束

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

只要 private 区可能包含 pinned registered state，就禁止继续向 safe 驱动暴露宽泛
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

生产流水线不得读取官方 Rust 实现。官方 Rust 只属于 benchmark 校准输入。

## Rule 12

若当前树里存在官方 Rust 版本，对比它属于 `reference-based evaluation`，
只能放在 hardening 之后。

## Rule 13

agent 只能实例化已登记的 pattern library，不能自行发明新的生命周期模型。

## Rule 14

静态前端必须先输出 `c-structure-map.json`，再允许进入抽象与驱动生成阶段。

## Rule 15

若模式无法稳定归类，输出必须同时标记：

- `pattern unresolved`
- `manual rule needed`
- `stage ceiling reached`

## Rule 16

对 `devm_*` / probe-remove 配对对象，默认优先 devres-owned registration/resource。

## Rule 17

对 split lifecycle 对象，默认按“真正拥有资源的最小生命周期单元”建模，
而不是先照搬 C 的全局 registry 或 probe-scoped 聚合结构。

## Rule 18

helper 只能导出最小函数桥接，不能承载新的驱动私有 C 类型。

## Rule 19

以后任何结论都必须同时报告三条状态轴：

- `conversion_stage`
- `style_alignment`
- `runtime_state`

单个 grade 不能单独承载全部结论。

## Rule 20

所有行为差异必须进入 `difference_ledger.yaml`，并分类为：

- `安全修复`
- `旧树约束`
- `可接受工程差异`
- `不可接受功能差异`

未分类差异不能进入最终验收。

## Rule 21

若 configfs child 持有已注册对象，必须同时审计：

- `power` 或等价用户入口
- `drop_item`
- final `release`

不能只检查用户显式的 on/off 路径就宣称生命周期闭合。

## Rule 22

历史基线在现代 toolchain 上暴露出的兼容修复，必须归类为 `旧树约束` 或环境漂移，
不能混写成驱动行为正确性结论。
