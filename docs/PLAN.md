# Lodestone Protocol (lodestone-md) 开发导航牌（PLAN）

> **版本**：v1.5（拓扑拆分收官，2026-09-02）
> **状态**：🚧 P3 — 协议演进支撑（T1–T2 ⏳）
> **上一阶段**：P0/P1/P2（✅，详见 GROWTH.md）；P2 T4（crates.io）已按决策档案**全面推迟**
> **分支**：main
> **所属方法论**：phyt-DNA v1.0（PLAN 动态流转闭环）
> **规则**：本文件只含当前阶段 + 下一阶段预览 + 阶段总览地图。完成阶段 → GROWTH.md。总行数 ≤150，超出触发历史迁移。

---

## 1. 当前阶段：P3 — 协议演进支撑

> **状态**：⏳ T1 待启动。
> **前置依赖**：拓扑拆分收官（✅，见 ADR-0003）；规范仓 lodestone-spec v1.3.0 已建（tag 已推，Release 待 UI 创建）。

### 1.1 目标

| 任务 | 内容 | 入口 | 状态 |
|---|---|---|---|
| T1 | 一致性维护：以 spec 仓语料为权威，语料变更走 spec 仓 | ADR-0003 | ⏳ |
| T2 | 观察项登记：comrak↔CommonMark 0.31.2 核对（记入 ADR-0001） | 决策档案 §五 | ⏳ |
| T3 | 性能复验（语料迁移后 bench 基线） | examples/bench.rs | ⏳ |
| T4 | （原 crates.io 发布）**推迟**——仅当第三方 Rust 依赖需求出现 | 决策档案 §六 | ⏸ |

### 1.2 代码真相源

- 语料 = `vendor/lodestone-spec/fixtures`（submodule，spec 仓权威）；
- `UPDATE_GOLDEN=1` 写入 submodule = spec 仓变更，仅限规范维护流程（CI guard 拦截 PR 内改动）。

### 1.3 项目特有验收状态

| 验收项 | 状态 | 落地位置 |
|---|---|---|
| cargo test 全绿 | ✅（23 项，经 submodule 语料） | CI（submodules: recursive） |
| cargo clippy `-D warnings` | ✅ 0 | CI |
| 语料不被 PR 改动 | ✅ guard job | .github/workflows/ci.yml |
| 版本声明 | ✅ "Implements Lodestone Protocol v1.3.0" | README ×2 / Cargo.toml |
| 注释英文 / 文档中文 | ✅ | DNA 铁律 #6 |

### 1.4 入口 ADR

- **ADR-0001/0002**（历史）+ **ADR-0003**（语料迁 spec submodule，Active）
- P3 候选：comrak 版本核对结论如需锁定/升级，开 ADR-0004

### 1.5 已确认决策点

| # | 决策点 | 决议 | 状态 |
|---|---|---|---|
| D1–D6 | P0–P2 既有决策 | 保持 | ✅ |
| D7 | crates.io | 全面推迟，等第三方需求 | ✅ |
| D8 | 协议发布形态 | spec 仓 tag + GitHub Release（非 crates.io） | ✅ |
| D9 | 许可 | spec 仓 Apache-2.0；本仓 MIT | ✅ |

### 1.6 P3 进度

| 任务 | 内容 | 状态 | 测试 |
|---|---|---|---|
| T1 | 一致性维护 | ⏳ 待启动 | - |
| T2 | 观察项登记 | ⏳ 待启动 | - |
| T3 | 性能复验 | ⏳ 待启动 | - |
| T4 | crates.io | ⏸ 推迟 | - |

### 1.7 验收标准

- T1：语料变更零漂移（guard 生效、golden 字节比对通过）。
- T2：comrak 0.54 ↔ CommonMark 0.31.2 核对结论记入 ADR-0001（追加）。
- T3：bench 基线复验无回归。

---

## 2. 下一阶段预览：P4 — 生态扩展（有触发条件才启动）

- 第二语言实现（lodestone-py 等）按 ADR 归属规则各自建仓，与本仓共享 spec 语料做互验；
- 跨文档引用语法预研（v2.0 D 系列底座，进试验分支）。

---

## 3. 阶段总览

| 阶段 | 名称 | 状态 | 产出 |
|---|---|---|---|
| P0 | 参考解析器骨架与 Golden Fixture | ✅ | 解析器 + 25 组 Fixture（14 码全覆盖） |
| P1 | 消费端与投影 | ✅ | 投影标签 + L1/L2 API + CLI 五模式 |
| P2 | 生态对齐与拓扑拆分 | ✅ | 一致性语料迁 spec 仓 + 双语 README + guard |
| P3 | 协议演进支撑 | 🚧 当前 | 一致性维护、观察项登记 |
