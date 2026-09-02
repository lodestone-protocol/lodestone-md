# ADR-0001 Rust 参考实现与解析底座选型

> **状态**：Active（2026-09-02）
> **关联**：Lodestone Protocol (MD-DAG) v1.3 Final；P0 阶段
> **方法论**：phyt-DNA v1.0（决策先于代码）

---

## 背景

lodestone-md 是 Lodestone Protocol（MD-DAG）v1.3 Final 规范的权威参考实现仓库。P0 阶段需要交付：

1. 一个符合 §1.4 一致性类中「解析器 + 验证器」义务的参考解析器；
2. Golden Fixture 测试套件（规范末尾"遵循建议 E"），含 §10 完整示例的数值级验证。

规范对解析器的硬性约束：

- 节点边界识别 **MUST 基于 CommonMark 0.31.2 块级解析**（§4.1），MUST NOT 仅按行前缀匹配标题（无法处理围栏内内容）；
- slug 派生 MUST 使用 Unicode 15.1 Simple Lowercase Mapping（§4.2）；
- 解析输出是文档当前字节的纯函数（确定性优先）。

## 决策

| 决策点 | 决议 | 放弃的方案 |
|---|---|---|
| 实现语言 | **Rust**（用户偏好 + 确定性/零成本契合协议气质） | Python / TypeScript |
| CommonMark 底座 | **comrak 0.54**（钉死 CommonMark 0.31.2，提供 AST 与 sourcepos 物理行号） | pulldown-cmark / 自研行扫描器 |
| slug 小写映射 | 在 `char::to_lowercase()`（全小写映射）基础上取首码位近似 Simple Lowercase Mapping，差异个案在测试中记录 | 手写全量 Unicode 表 |
| NFC 检测 | unicode-normalization 0.1（支撑 W-NFC-VIOLATION 的 SHOULD 义务） | 不实现 |
| JSON 读写 | serde + serde_json（重复键检测自研包装，std JSON 解析器默认丢弃重复键，无法报 E-META-SYNTAX） | serde_json 裸用 |
| 仓库布局 | 单 crate `mddag`（lib + bin），P0 不拆 workspace | 多 crate workspace |
| 测试策略 | Golden Fixture：`tests/fixtures/*.md` + 期望 JSON 快照，§10 示例数值逐项断言 | 仅单元测试 |

## 理由

- **comrak vs 自研扫描器**：规范将边界识别钉死在 CommonMark 0.31.2 块级解析上。自研扫描器需复刻围栏、缩进代码块、HTML 块、容器块的完整交互，漂移风险高；comrak 是该标准的成熟 Rust 实现，AST 顶层子节点天然给出"顶层块序列"，`setext` 标志可区分 ATX 与 Setext 标题（协议只认 ATX 一级标题）。
- **重复键检测自研**：§5.2 / §3.1 钉死 JSON 重复键必须报错（E-META-SYNTAX / W-DOC-META），而 serde_json 遵循多数解析器惯例静默保留末键。故元数据注释体先经自研的重复键探测预扫描，再交 serde_json 解析。
- **单 crate**：P0 交付面是一个解析器，拆分只会增加防腐化成本；v2 若出现 CLI 生态再评估 workspace。

## 后果

- Cargo.lock 入库，保证跨机器字节级一致的依赖图（确定性边界的工程延伸）。
- comrak 升级属于接口变更，须走新 ADR。
- Simple Lowercase Mapping 的近似实现若在测试中暴露个案偏差，以新增测试个案 + 修正映射函数的方式收敛，不回退到全表方案。

---

## 附注：comrak 0.54 ↔ CommonMark 0.31.2 核对结论（P3 T2 观察项，2026-09-02）

comrak 0.54.0 官方 README 声明 "Compliant with **CommonMark 0.31.2** by
default"，并附 652/652 一致性徽章（对 commonmark-spec
@9103e341…/spec.txt）。与协议 §4.1 钉死的 CommonMark 0.31.2 基准**精确一致，
无偏差**。GFM 扩展默认关闭（`Options::default()` 零扩展面），与协议"零渲染
污染/零平台依赖"立场一致。决议：维持 comrak 0.54 锁定，无需升级或替换。
若未来协议钉死新版 CommonMark，comrak 升级走新 ADR。
