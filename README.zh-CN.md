# lodestone-md

> **Lodestone Protocol (MD-DAG) v1.3 Final 权威参考实现**
> 为人机对话的知识脱水与收敛而生长：节点如磁石吸附结论，边如磁力线登记依赖与分歧，draft → converged → aligned 标记观点成型阶段。

[![CI](https://github.com/lodestone-protocol/lodestone-md/actions/workflows/ci.yml/badge.svg)](https://github.com/lodestone-protocol/lodestone-md/actions/workflows/ci.yml)

> 中文镜像说明：README.md（英文）为 crates.io 主 README；本文为中文镜像。
> 仓库治理文档（docs/，中文）与代码注释（英文）规则见 DNA 铁律 #6。

## 是什么

- **协议**：在标准 Markdown 中以 HTML 注释嵌入 DAG 结构元数据（节点 / 边 / 状态机），渲染透明、机器可确定性解析、零平台依赖。规范全文：Lodestone Protocol (MD-DAG) v1.3 Final。
- **本仓库**：协议的权威 Rust 参考解析器 + Golden Fixture 测试套件。解析输出是文档当前字节的纯函数（确定性优先）；规范 §10 完整示例的数值经测试逐项钉死。

## 快速开始

```bash
cargo build --release
target/release/mddag tests/fixtures/10_example.md          # §8 契约 JSON
target/release/mddag --body concl-01 tests/fixtures/10_example.md   # L2 定点正文
target/release/mddag --projection tests/fixtures/10_example.md      # 附录 A 边投影标签
target/release/mddag --review tests/fixtures/10_example.md          # 分歧域 + 警告概览
```

库调用：

```rust
let result = mddag::parse(&markdown_text);
// L1 骨架：result.nodes / result.edges / result.diagnostics / result.graph
// L2 定点正文：mddag::body_text(&markdown_text, "concl-01")
// 附录 A 投影：mddag::projection::project(&result) / mddag::projection::review(&result)
```

## 三级加载（消费方原生读取模式）

| 级 | 内容 | 用途 |
|---|---|---|
| L1 骨架 | 节点表 + 规范化边集合 + 诊断 | 重建全局认知、规划读取 |
| L2 定点正文 | 按 `body_start` / `body_end` 行号读取 | 回答具体问题、追溯依赖链 |
| L3 全文 | 文档全部字节 | 查全率任务（审查 / 迁移 / 审计） |

## 测试

```bash
cargo test                              # 单元 + Golden Fixture + 确定性断言
UPDATE_GOLDEN=1 cargo test              # 重新生成期望快照（须人工核对）
cargo clippy --all-targets -- -D warnings   # 验收门槛：warnings 视为错误
cargo run --release --example bench -- 5000 # 性能体检（非 CI 门槛）
```

### 验收状态（2026-09-02 实测）

| 门槛 | 实测 | 验证位置 |
|---|---|---|
| `cargo test` | ✅ 23 passed（15 单元 + 6 Golden + 2 P1 集成） | CI / 本地 |
| `cargo clippy --all-targets -- -D warnings` | ✅ 0 warnings | CI / 本地 |
| §10 黄金基准 | ✅ chars 12/74/23/11/3，区间 5–5/9–15/19–19/23–23/27–27，全局图为空 | `tests/golden.rs::spec_10_example_values` |
| 解析确定性 | ✅ 同输入两次解析输出逐字节一致 | `tests/golden.rs::parse_is_deterministic` |
| 错误/警告码覆盖 | ✅ §11 全部 14 码（25 组 Fixture 一一对应） | `tests/fixtures/MANIFEST.md` |
| L1/L2 加载语义 | ✅ `body_text()` 与派生字段逐字符一致（含围栏/空正文） | `tests/p1.rs` |

## 项目治理

本仓库以 [phyt-DNA](https://github.com/Jasonmilk/phyt-DNA) 方法论自生长：
仓库治理文档用中文，代码注释用英文。

- `docs/VISION.md` — 愿景与原子原则、哲学七律（逻辑闭环 / 极致复用 / 极致解耦 / 按需获取 / 按需加载 / 物理实时优先 / 确定性优先）
- `docs/DNA.md` — 不可变原则与项目特有铁律
- `docs/RNA.md` — 三层加载协议与 AI 协作铁律
- `docs/PLAN.md` — 当前生长阶段导航牌（新会话必读）
- `docs/decisions/` — ADR 决策记录（ADR-0001：Rust + comrak；ADR-0002：一致性语料与性能策略）

## License

MIT
