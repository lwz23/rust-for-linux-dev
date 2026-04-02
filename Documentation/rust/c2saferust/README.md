# C2SafeRust

该目录用于在干净 `rust-next` 基线上承载 `C2SafeRust` 工具的实验性产物。

当前只服务首个样本 `nlmon`，目标是把以下五个阶段固化为可复用工作流：

1. Kbuild 集成规划
2. bindings / helpers 审计
3. 安全抽象层计划
4. 驱动层生成
5. 动态 oracle 与差分验证

首批样本 artifact 位于：

- `Documentation/rust/c2saferust/nlmon/`
