# `nlmon` evaluation report

- 生成日期：2026-03-20
- 当前等级：`mainline-grade`
- benchmark 类型：current-tree small sample

## 结论

`nlmon` 当前已经完成 blind-first、hardening、差分验证和结构化文档刷新，
可以作为 current-tree benchmark 的小样本闭环基线。

它当前可以宣称：

- 通过了本地工程闭环；
- 完成了对象模型 hardening；
- 在 `nlmon` 这个具体 DUT 上达到了 `mainline-grade`。

它当前不能宣称：

- `rust/kernel/net/*` 已经可以直接视为通用上游抽象；
- 已经达到 `upstreamable-grade`；
- 所有 netlink 负载内容都已经完成逐字段语义比较。

## 主要证据

### 工程闭环

- C / Rust 版本可通过 `CONFIG_NLMON_RUST` 切换构建。
- `drivers/net/nlmon_rust.rs` 维持零 `unsafe`。
- `memory-debug`、`concurrency-debug`、`leak-debug` 三套 profile 均已有通过证据。

### hardening

- `NetlinkTapHandle` 已改成 pinned registered object。
- `Registration<T>` 已改成 mainline-style pinned opaque registration。
- 私有区访问模型已从宽泛 `&mut Private` 收缩为 pinned private access。

### 差异判断

- `dev_kfree_skb()` 与当前 Rust 路径使用的 `consume_skb()` 已明确判定为当前树等价路径。
- baseline 摘要的同日漂移已通过 C / Rust 同日补跑归类为环境漂移，而不是 Rust 特有功能差异。

## 阻塞项

`nlmon` 还没有升级到 `upstreamable-grade`，主要阻塞项如下：

- 没有更大样本来证明方法具有迁移性；
- `rust/helpers/net.c` 仍需单独做 helper 退役审计；
- `Registration<T>` 的 `Send/Sync` 仍依赖结构性证明，而不是更强的自动类型推导；
- 尚未提供“面向维护者”的对外 RFC 摘要。

## 下一步用途

- 作为 benchmark schema 示例目录；
- 作为后续 `ax88796b`、`cpufreq-dt`、`drm_panic_qr` 的格式与结论模板；
- 作为“什么叫 current-tree mainline-grade”的本地参照物。
