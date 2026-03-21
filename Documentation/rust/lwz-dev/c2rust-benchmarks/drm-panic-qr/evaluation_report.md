# `drm-panic-qr` evaluation report

- 生成日期：2026-03-21
- `pipeline_mode=calibration`
- `conversion_stage=hardening`
- `style_alignment=benchmark-calibrated`
- `runtime_state=unvalidated`
- `stage ceiling=hardening`
- `validation gap`：
  - 没有可信 panic/NMI runtime harness
  - 本机 `clang/libclang 14.0.0` 低于该树要求，阻塞 Rust-enabled build/KUnit
  - blind lane 在正式 S9 之前发生了参考污染

## 事实

- 给定的 `pre-Rust base commit` 和 `Rust introduction commit` 都不是可信的路径级边界，因为它们已经包含 `drivers/gpu/drm/drm_panic_qr.rs`。
- 通过临时 metadata-only clone 校准后，真实路径级引入点是 `cb5164ac43d0fc37ac6b45cabbc4d244068289ef`，其父提交 `8f4eca6ac52a72181b4f054d4ef6289a5d8cfa5d` 才是可信 blind base。
- `feature/nlmon-rust` 与 `rust-next` 在 `drm_panic.c`、DRM `Kconfig`、`Makefile`、`drm_panic_test.c` 上 blob 相同，因此工作树从 `feature/nlmon-rust` 分出只影响文档骨架，不改变 DUT 代码面。
- 该 DUT 是组件 benchmark，不是完整驱动：真正的约束来自 panic-context、no-allocation、caller/preallocated buffer ownership，以及 `drm_panic.c` 内部 seam。
- blind candidate 实现了零 `unsafe` 的驱动层导出壳，并把 raw FFI 压缩到 `rust/kernel/drm/panic_qr.rs`。

## 结论

这次执行可以写成：

- 已完成 blind-first 组件初版；
- 已完成 hardening 级别的边界收紧；
- 已完成官方风格校准；
- 但尚未形成 runtime 工程闭环。

这次执行不能写成：

- `engineering-grade`
- `mainline-grade`
- `runtime_state=validated`
- “完全 reference-free 的 blind-first 结果”

## 与官方实现的差距

- 官方实现用自定义轻量 QR 引擎，固定 checkerboard mask；blind candidate 用改编的通用 QR 引擎并自动选 mask。
- 官方 URL 模式使用 7-byte -> 17-digit 的 FIDO 风格数值打包；blind candidate 用逐字节 3 位十进制打包，因此 decoder 兼容性和容量都不一致。
- 官方旧树文件把 `unsafe extern "C"` 导出直接放在驱动文件里；blind candidate 把 unsafe FFI 收进了 `rust/kernel/drm/panic_qr.rs`，驱动层保持零 `unsafe`。
- 两者都保持了同一个 current-tree 组件边界：预分配、zlib、draw、init/exit 仍由 `drm_panic.c` 拥有。

## 对流程的反推修订建议

- `drm-panic-qr` 证明了“组件级 Rust 化”不应默认套用完整驱动模板；`panic-context-pure-component` 作为首选 pattern 是合理的。
- benchmark orchestration 需要把“路径级 intro/base 校准”提升成硬前置门，而不是只依赖用户提供 commit。
- pre-S9 参考污染需要更强的自动防护：像 `git diff` 这种对已重写文件的命令，也应视为潜在官方内容泄露源。
- 当本机 toolchain 不满足 `clang/libclang` 最低版本时，流程应更早把 build/KUnit 判为“环境/测试基础设施缺口”，避免把时间浪费在后续无效尝试上。
