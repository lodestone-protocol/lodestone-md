# Lodestone Protocol (lodestone-md) 开发导航牌（PLAN）

> **版本**：v1.1（P0 收官流转，2026-09-02）
> **状态**：🚧 P1 — 消费端与投影（T1–T4）
> **上一阶段**：P0 参考解析器骨架与 Golden Fixture（✅ 2026-09-02，详见 GROWTH.md）
> **分支**：main
> **所属方法论**：phyt-DNA v1.0（PLAN 动态流转闭环）
> **规则**：本文件只含当前阶段 + 下一阶段预览 + 阶段总览地图。完成阶段 → GROWTH.md。总行数 ≤150，超出触发历史迁移。

---

## 1. 当前阶段：P1 — 消费端与投影

> **状态**：🚧 T1 待启动。
> **前置依赖**：P0 解析器与 Golden Fixture（✅）。

### 1.1 目标

| 任务 | 内容 | 入口 | 状态 |
|---|---|---|---|
| T1 | 视图投影标签（coherent/pending/nascent/redundant/dangling/cyclic）作为库函数输出（附录 A，非规范性但消费方第一刚需） | docs/spec/contract.md | ⏳ |
| T2 | L1 骨架 / L2 定点正文读取 API（按需加载第一公民：`skeleton()` / `body(lines, id)`） | docs/spec/philosophy.md 公理二 | ⏳ |
| T3 | CLI 增强（`--skeleton` / `--body <id>`） | 同上 | ⏳ |
| T4 | 人类审查面：警告列表与分歧域（refute 域）摘要输出 | docs/spec/position.md §3 | ⏳ |

### 1.2 代码真相源

- **T1–T4**：待实现（投影模块 `src/projection.rs` 尚未创建；CLI 仅支持全文解析输出）。

### 1.3 项目特有验收状态

| 验收项 | 状态 | 落地位置 |
|---|---|---|
| cargo test 全绿 | ✅（P0 起保持） | CI |
| cargo clippy 0 warning | ✅（P0 起保持） | CI |
| §10 示例数值逐项吻合 | ✅（P0） | tests/golden.rs |
| 重复解析输出逐字节一致 | ✅（P0） | tests/golden.rs |

### 1.4 入口 ADR

- **ADR-0001**：Rust 参考实现与解析底座选型（Active）
- **P1 新增 ADR 候选**：投影标签是否进入 lib 公共 API（若涉及契约扩展，新建 ADR-0002）

### 1.5 已确认决策点

| # | 决策点 | 决议 | 状态 |
|---|---|---|---|
| D1 | 边 failure 语义 | §8.2 示例码三分支；pending/nascent 边 failure=null（ADR-0001 已记录） | ✅ |
| D2 | 输出契约扩展 | `NodeEntry.title` 为 Append-Only 扩展（ADR-0001 已记录） | ✅ |

### 1.6 P1 进度

| 任务 | 内容 | 状态 | 测试 |
|---|---|---|---|
| T1 | 投影标签 | ⏳ 待启动 | - |
| T2 | L1/L2 API | ⏳ 待启动 | - |
| T3 | CLI 增强 | ⏳ 待启动 | - |
| T4 | 审查面摘要 | ⏳ 待启动 | - |

### 1.7 验收标准

- T1：六种投影标签与附录 A 条件表逐项一致（Fixture 断言）。
- T2：`skeleton()` 输出 ≤ 节点表+边+诊断；`body()` 按 id 返回正文文本（与 body_start/body_end 一致）。
- T3：`mddag --skeleton <file>` / `mddag --body <id> <file>` 行为正确。
- T4：分歧域摘要列出全部 refute 边及端点状态。

---

## 2. 下一阶段预览：P2 — 生态对齐

- 一致性基准发布（fixtures 作为跨实现验证器语料）；
- 性能验证（大文档解析基准、附录 A 启发式快速扫描路径评估）；
- v2.0 议程反馈收集（derive 误用率观察项 γ 等）。

---

## 3. 阶段总览

| 阶段 | 名称 | 状态 | 产出 |
|---|---|---|---|
| P0 | 参考解析器骨架与 Golden Fixture | ✅ 完成 | mddag v1.3 解析器 + 22 组 Fixture（14 码全覆盖） |
| P1 | 消费端与投影 | 🚧 当前 | 投影 API + L1/L2 加载 + CLI 增强 |
| P2 | 生态对齐 | ⏳ | 一致性基准发布、性能验证、v2.0 反馈 |
