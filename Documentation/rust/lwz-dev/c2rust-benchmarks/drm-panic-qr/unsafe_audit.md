# `drm-panic-qr` unsafe audit

- 生成日期：2026-03-21
- `pipeline_mode=calibration`
- `conversion_stage=hardening`
- `style_alignment=benchmark-calibrated`
- `runtime_state=unvalidated`
- `stage ceiling=hardening`
- `validation gap`：
  - 没有可信 panic runtime harness
  - 本机 `libclang 14.0.0` 阻塞 Rust-enabled build

## 当前结论

驱动层 `drivers/gpu/drm/drm_panic_qr.rs` 保持零 `unsafe`。

当前仅有 3 个 `unsafe` 点，全部压缩在
`rust/kernel/drm/panic_qr.rs`，且都属于原始 FFI 边界：

- `slice::from_raw_parts_mut(data, data_size)`
- `slice::from_raw_parts_mut(tmp, tmp_size)`
- `CStr::from_char_ptr(url)`

没有新增 `unsafe impl Send/Sync`，也没有新增 helper。

## 不变式

### `data` buffer

- 前置条件：C 侧传入的 `data` 非空、可写、长度至少为 `data_size`。
- 后置条件：Rust 只在本次调用期间独占地把它视作可变切片，并把最终 QR bitmap 写回。

### `tmp` buffer

- 前置条件：C 侧传入的 `tmp` 非空、可写、长度至少为 `tmp_size`。
- 后置条件：它只作为临时 scratch / codeword / segment buffer 使用，不跨调用保留。

### `url`

- 前置条件：`url` 为 `NULL` 或合法 NUL 结尾字符串。
- 后置条件：Rust 只做只读借用，不缓存指针、不延长生命周期。

## Safe API 破坏面

- safe 驱动层看不到任何 raw pointer 转换入口。
- 组件层没有 `Send/Sync` 扩展，也没有可移动的注册对象。
- QR 逻辑只暴露与 `drm_panic.c` 对齐的两个 C ABI 函数。
