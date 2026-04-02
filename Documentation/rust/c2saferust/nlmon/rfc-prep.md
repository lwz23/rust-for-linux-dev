# RFC Prep

## Branches

- 完整实验分支：
  - 本地：`review/nlmon-tooling-clean-v1-fullstate`
  - 远端：`origin/review/nlmon-tooling-clean-v1-fullstate`
- upstream-clean RFC 分支：
  - 本地：`rfc/nlmon-reference-driver-v1`
  - 远端：`origin/rfc/nlmon-reference-driver-v1`
  - RFC worktree：`/tmp/nlmon-rfc-v1.k3atFH`
  - 对外作者：`Wenzhao Liao <wenzhaoliao@ruc.edu.cn>`

## RFC 保留内容

- 仅保留可能 upstream 的内核 patch：
  - `MAINTAINERS`
  - `drivers/net/Kconfig`
  - `drivers/net/Makefile`
  - `drivers/net/nlmon_rust.rs`
  - `rust/bindings/bindings_helper.h`
  - `rust/helpers/helpers.c`
  - `rust/helpers/net.c`
  - `rust/kernel/net.rs`
  - `rust/kernel/net/netdevice.rs`
  - `rust/kernel/net/netlink_tap.rs`
  - `rust/kernel/net/rtnl.rs`
  - `rust/kernel/net/skbuff.rs`
  - `rust/kernel/net/stats.rs`

## RFC 不保留内容

- `Documentation/rust/c2saferust/*`
- `scripts/c2saferust/*`
- `plan.md`
- `task_context.md`
- JSON artifact、实验日志、QEMU smoke 记录

## Patch Series

- `bef13e17b4b9 rust: bindings: expose networking headers needed by nlmon`
- `b0a8f06ff98f rust: helpers: add net_device and sk_buff helper wrappers`
- `a8080ce3ee34 rust: net: add minimal skbuff, netdevice, and stats abstractions`
- `654a4d889a92 rust: net: add minimal rtnl registration and netlink tap support`
- `482cd5a6601d net: add Rust reference driver for nlmon`
- `1765259a2fe6 MAINTAINERS: add Rust net and nlmon entries`

patch 输出目录：

- `/tmp/nlmon-rfc-patches`

cover letter 主题：

- `[RFC PATCH 0/6] rust: net: introduce minimal rtnl/netdevice abstractions and nlmon reference driver`

## Validation

- safety gate：
  - `/tmp/nlmon-rfc-safety.json`
  - `pass = true`
- agent gate：
  - `/tmp/nlmon-rfc-gate.json`
  - `pass = true`
- QEMU smoke：
  - `/tmp/nlmon-rfc-smoke.json`
  - `/tmp/nlmon-rfc-smoke.log`
  - `pass = true`
  - `ip link add nlmon0 type nlmon`
  - `ip link set nlmon0 up`
  - `ip link show nlmon0`
  - `ip link set nlmon0 down`
  - `ip link del nlmon0`
- patch apply：
  - `/tmp/nlmon-rfc-patches/0001-0006` 已在临时 clone 通过 `git am`
- `git send-email`：
  - 已通过用户本地安装补齐到 `~/bin/git-send-email`
  - `git send-email --dry-run` 通过

## Recommended Recipients

主列表：

- `rust-for-linux@vger.kernel.org`
- `netdev@vger.kernel.org`

额外抄送：

- `linux-kernel@vger.kernel.org`
- `ojeda@kernel.org`
- `boqun@kernel.org`
- `gary@garyguo.net`
- `bjorn3_gh@protonmail.com`
- `lossin@kernel.org`
- `a.hindborg@kernel.org`
- `aliceryhl@google.com`
- `tmgross@umich.edu`
- `dakr@kernel.org`
- `andrew+netdev@lunn.ch`
- `davem@davemloft.net`
- `edumazet@google.com`
- `kuba@kernel.org`
- `pabeni@redhat.com`

## Draft Command

只做 dry-run，不实际发送：

`git -C /tmp/nlmon-rfc-v1.k3atFH send-email --dry-run --suppress-cc=self --suppress-cc=sob --to rust-for-linux@vger.kernel.org --to netdev@vger.kernel.org --cc linux-kernel@vger.kernel.org --cc ojeda@kernel.org --cc boqun@kernel.org --cc gary@garyguo.net --cc bjorn3_gh@protonmail.com --cc lossin@kernel.org --cc a.hindborg@kernel.org --cc aliceryhl@google.com --cc tmgross@umich.edu --cc dakr@kernel.org --cc andrew+netdev@lunn.ch --cc davem@davemloft.net --cc edumazet@google.com --cc kuba@kernel.org --cc pabeni@redhat.com /tmp/nlmon-rfc-patches/*.patch`

当前环境说明：

- 本机已补齐 `git send-email`
- 目前完成到“patch、收件人、cover letter、dry-run、git am 自测均已准备好”
- 仍未向外部列表实际发信

## Format Notes

- 只从 `rfc/nlmon-reference-driver-v1` 导出 patch，不从 `review` 分支导出
- cover letter 与 patch 全部为英文
- 不向邮件系列中带入 `scripts/c2saferust/*`、artifact、实验记录或中文文档
- 当前 2/6-5/6 在 `checkpatch` 中仅剩 “new file, does MAINTAINERS need updating?”
  的通用提醒；由于 6/6 已补 `MAINTAINERS`，该状态可接受

## Cover Letter 重点

- 强调 minimal Rust net abstractions 为什么合理
- 强调 `nlmon` 为什么是合适的 reference driver
- 强调 soundness boundary：
  - driver `#![forbid(unsafe_code)]`
  - unsafe confined to abstractions
- 强调功能等价验证：
  - 编译通过
  - QEMU `ip link` smoke 通过
- 不强调“自动翻译器产物”
