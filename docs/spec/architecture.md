# 核心架构卷（architecture）

> **锚定原则**：确定性优先 / 单一解析管线 / 容错分层 / CommonMark 锚定
> **关联**：ADR-0001（Rust + comrak 选型）

---

## 1. 仓库布局

```
lodestone-md/
├── docs/                  # phyt-DNA 文档生态（N 层 + D 层）
├── src/
│   ├── lib.rs             # 公共 API：parse() -> ParseResult
│   ├── scanner.rs         # 步骤1：comrak 顶层块扫描，节点物理行区间
│   ├── docmeta.rs         # 步骤2：文档级元数据（§3.1 行为表）
│   ├── nodemeta.rs        # 步骤3：三层校验（形态/JSON/字段）
│   ├── ids.rs             # 步骤4：slug 派生 + E-DUP-ID
│   ├── body.rs            # 步骤5：派生字段 chars/body_start/body_end
│   ├── edges.rs           # 步骤6-9：规范化、引用校验、生效判定、环检测
│   ├── jsonutil.rs        # 重复键预扫描（E-META-SYNTAX / W-DOC-META）
│   ├── diag.rs            # 诊断码常量（§11 一一对应）
│   ├── main.rs            # CLI：mddag <file> -> JSON
│   └── output.rs          # §8.2-§8.4 输出契约结构体
└── tests/
    ├── golden.rs          # Fixture 驱动测试
    └── fixtures/          # 每条 MUST/SHOULD 一个 .md + 期望 JSON
```

## 2. 核心数据流

单一管线（DNA 特有铁律 4）：所有输出出自 `parse()` 一趟结果，禁止旁路计算。

```
&str ──> 扫描节点区间 ──> 文档级元数据 ──> 节点元数据三层校验
     ──> E-DUP-ID 判定 ──> 派生字段 ──> 边规范化 ──> 引用校验
     ──> 软校验（环/上游）──> 生效判定 + 全局环检测 ──> ParseResult
```

ParseResult（§8 契约的 Rust 物化）：

- `doc_meta: Option<DocMeta>`（version 协商结果）
- `nodes: Vec<NodeEntry>`（id/status/valid/tags/chars/body_start/body_end/title）
- `edges: Vec<EdgeEntry>`（from/to/relation/effective/failure）
- `diagnostics: Vec<Diagnostic>`（code/level/node_id/edge/message）
- `graph: Graph`（生效边集合）

## 3. 关键实现决议

| 议题 | 决议 | 规范依据 |
|---|---|---|
| 节点边界 | comrak AST 顶层 `Heading{level:1, setext:false}` 子节点 | §4.1（MUST NOT 自定义规则匹配） |
| 文档级元数据提取 | 行级判定（首行物理上不可能是围栏内容，行级安全） | §3.1 |
| 节点元数据提取 | 标题行后首个非空行，顶格 + 精确前缀 `<!-- mddag: ` + `-->` 结尾 | §5.1 |
| 排除行判定 | 同形态单行完整注释；未闭合/跨行/近似变体不构成排除行 | §8.1 |
| slug 小写 | `char::to_lowercase()` 首码位近似 Simple Lowercase Mapping（ADR-0001 记录偏差风险） | §4.2 |
| 重复键 | serde_json 之前流式预扫描 | §5.2 / §3.1 |
| 环检测 | Tarjan SCC；同 SCC（size>1）内边或自环 ∈ 循环边全集 | §7.3 |
| 边 failure 取值 | E-REF-NOT-FOUND / E-CYCLE / W-UPSTREAM-PENDING（源 aligned 目标未 aligned）；其余未生效边（对齐性 pending/nascent）failure 为 null 并在 ADR 记录解释 | §8.2 |
| CRLF / BOM | BOM 剥离后解析；CRLF 计为一行结束；物理行号 1-based | §8.1 |

## 4. 演进边界

- comrak 升级 = 接口变更 = 新 ADR；
- 输出契约（§8.1–§8.4）冻结，扩展 Append-Only；
- v2.0 议程（附录 C）不进代码。
