# ADR-0003 Fixture corpus moved to the lodestone-spec submodule

> **Status**: Active (2026-09-02)
> **关联**: lodestone-spec 仓建仓（spec 仓 adr/ADR-0001）；决策档案阶段 B/D
> **方法论**: phyt-DNA v1.0（决策先于代码）

---

## 背景

Lodestone Protocol 是语言无关的协议。v1.3.0 起，规范权威与 Golden Fixture
语料迁移到独立仓 `lodestone-protocol/lodestone-spec`（tag `v1.3.0`，导入自
本仓快照 `3156199`，CI 全绿，字节一致）。本实现仓必须转为语料消费方。

## 决策

| 决策点 | 决议 |
|---|---|
| 语料获取 | `tests/fixtures/` 删除，改为 git submodule `vendor/lodestone-spec`（spec 仓 fixtures/ 为权威源）；CI 以 `submodules: recursive` 拉取 |
| 测试路径 | golden/p1/投影测试统一指向 `vendor/lodestone-spec/fixtures`（`include_str!` 同步改指） |
| 期望输出归属 | spec 仓 `.json` 是规范化字节；`UPDATE_GOLDEN=1` 只允许在 spec 仓维护流程中使用 |
| PR guard | CI 新增 guard job：PR 改动 `vendor/lodestone-spec/` 即失败（防 UPDATE_GOLDEN 滥用 / 语料漂移） |
| 版本声明 | 本仓 README 与包元数据声明 "Implements Lodestone Protocol v1.3.0"；主.次版本跟随协议，补丁独立 |
| crates.io | 全面推迟：本仓 Cargo 元数据保留发布字段（无害），但不执行任何发布动作，直至第三方需求出现 |

## 理由

- 协议语料与实现解耦：未来第二语言实现（lodestone-py 等）以同一 spec 仓
  语料做一致性验证，避免每仓复制一份导致的漂移；
- submodule 指针即语料版本：实现仓可声明"对 spec 仓 X 提交的一致性"，
  语料更新有显式提交点；
- guard job 把"期望输出是契约不是代码"（CONTRIBUTING）落到 CI 执行层。

## 后果

- 旧 ADR 中 `tests/fixtures/` 表述为历史事实保留；本文为现行拓扑。
- 解析逻辑零改动：语料路径变更后 23 测试全绿（golden 字节比对兜底）。
- docs/ 活跃文档（PLAN/SPEC）中的语料引用以本 ADR 为准。
