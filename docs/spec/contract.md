# 契约与测试卷（contract）

> **锚定原则**：契约冻结 / 测试即规范之镜（Golden Fixture）
> **关联**：规范 §8（解析器输出契约）、§11（错误码与警告码）

---

## 1. 输出契约（冻结）

DNA 防腐化铁律 2：以下契约为冻结接口，扩展走 Append-Only。

### 1.1 节点表（§8.1）

```json
{ "id": "string|null", "status": "draft|converged|aligned", "valid": true,
  "tags": [], "chars": 0, "body_start": 1, "body_end": 1 }
```

派生字段计算要点（实现与测试共同钉死）：

- 节点区间 = 标题行起，至下一标题行前一行或文件末行；
- 排除行 = 标题行后首个非空行，且为单行完整形态 mddag 注释；
- 正文区间 = 剔除标题行、排除行、首尾空行（内部空行保留）；
- `chars` = 正文各行去行尾换行序列后按 U+000A 连接的 Unicode 码点数；行内尾随空格计入。

### 1.2 规范化边集合（§8.2）

derive 转置折叠后输出；`effective=false` 时 `failure` 标注主要失效原因。

### 1.3 诊断列表（§8.3）

`code / level / node_id / edge / message`。错误码与警告码全集见规范 §11，实现以常量枚举定义，禁止字符串散落。

### 1.4 全局图（§8.4）

生效边集合；节点数组含全部有效节点 id。

## 2. Golden Fixture 测试策略（建议 E）

| Fixture 组 | 覆盖 | 规范来源 |
|---|---|---|
| `10_example` | §10 完整示例，数值逐项断言（chars 12/74/23/11/3；区间 5–5、9–15、19–19、23–23、27–27；全局图为空） | §10 |
| `docmeta_*` | version 协商四分支（缺失/匹配/不匹配/JSON 坏 + 重复键） | §3.1 |
| `meta_syntax_*` | 形态层失败（跨行/未闭合/近似变体/体内 `-->`/缩进） | §5.1 |
| `meta_field_*` | 字段层降级（坏 status/坏边/坏 id/坏 tags） | §5.3 |
| `dup_keys` | 节点级与文档级重复键 | §5.2 / §3.1 |
| `dup_id` | E-DUP-ID 全节点无效 + 引用报 E-REF-NOT-FOUND + 不级联 | §4.2 |
| `slug` | 中文标题派生、64 截断、空 slug → E-MISSING-ID、非法 id 字符 → E-MISSING-ID 路径 | §4.2 |
| `edges_*` | derive 转置折叠、W-REDUNDANT-EDGE、E-REF-NOT-FOUND | §7.2 |
| `cycle` | 全局环 E-CYCLE（循环边全集失效、其余子图正常）、声明层 W-CYCLE-DECLARED | §7.3 |
| `aligned` | W-UPSTREAM-PENDING、延迟绑定（部分 aligned 的图） | §6 |
| `boundary` | 围栏内 `#`、容器内标题、缩进标题不切分 | §4.1 |
| `node0` | 前言不含节点表、W-META-PLACEMENT 扫描 | §3.2 |
| `derived` | 首尾空行剔除、内部空行保留、chars=0、CRLF、尾随空格计入 | §8.1 |

## 3. 验收标准

- `cargo test` 全绿；`cargo clippy` 0 warning；
- §10 数值断言通过（规范内置黄金基准）；
- 每个 fixture 的期望 JSON 与实现输出 diff 为空；
- 同一 fixture 重复解析 2 次，输出逐字节一致（确定性断言）。
