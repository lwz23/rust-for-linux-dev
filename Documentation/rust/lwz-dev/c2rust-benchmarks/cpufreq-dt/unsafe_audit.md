# `cpufreq-dt` normalized unsafe audit

- 生成日期：2026-03-20
- 当前等级上限：`prototype-grade`

## 当前结论

`drivers/cpufreq/rcpufreq_dt.rs` 维持零 `unsafe`。当前这次 Rust 化里的
`unsafe`、raw pointer 和 FFI 边界主要集中在：

- `rust/kernel/cpu.rs`
- `rust/kernel/cpufreq.rs`
- `rust/kernel/platform.rs`
- `rust/kernel/opp.rs`

本轮 hardening 和 compare-driven cleanup 之后，没有再发现“安全驱动无需写
`unsafe` 也能直接打破已注册 policy 资源生命周期”的明显路径。

## 重点不变式

### CPU device lookup

- 前置条件：`get_cpu_device()` 返回的设备指针必须仍指向内核持有但不释放的 CPU 设备对象。
- 后置条件：安全调用者不能把该借用逃逸进长生命周期状态。
- safe 破坏面：`cpu::with_device()` 只允许 callback-scoped 借用。

### Platform data cast

- 前置条件：`platform_data` 必须真的是 `struct cpufreq_dt_platform_data`
  兼容布局，且生命周期受平台设备约束。
- 后置条件：只有 `cpufreq-dt` 自己能拿到这份窄化后的解释。
- safe 破坏面：泛型 `platform_data<T>()` 已移除，外部安全代码不能再任意指定 `T`。

### Policy private data and attached resources

- 前置条件：`driver_data`、`freq_table` 和 `clk` 的附着必须来自同一份活着的私有对象。
- 后置条件：exit/drop 时必须成对清理，不能留下悬空 policy 字段。
- safe 破坏面：`PolicyResources` 与 `set_data/clear_data` 的配对把附着和清理绑定在一起，
  驱动层不再单独手写资源安装。

### Registration and platform lifecycle

- 前置条件：注册对象在平台设备绑定期间必须保持有效。
- 后置条件：设备 detach 时必须自动撤销 cpufreq registration。
- safe 破坏面：`new_foreign_owned_with()` 把 registration 生命周期收缩进 devres，
  避免平台驱动手工持有宽状态。

### Raw platform callbacks

- 前置条件：`suspend/resume/get_intermediate/target_intermediate` 的函数指针必须满足
  C ABI 约定，且由平台数据拥有。
- 后置条件：Rust 侧只能原样转交，不得伪造更宽的安全调用模型。
- safe 破坏面：当前仍是“raw callback passthrough”，没有把这些指针包装成更宽的 safe API。

## 当前仍保留的审计关注点

- `cpufreq::Registration<T>` 的 `Send/Sync` 仍依赖结构性论证，而不是编译器自动推导。
- vtable 使用零尾初始化仍依赖当前树 ABI 约定。
- `RegistrationConfig` 和 `PolicyResources` 目前只被 `cpufreq-dt` 证明过，还不是多模块收敛后的抽象。
- 没有可信 DT-backed runtime harness，因此无法用运行证据覆盖 policy online/offline、
  probe 失败回滚和真实 OPP 切换。

## 与官方 compare 的补充结论

- 官方当前树驱动仍在驱动层使用 `unsafe from_cpu / set_freq_table / set_clk`。
- 本次本地抽象把这些边界进一步安全化了，但这只能说明“本地更窄”，不能自动推导出
  “已经是官方风格”。
- 因此本 benchmark 同时记录了安全改进和风格差距，避免把两者混为一谈。
