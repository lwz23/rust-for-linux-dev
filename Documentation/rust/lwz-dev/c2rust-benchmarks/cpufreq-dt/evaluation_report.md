# `cpufreq-dt` evaluation report

- 生成日期：2026-03-20
- 当前等级：`prototype-grade`
- 目标等级：`engineering-grade`
- true blind base：`14f47156cf390606eb719da9ad1058f87af0a291`
- true Rust 引入提交：`06149d8f2216894cee86106c701d13141948f159`

## 结论

`cpufreq-dt` 这次已经完成了：

- true pre-Rust 基线校准；
- blind-first；
- hardening；
- hardening 之后的 official-style compare；
- benchmark 工件和方法学回写。

但它当前仍然只能宣称 `prototype-grade`，不能宣称 `engineering-grade`
或更高。原因不是 blind-first 或 hardening 没做完，而是这次没有建立可信的
DT-backed runtime harness，无法对 probe 失败回滚、policy online/offline、
真实 OPP 切换和平台回调路径给出可信运行证据。

## 主要证据

### 工程闭环

- `CONFIG_CPUFREQ_DT_RUST` 已提供 C / Rust 切换构建。
- `drivers/cpufreq/rcpufreq_dt.rs` 维持零 `unsafe`。
- `git diff --check` 通过。
- `drivers/cpufreq/rcpufreq_dt.o` 在
  `/home/lwz/rfl-dev/build-cpufreq-dt-blind` 中可成功构建。

### hardening 结果

- 生命周期模型从更像 C 的全局/早期聚合思路，收敛为每个 policy 自持
  `opp_table + freq_table + cpumask + config_token + clk`。
- probe 侧进一步收敛为 devres 托管的 cpufreq registration，而不是平台驱动私有字段手工持有。
- `rust/kernel/platform.rs` 不再向安全调用者暴露泛型 `platform_data<T>()`。
- `rust/kernel/cpufreq.rs` 使用 `PolicyResources` 自动配对
  `driver_data`、`freq_table` 和 `clk` 的附着与清理，驱动层不再需要
  `unsafe set_freq_table / set_clk`。
- blind-first 期为全局 registry 方案添加的额外 helper 和不必要的
  `Send/Sync` 已在 compare 后清理。

### 与 C 金标准的对齐

- 保留了 `cpufreq-dt.h` 平台数据里的
  `have_governor_per_policy / suspend / resume / get_intermediate / target_intermediate`
  行为入口。
- `set_boost` 仍通过 `cpufreq_boost_set_sw()` 保持与冻结 C 实现一致。
- transition latency fallback 仍使用 `CPUFREQ_ETERNAL`，而不是向官方当前树行为靠拢。

## 与官方实现的差距

### 已对齐的点

- 对象模型已经回到每 policy 私有状态，而不是 probe 期全局域注册表。
- probe/remove 生命周期已经表达为平台设备驱动 + cpufreq 注册对象的配对关系。
- OPP table、freq table、clk、config token 的 ownership 已集中在 policy 私有对象。
- 驱动层保持零 `unsafe`。

### 仍未对齐的点

- 官方当前树使用 OF match table；本次实现仍保留更接近冻结 C 金标准的
  platform-device 注册外形。
- 官方当前树没有保留 `cpufreq-dt.h` 的平台数据扩展；本次实现保留它们是为了
  忠实于 C 金标准，而不是为了模仿当前主线风格。
- 官方当前树的 `rust/kernel/cpufreq.rs` 仍让驱动显式完成
  `set_freq_table / set_clk`；本次为了把驱动层压到零 `unsafe`，引入了本地
  `PolicyResources` 和 `RegistrationConfig`。这在本树里更安全，但不自动等于
  “更像官方 API”。
- blind base 缺少当前树的 `CpuId` 风格 API，因此本次 cpufreq API 仍用 `u32`
  表示 CPU 标识。

### 模块实现差距

- 如果未来旧树也具备更成熟的 OF / cpufreq 抽象，本模块仍可继续向官方实现的
  注册和匹配风格收敛。
- `RegistrationConfig` 目前只为 `cpufreq-dt` 平台数据而生，还不是经过多模块验证的
  通用接口。

### 工作流程差距

- blind-first 初期过于字面复刻 C 结构，说明流程里还缺一条更早的约束：
  在引入全局 registry、探测期聚合和宽 helper 之前，必须先证明
  “per-policy state + devres/platform registration” 不能表达目标驱动。
- 之前流程把 “与官方实现对比” 视为可选收尾；这次说明它必须成为 hardening 后的
 强制步骤，而且必须显式回答“哪里还不够主线风格”。
- safe API 评估以前只和 C 金标准比，这不够。现在必须再比一次官方当前树是否暴露了
  同样的能力。

## 对流程的反推修订建议

- 任何 blind-write 任务在编码前都必须先校准 true blind base。若本地历史经过 graft
  或 rewritten，不能直接沿用用户给的 SHA 当作 blind 结论依据。
- hardening 之后新增一个强制步骤：`official-style delta review`。
- `official-style delta review` 必须分别输出：
  - 模块实现差距；
  - 工作流程差距；
  - 哪些差距属于旧树约束；
  - 哪些差距是流程不够严格导致。
- 在 helper / safe API 审计里新增一条显式问题：
  “这个接口只是让当前任务更快，还是它真的是旧树必需的最窄安全边界？”
- 如果缺少可信 runtime harness，则等级结论必须立刻封顶，不能因为 compare 很漂亮就越级。

## 阻塞项

- 没有可信 DT-backed runtime harness。
- 仍未验证真实 policy online/offline 与 probe 失败回滚的运行时轨迹。
- `rust/kernel/cpufreq.rs` 里的新接口还只被单模块验证。

## 当前允许与不允许的表述

允许写：

- blind-write 已从 true pre-Rust 基线完成；
- hardening 已完成；
- official-style delta review 已完成；
- 这次样本已经逼出了新的手册规则。

不允许写：

- 已达到 `engineering-grade`；
- 已达到 `mainline-grade`；
- 已建立可信运行时等价；
- 当前新增 cpufreq 抽象已经可以直接视为官方通用形态。
