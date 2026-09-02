# Lodestone Protocol (lodestone-md) 生长记录（GROWTH）

> **所属方法论**：phyt-DNA v1.0
> **规则**：保留最近 3 次健康快照。超过 3 条时，最旧的移入 `docs/archive/growth/`。历史永不删除。

---

### 2026-09-02 P1 消费端与投影收官

- **事件**：三级加载语义从协议文本变为可执行 API 与命令行面。投影模块（附录 A 六标签）、L2 定点正文 `body_text()`、CLI 五模式（全量 / --skeleton / --body / --projection / --review）、人类审查面（分歧域 + 警告概览）落地，§10 黄金基准在投影层复用验证（concl→exp pending、counter→concl nascent、plan→old-note dangling）。
- **关键决策**：
  - 投影标签判定优先级钉死：dangling > cyclic > redundant > coherent > pending > nascent（互斥分类，确定性计算）；
  - L1 骨架 = 单趟解析的节点表 + 规范化边集合 + 诊断（§9 契约即 L1），无独立旁路；
  - 投影与审查面为只读视图，绝不回写解析事实（协议仅规范事实，不规范呈现）。
- **方法论闭环**：PLAN 流转（P1 → P2）+ GROWTH 记录 + ADR-0001 关联
- **健康度**：20 tests passed（15 单元 + 3 golden + 2 P1），clippy 0 warning
- **版本**：mddag 1.3.0（P1 增量待提交）

---

### 2026-09-02 P0 参考解析器骨架与 Golden Fixture 收官

- **事件**：lodestone-md 从空仓库生长为 Lodestone Protocol (MD-DAG) v1.3 Final 的参考实现：完成 phyt-DNA 十件套文档体系、ADR-0001 技术选型（Rust + comrak 0.54）、§8 十步解析管线全量实现（22 组 Golden Fixture，§11 全部 14 个错误/警告码覆盖）。
- **关键决策**：
  - ADR-0001：Rust + comrak（CommonMark 0.31.2 钉死）；重复键检测自研流式预扫描（serde_json 之前）；slug 小写映射以 `char::to_lowercase()` 首码位近似 Simple Lowercase Mapping；
  - E-DUP-ID 后果完整落地：同 id 全部节点无效 + 占位保留 + 引用报 E-REF-NOT-FOUND + 不级联；
  - E-CYCLE 失效集合精确化为循环边全集（Kosaraju 迭代 SCC），其余子图正常输出；
  - 用户哲学七律写入 VISION：逻辑闭环 / 极致复用 / 极致解耦 / 按需获取 / 按需加载 / 物理实时优先 / 确定性优先。
- **方法论闭环**：PLAN 流转（P0 → P1）+ GROWTH 首条记录 + ADR-0001 关联
- **健康度**：14 tests passed（11 单元 + 3 集成），clippy 0 warning；§10 黄金基准数值逐项吻合（chars 12/74/23/11/3，区间 5–5/9–15/19–19/23–23/27–27，全局图为空）
- **版本**：mddag 1.3.0（待提交）
