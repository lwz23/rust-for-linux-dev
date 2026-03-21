# AGENT_PLAYBOOK

- 版本：2026-03-21
- 角色：给 agent 的执行手册
- 使用方式：把本文件、`HARD_RULES.md`、`ACCEPTANCE_CHECKLIST.md`、
  `EVALUATION_LADDER.md` 和 `tooling-spec/` 一起提供给 agent 或
  orchestration 层

## 目标

在给定一个 Linux 内核 C 驱动模块后，agent 必须沿着受约束的 Rust 化流程工作，
输出结构化工件与 verifier 可消费的中间产物，而不是只产出“能编译的 Rust 文件”。

## 两条流水线

### 生产流水线

- 目标：面对未 Rust 化的 C 驱动，从 0 生成到 `hardening candidate`
- 约束：不得读取官方 Rust 实现
- 默认状态轴：
  - `conversion_stage`
  - `style_alignment`
  - `runtime_state`

### 校准流水线

- 目标：对 benchmark 或历史样本做 hardening 后的官方风格对照
- 约束：只有在 hardening 后才能读取官方 Rust 实现
- 产物：`calibration-delta.json`、新规则、新模板、新 verifier 项

## 固定 artifact

每次任务都必须逐步生成或刷新：

- `dut-spec.json`
- `c-structure-map.json`
- `binding-gap-audit.json`
- `helper-audit.json`
- `pattern-selection.json`
- `unsafe-obligations.json`
- `difference_ledger.yaml`
- `verification-summary.json`

仅校准流水线额外生成：

- `calibration-delta.json`

## 固定阶段

### 阶段 0：环境冻结与模式选择入口

必须做：

- 确认工作树、分支、remote、构建目录
- 记录 DUT、事件发生器、C 金标准、当前 `pipeline_mode`
- 创建或刷新 `module_manifest.yaml`、工作日志和 `dut-spec.json`
- 若目标是历史基线，先把 host toolchain / build-system drift 记成
  `old tree constraint` 候选，不能直接当成驱动行为差异

禁止做：

- 还没确认 DUT 就开始写代码
- 在生产流水线里读取官方 Rust 实现

### 阶段 1：静态前端

必须做：

- 扫描 C 驱动、Kconfig、Makefile、头文件、alloc/free、register/unregister、
  get/put、devm、driver_data、callback table、function pointer、list_head、
  container_of、configfs、request PDU、queuedata
- 生成 `c-structure-map.json`

目标：

- 先做 kernel-aware 对象分类，再让 agent 继续生成

### 阶段 2：Kbuild / Kconfig

必须做：

- 保留 C 实现
- 让 Rust 实现以 config 选择器切换构建，或明确记录为何必须采用其他集成形态
- 记录 C / Rust 模式配置组合

### 阶段 3：bindgen 缺口审计

必须做：

- 先查主 bindings 能否覆盖需求
- 生成 `binding-gap-audit.json`
- 只把真正缺失的 inline / 宏盲区记为 helper 候选
- 若历史树因现代工具链而出现额外兼容补丁，单独记录，不得混成 DUT 结论

### 阶段 4：helper 审计

必须做：

- 生成 `helper-audit.json`
- 只允许最小函数桥接 helper

禁止做：

- 用 helper 承载新的驱动私有 C 类型
- 把 helper 当默认出口

### 阶段 5：pattern 选择与抽象骨架

必须做：

- 根据 `c-structure-map.json` 和 `pattern-library.yaml` 选择模式
- 生成 `pattern-selection.json`
- 把 raw FFI 和 `unsafe` 封进 `rust/kernel/*` 或极小 helper
- 对 intrusive / registered / callback-owned 对象优先使用
  `Pin + Opaque + state machine`
- 若 configfs child 持有已注册对象，必须把 `power` 路径、`drop_item`、
  final `release` 一起纳入生命周期建模

若模式无法稳定归类：

- 仍可继续生成
- 但必须输出：
  - `pattern unresolved`
  - `manual rule needed`
  - `stage ceiling reached`

### 阶段 6：blind-first / driver synthesis

必须做：

- 先得到驱动层零 `unsafe` 的可运行版本
- 在 `verification-summary.json` 中把 `conversion_stage` 标成 `blind`

禁止做：

- 把第一次跑通直接写成最终结论

### 阶段 7：hardening

必须做：

- 收紧 safe API
- 检查 private data 访问模型
- 检查 helper 是否仍必要
- 检查 `unsafe impl Send/Sync`
- 检查 registration、pinning、drop pair、ownership pair
- 若使用 configfs child 承载已注册对象，显式检查：
  - `drop_item` 与 final release 是否重复/漏配
  - powered-on 状态下配置更新是否被阻止
  - live object 是否仍可被 safe code 移动
- 更新 `unsafe-obligations.json`

### 阶段 8：验证

必须做：

- `git diff --check`
- 默认构建
- 驱动层 `unsafe` 扫描
- helper 审计
- `Send/Sync` 审计
- private access 宽度审计
- registered object pinning 审计
- ownership / drop pair 审计
- 对 configfs child + registered object 额外执行 `drop_item/release` 配对审计
- 对历史基线额外执行 toolchain compatibility 归类审计
- 更新 `difference_ledger.yaml`
- 更新 `verification-summary.json`

### 阶段 9：校准流水线的 official compare

只在 `pipeline_mode=calibration` 时执行：

- 在 hardening 之后读取官方 Rust 实现
- 比较对象模型、safe API、`Pin/Opaque/state machine`、helper 残留、
  `Send/Sync`、private data 访问模型、request completion、queue-data ownership
- 生成 `calibration-delta.json`

输出必须拆成：

- 已对齐项
- 未对齐项
- 旧树约束
- 工作流程缺陷

### 阶段 10：文档与提交

必须做：

- 更新 `evaluation_report.md`
- 更新 `acceptance_checklist.md`
- 更新工作日志
- 如有新教训，回写 `HARD_RULES.md` 与 `tooling-spec/`
- 按可回滚阶段拆 commit

提交正文必须写：

- `Why`
- `What`
- `Verification`

## 失败处理

如果构建或测试失败，agent 必须先分类：

- 代码 bug
- helper / bindings 缺口
- 旧树约束
- 测试基础设施缺口
- 模式未决
- 历史 toolchain 漂移

然后再决定：

- 改代码
- 改抽象
- 新增规则
- 记录为阻塞项

不允许直接跳过失败并继续宣称通过。
