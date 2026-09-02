# Lodestone Protocol (lodestone-md) 开发导航牌（PLAN）

> **版本**：v1.2（P1 收官流转，2026-09-02）
> **状态**：⏳ P2 — 生态对齐（T1 待启动）
> **上一阶段**：P1 消费端与投影（✅ 2026-09-02，详见 GROWTH.md）
> **分支**：main
> **所属方法论**：phyt-DNA v1.0（PLAN 动态流转闭环）
> **规则**：本文件只含当前阶段 + 下一阶段预览 + 阶段总览地图。完成阶段 → GROWTH.md。总行数 ≤150，超出触发历史迁移。

---

## 1. 当前阶段：P2 — 生态对齐

> **状态**：⏳ T1 待启动。
> **前置依赖**：P0/P1（✅）。

### 1.1 目标

| 任务 | 内容 | 入口 | 状态 |
|---|---|---|---|
| T1 | 一致性基准发布：fixtures 作为跨实现验证器语料（fixture 头注释标注 §来源） | docs/spec/contract.md §2 | ⏳ |
| T2 | 性能验证：大文档解析基准 + 附录 A 启发式快速扫描（L1-only）路径评估 | docs/spec/philosophy.md 公理二 | ⏳ |
| T3 | v2.0 议程反馈：观察项 γ（derive 误用率）、附录 C 悬空引用清扫 | 协议附录 C | ⏳ |
| T4 | 发布准备：crates.io 发布（`mddag` crate）+ 版本标签 v1.3.0 | ADR-0001 | ⏳ |

### 1.2 代码真相源

- **T1**：fixtures 已在 `tests/fixtures/`（22 组），待补 §来源 标注。
- **T2**：`src/` 无基准与启发式路径。
- **T3**：无（v1.x 不实现 v2.0 议程，仅收集反馈）。
- **T4**：Cargo.toml 已备发布元数据（repository/license/description）。

### 1.3 项目特有验收状态

| 验收项 | 状态 | 落地位置 |
|---|---|---|
| cargo test 全绿 | ✅（P0 起保持，现 20 项） | CI |
| cargo clippy 0 warning | ✅（P0 起保持） | CI |
| §10 数值逐项吻合 | ✅（P0） | tests/golden.rs |
| 重复解析输出逐字节一致 | ✅（P0） | tests/golden.rs |
| L1/L2 加载语义 | ✅（P1） | tests/p1.rs |

### 1.4 入口 ADR

- **ADR-0001**：Rust 参考实现与解析底座选型（Active）
- **P2 新增 ADR 候选**：投影 API 是否随 crate 首发进入公共面（若是，需补契约章节）；crates.io 发布名与版本策略

### 1.5 已确认决策点

| # | 决策点 | 决议 | 状态 |
|---|---|---|---|
| D1 | 边 failure 语义 | §8.2 示例码三分支；pending/nascent 边 failure=null | ✅ |
| D2 | 输出契约扩展 | `NodeEntry.title` 为 Append-Only 扩展 | ✅ |
| D3 | 投影标签优先级 | dangling > cyclic > redundant > coherent > pending > nascent | ✅ |

### 1.6 P2 进度

| 任务 | 内容 | 状态 | 测试 |
|---|---|---|---|
| T1 | 一致性基准发布 | ⏳ 待启动 | - |
| T2 | 性能验证 | ⏳ 待启动 | - |
| T3 | v2.0 反馈收集 | ⏳ 待启动 | - |
| T4 | 发布准备 | ⏳ 待启动 | - |

### 1.7 验收标准

- T1：每个 fixture 头注释含规范章节引用，fixtures/ 可独立作为一致性语料发布。
- T2：基准报告（时间/内存）入库；启发式路径与全量解析在含围栏文档上输出一致（若采纳）。
- T3：观察项 γ 记录至 docs/decisions/ 或 ISSUE。
- T4：crate 发布成功，`mddag --version` 可打印 v1.3.0。

---

## 2. 下一阶段预览：P3 — 协议演进支撑

- 跨文档引用语法预研（v2.0 D 系列底座进入试验分支）；
- 与观察项相关的协议修订建议回馈给规范（ADR 通道）。

---

## 3. 阶段总览

| 阶段 | 名称 | 状态 | 产出 |
|---|---|---|---|
| P0 | 参考解析器骨架与 Golden Fixture | ✅ | 解析器 + 22 组 Fixture（14 码全覆盖） |
| P1 | 消费端与投影 | ✅ | 投影标签 + L1/L2 API + CLI 5 模式 + 审查面 |
| P2 | 生态对齐 | ⏳ 当前 | 一致性基准、性能验证、发布 |
