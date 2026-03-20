# 面向内核驱动 C2Rust 的评价阶梯

- 版本：2026-03-20
- 适用范围：`/home/lwz/rfl-dev/linux`
- 核心目的：把“能编译、能跑、看起来没问题”拆成可审计的等级，而不是继续混在一起。

## 使用原则

- 每个模块都必须只声明自己已经达到的最高等级，不能预支更高结论。
- 升级等级时，必须同时给出证据文件路径，而不是口头判断。
- 若模块触发更低等级的阻塞项，应立即降级，不允许保留过高标签。

## 四级定义

### 1. `prototype-grade`

定义：

- C / Rust 实现已经能切换构建。
- 驱动层保持零 `unsafe`。
- 至少有一条最小运行证据，证明它不是纯静态玩具代码。

必须具备：

- `module_manifest.yaml`
- 最小工作日志
- 至少一次成功构建记录
- 至少一次最小运行或装载证据

不允许宣称：

- 功能已经完整等价
- safe API 已经达到主线工程标准
- 可以直接发外部补丁

### 2. `engineering-grade`

定义：

- 模块不只是“跑起来”，而是已经形成可重复的工程验证闭环。

必须新增：

- `evaluation_report.md`
- `difference_ledger.yaml`
- `unsafe_audit.md`
- `acceptance_checklist.md`
- 完整 debug build / runtime / diff 验证
- 无未分类功能差异

必须满足：

- 原始 C 实现仍为功能金标准
- 所有差异都被分类为：
  - `安全修复`
  - `旧树约束`
  - `可接受工程差异`
  - `不可接受功能差异`

升级阻塞项：

- 构建或运行验证仍依赖手工口头解释
- `unsafe` 审计只写前置条件，不写后置条件
- 存在未解释的行为差异

### 3. `mainline-grade`

定义：

- 模块已经经历 hardening，safe API 和对象模型不再停留在 blind-first 原型期。

必须新增：

- 对象模型已经按当前树成熟 Rust 风格收紧
- intrusive / registered / callback-owned 对象默认使用
  `Pin + Opaque + state machine`
- `Send/Sync`、helper 生命周期、private data 访问模型都经过单独审查

必须满足：

- 若存在当前树官方 Rust 版本，必须完成 reference-based compare
- 若私有区承载 pinned registered state，safe API 不得继续暴露宽泛 `&mut Private`
- helper 若仍保留，必须在 ledger 里明确标记为过渡或旧树约束

升级阻塞项：

- safe API 仍能让调用者移动已注册对象
- `unsafe impl Send/Sync` 没有结构性证明
- 仍把第一次跑通的 blind-first 设计直接当最终抽象

### 4. `upstreamable-grade`

定义：

- 模块不仅在本地闭环里成立，而且已经具备“对维护者可解释”的理由链。

必须新增：

- 对外 RFC 决策备忘录
- 面向维护者的简洁理由摘要：
  - 为什么需要这份 Rust 化
  - 为什么安全边界是可信的
  - 为什么行为等价判断成立
  - 为什么抽象边界不会把旧树过渡设计硬塞给上游

必须满足：

- 至少 1 个新模块已达到 `mainline-grade`
- 当前树内至少 3 个 benchmark 证据包完成
- 没有未解释的功能差异
- 结论不依赖“测试暂时没炸”

升级阻塞项：

- 样本过少，无法说明方法具有迁移性
- 对外摘要仍停留在本地过程描述，没有维护者视角的理由链

## 等级晋升门槛

从 `prototype-grade` 到 `engineering-grade`：

- 补齐全部标准化工件
- 补齐 build/runtime/diff/unsafe 四类证据

从 `engineering-grade` 到 `mainline-grade`：

- 完成 hardening
- 完成 reference-based compare（若存在官方 Rust 版本）
- 完成 helper 与 `Send/Sync` 审查

从 `mainline-grade` 到 `upstreamable-grade`：

- 形成对外沟通包
- 明确社区目标对象与邮件/RFC 范围

## 结论表述模板

- `prototype-grade`：只允许写“原型已跑通，尚未完成完整工程闭环”。
- `engineering-grade`：允许写“已通过本地工程闭环验收”。
- `mainline-grade`：允许写“对象模型已完成 hardening，并与当前树成熟 Rust 风格对齐”。
- `upstreamable-grade`：允许写“已具备对外 RFC/补丁讨论条件”。
