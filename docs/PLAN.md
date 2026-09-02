# Lodestone Protocol (lodestone-md) 开发导航牌（PLAN）

> **版本**：v1.4（P2 推进 + 审查决议执行，2026-09-02）
> **状态**：🚧 P2 — 生态对齐（T1–T3 ✅，T5 ✅，T4 ⏳ 待发布授权）
> **上一阶段**：P1 消费端与投影（✅ 2026-09-02，详见 GROWTH.md）
> **分支**：main
> **所属方法论**：phyt-DNA v1.0（PLAN 动态流转闭环）
> **规则**：本文件只含当前阶段 + 下一阶段预览 + 阶段总览地图。完成阶段 → GROWTH.md。总行数 ≤150，超出触发历史迁移。

---

## 1. 当前阶段：P2 — 生态对齐

> **状态**：🚧 T1–T3/T5 ✅，T4（crates.io 发布）⏳ 待授权。
> **前置依赖**：P0/P1（✅）。

### 1.1 目标

| 任务 | 内容 | 入口 | 状态 |
|---|---|---|---|
| T1 | 一致性语料发布：MANIFEST.md 章节索引 + 14/14 码覆盖率表 | docs/spec/contract.md §2 | ✅ |
| T2 | 性能验证：examples/bench.rs（5000 节点 / 870KB / 全量单趟 731ms） | docs/spec/philosophy.md 公理二 | ✅ |
| T3 | v2.0 议程反馈：观察项 γ 登记 + 附录 C 悬空引用清扫 | 协议附录 C | ✅ |
| T4 | 发布准备：crates.io 发布 + 版本标签 v1.3.0 | ADR-0001 | ⏳ 待授权 |
| T5 | 审查决议执行：3 新 fixture + 3 断言测试 + MANIFEST/ADR-0002 更新 + 代码注释英文化（DNA 铁律 #6） | ADR-0002 审查备忘录 | ✅ |

### 1.2 代码真相源

- **T1**：`tests/fixtures/MANIFEST.md`（含 `MANIFEST.md` 排除逻辑）。
- **T2**：`examples/bench.rs`；结论入 ADR-0002。
- **T3**：ADR-0002 第五节观察项 γ。
- **T4**：Cargo.toml 元数据已备；发布动作需人工授权。
- **T5**：`slug_istanbul` / `boundary_setext` / `meta_blank_gap` fixture + 快照；golden.rs 断言测试；ADR-0002 第六节备忘录；src/tests/examples 注释已英文化。

### 1.3 项目特有验收状态

| 验收项 | 状态 | 落地位置 |
|---|---|---|
| cargo test 全绿 | ✅（现 23 项） | CI |
| cargo clippy 0 warning | ✅ | CI |
| §10 数值逐项吻合 | ✅ | tests/golden.rs |
| 重复解析输出逐字节一致 | ✅ | tests/golden.rs |
| L1/L2 加载语义 | ✅ | tests/p1.rs |
| 一致性语料可独立发布 | ✅（T1） | tests/fixtures/ |
| 代码注释零中文残留 | ✅（T5，扫描确认） | src/ tests/ examples/ |

### 1.4 入口 ADR

- **ADR-0001**：Rust 参考实现与解析底座选型（Active）
- **ADR-0002**：一致性语料与性能策略（Active，含审查备忘录）

### 1.5 已确认决策点

| # | 决策点 | 决议 | 状态 |
|---|---|---|---|
| D1 | 边 failure 语义 | §8.2 三分支 + pending/nascent failure=null | ✅ |
| D2 | 输出契约扩展 | `NodeEntry.title` Append-Only | ✅ |
| D3 | 投影标签优先级 | dangling > cyclic > redundant > coherent > pending > nascent | ✅ |
| D4 | 性能策略 | 全量单趟即 L1；不引入启发式双路径（ADR-0002） | ✅ |
| D5 | 审查决议 | P2/P3/P4/P5 误报驳回；P1/setext/空行间隔 3 fixture 吸收（ADR-0002） | ✅ |
| D6 | 注释语言 | 代码注释英文，交流中文（DNA 铁律 #6） | ✅ |

### 1.6 P2 进度

| 任务 | 内容 | 状态 | 测试 |
|---|---|---|---|
| T1 | 一致性语料 | ✅ | golden 保持 |
| T2 | 性能验证 | ✅ | examples/bench.rs |
| T3 | v2.0 反馈登记 | ✅ | ADR-0002 §五 |
| T4 | crates.io 发布 | ⏳ 待授权 | - |
| T5 | 审查决议执行 | ✅ | 23 测试（+3） |

### 1.7 验收标准

- T1：MANIFEST.md 双向索引 + 覆盖率表。
- T2：bench 报告入库；启发式双路径决议见 ADR-0002。
- T3：观察项 γ 记录于 ADR-0002。
- T4：`cargo publish` 成功 + `git tag v1.3.0`（需人工授权）。
- T5：3 fixture 断言（istanbul 无 U+0307 / setext 仅 1 节点 / 空行间隔无 W-META-PLACEMENT）全绿；注释英文化扫描零残留。

---

## 2. 下一阶段预览：P3 — 协议演进支撑

- 跨文档引用语法预研（v2.0 D 系列底座进入试验分支）；
- 与观察项相关的协议修订建议回馈给规范（ADR 通道）。

---

## 3. 阶段总览

| 阶段 | 名称 | 状态 | 产出 |
|---|---|---|---|
| P0 | 参考解析器骨架与 Golden Fixture | ✅ | 解析器 + 22 组 Fixture（14 码全覆盖） |
| P1 | 消费端与投影 | ✅ | 投影标签 + L1/L2 API + CLI 五模式 + 审查面 |
| P2 | 生态对齐 | 🚧 当前 | 一致性语料 + 性能基准 + γ 登记 + 审查决议（T4 待授权） |
