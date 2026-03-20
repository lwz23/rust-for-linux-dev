# `ax88796b` evaluation report

- 生成日期：2026-03-20
- 当前等级：`prototype-grade`
- benchmark 类型：current-tree small sample

## 结论

`ax88796b` 已完成 blind-first、hardening、reference compare 和结构化工件刷新，
并且在隔离 worktree 内通过了 C / Rust 两套定向构建与完整 `bzImage modules`
构建。

它当前可以宣称：

- 手册流程能够驱动一个新的 current-tree phylib benchmark 落地；
- blind 驱动在 hardening 后仍保持驱动层零 `unsafe`；
- phylib 暴露出了不同于 `nlmon` 的新规则，尤其是 safe setter 宽度和
  注册对象的 `Send/Sync` 证明边界。

它当前不能宣称：

- 已达到 `engineering-grade` 或更高；
- 当前 `rust/kernel/net/phy.rs` 已可直接视为最终上游形态；
- 已完成可信 runtime 语义验证。

## 主要证据

### blind-first 与 hardening

- blind-first 从回退后的隔离基线起步完成，不读取现成 `ax88796b` Rust 驱动内容。
- `bindgen` 审计确认本轮所需符号均已覆盖，最终未新增 helper。
- hardening 将 `set_speed(u32)` 收紧为 `set_basic_speed(BasicSpeed)`，
  避免 safe 驱动任意写入 `phydev->speed`。
- hardening 去除了 `DriverVTable` 上的 `unsafe impl Sync`，把线程安全证明
  收敛到 `Registration` 句柄。

### reference compare

- `soft_reset`、`read_status`、`link_change_notify` 与 callback table 形状
  与参考实现保持等价。
- 对照后的主要差异均已进入 `difference_ledger.yaml`，且没有未分类项。
- 参考历史中的提交 `79e25710e...` 在本地树里是 grafted 根提交，因此无法恢复
  父子 diff；本轮采用当前参考文件级 compare 作为替代证据。

### 构建证据

- `git diff --check` 通过。
- 定向构建通过：
  - `scripts/lwz-dev/ax88796b-build.sh c drivers/net/phy/ax88796b.o`
  - `scripts/lwz-dev/ax88796b-build.sh rust drivers/net/phy/ax88796b_rust.o`
- 完整构建通过：
  - `scripts/lwz-dev/ax88796b-build.sh c`
  - `scripts/lwz-dev/ax88796b-build.sh rust`

## 阻塞项

`ax88796b` 目前停在 `prototype-grade`，主要阻塞项如下：

- `runtime_status` 仍为 `gap`，没有 PHY 真实 bring-up 或等价仿真证据；
- `Registration` 的 `Send/Sync` 仍需要结构性证明，而不是完全自动推导；
- 参考树 commit 祖先不可见，降低了“Rust 引入提交”粒度的可追溯性。

## 下一步用途

- 作为 phylib benchmark 的当前小样本；
- 作为“safe setter 要不要收紧、`Send/Sync` 应放在哪个对象边界” 的手册案例；
- 作为后续需要 runtime harness 的 benchmark 候选。
