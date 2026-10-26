# Bitmap：UML + C2Rust 实验代码与运行记录

Linux 基线：`8b1872169d7f2e969122a1d541700e8154ba2a58`。C2Rust 基线：`e9ccd715597f7d63e8635629f4450e040a230118`。
当前分支的 `rust/kernel/bitmap.rs` 是测试使用的最终源码；`lib/find_bit.c` 保留真实内核输入。C2Rust 导出器的实际改动在配套 C2Rust fork，本目录另保留原始 patch 以便核对。

## 阅读顺序

1. [导出器补丁](evidence/c2rust-exporter.patch)
2. [find-bit 后处理](evidence/find-bit/post_one.sh) 与 [最终 find_bit.rs](evidence/find-bit/patched/src/find_bit.rs)
3. [真实 helper 包装](evidence/experiment/wrap/bitmap_bindings.c)、[helper 后处理](evidence/experiment/adapt_bitmap_bindings.py)、[最终 bitmap_bindings.rs](evidence/experiment/wrap-out/src/bitmap_bindings.rs)
4. [RFL Bitmap 与原有测试](../../rust/kernel/bitmap.rs)
5. [原始运行入口](evidence/experiment/run_bitmap_miri.sh) 与 [三模式运行参数](evidence/experiment/modes/lib.sh)
6. [TB 输出](evidence/experiment/modes/tb-all6.log)、[关闭 SB 输出](evidence/experiment/modes/nosb-all6.log)、[逐项 SB 日志目录](evidence/experiment/modes)
7. [纯 Rust ZST 探针及三模式原始日志](evidence/experiment/zst-probe)

## 复现前提与边界

历史脚本、compile_commands.json 和 helper include 保留实验机绝对路径。复跑前须按自己的 Linux 目录修改路径，准备 UML 配置和生成头、Clang 21.1.8、C2Rust 0.22.1、nightly-2026-07-21 对应的 KMiri 与 sysroot。本目录不携带构建产物、解释器二进制或 sysroot，不能仅凭 checkout 直接运行。

`patch_bitmap_rs.py` 是历史增量脚本，依赖 find-bit 已接入的中间状态；当前分支已放入最终 bitmap.rs，不要再对它重复执行该脚本。`run-e3-e4.sh` 含历史校准实验；单独应用 find-bit 后处理请读 `post_one.sh`。

历史结果：原生 UML 6/6（10-21）；TB 6/6、关闭 SB 6/6（10-26）；默认 SB 逐项五个 UB、bitmap_vec_new 通过。额外矩阵地址耗尽日志保留，不计为六项通过。本次整理和推送没有重新构建或重跑测试。

__ffs/__fls 是人工 Rust 替换；helper 使用已生成的 generic 实现；fortify 关闭，原子性与真实内核分配器不在验证范围，分配保留 Box。通过不等于完整 C 语义/provenance 保真。
