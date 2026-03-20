# Local AGENT_PLAYBOOK Addendum

- 版本：2026-03-20
- 目的：补充 `ax88796b` benchmark 在“与官方 Rust 实现对照”阶段得到的新流程经验

## 阶段 6.5：官方 Rust 实现对照

如果当前树里已经存在官方 Rust 实现，agent 在 hardening 之后必须做两层对照，
而不是只看文本 diff：

- `功能差异`
- `安全性差异`

### 功能差异

必须先回答：

- callback table 形状是否一致
- C 金标准覆盖到的功能路径是否一致
- runtime 场景下 C / benchmark Rust / 官方 Rust 的行为是否能由同一语义解释

如果 benchmark Rust 与 C 已 runtime 对齐，而与官方 Rust 的差异只剩模块元数据、
局部 helper 组织、注释或命名，这类差异按“非功能差异”处理，不得夸大成行为偏差。

### 安全性差异

必须单独审视：

- safe API 是否更宽或更窄
- `unsafe impl Send/Sync` 被放在哪个对象边界
- safe wrapper 是否重复手写了 bindgen 已能表达的布局知识
- 官方实现的“更通用”到底是必要抽象，还是尚未 harden 完的宽接口

### 强制输出

在 benchmark 工件和最终结论里，必须明确写出：

- 哪些差异属于功能等价前提下的工程差异
- 哪些差异属于安全收紧或 soundness 修复
- 官方抽象相对当前抽象“好在哪里”
- 当前抽象相对官方抽象“更稳在哪里”

## `ax88796b` 专项经验

### 官方抽象的优势

- `set_speed(u32)` 让驱动代码更接近 C 版，移植时阻力更小。
- 把线程安全证明放在更通用的容器上，能让模块接入路径更直接。
- 手写 bitfield 读取在 bindgen accessor 尚不成熟时更容易快速落地。

### 当前 benchmark 抽象的优势

- `set_basic_speed(BasicSpeed)` 把 safe 写入面限制在当前驱动真实需要的 10/100 范围。
- `Send/Sync` 证明收敛在 `Registration`，比落在 `DriverVTable` 上更贴近真正跨线程的对象边界。
- 直接使用 bindgen 生成的 `phy_device` accessor，避免 safe wrapper 复制字段布局知识。

### 流程规则

- 发现 runtime 失配时，优先怀疑抽象层是否复写了 ABI / layout 知识，而不是先怀疑驱动逻辑。
- 若官方实现更宽，先问“这份宽度有没有被当前 benchmark 用到”，再决定是否跟随。
- 若官方实现更窄或更稳，优先吸收其对象模型与不变式写法，再看 benchmark 分支是否需要收敛。
