# lodestone-md

> **Implements Lodestone Protocol v1.3.0**
>
> **Lodestone Protocol — Markdown-Embedded DAG for conversational knowledge
> convergence / Alias: MD-DAG**

[Lodestone Protocol (MD-DAG) v1.3.0](https://github.com/lodestone-protocol/lodestone-spec) 的 Rust 参考实现。协议为人机对话的知识脱水与收敛而生长：节点如磁石吸附结论，边如磁力线登记依赖与分歧，draft → converged → aligned 标记观点成型阶段。

[![CI](https://github.com/lodestone-protocol/lodestone-md/actions/workflows/ci.yml/badge.svg)](https://github.com/lodestone-protocol/lodestone-md/actions/workflows/ci.yml)

> 中文镜像说明：README.md（英文）面向国际读者；本文为中文镜像。仓库治理文档（docs/，中文）与代码注释（英文）规则见 DNA 铁律 #6。

## 仓库拓扑

- **lodestone-spec**（规范权威）：协议规范、错误码注册表、25 组 Golden Fixture 权威源。本仓库以 submodule（`vendor/lodestone-spec`）导入语料，永不改动。
- **lodestone-md**（本仓库）：Rust 解析器 + 一致性测试套件。

## 快速开始

```bash
# 拉取语料（spec submodule）一次：
git submodule update --init --recursive

cargo build --release
target/release/mddag vendor/lodestone-spec/fixtures/10_example.md           # §8 契约 JSON
target/release/mddag --body concl-01 vendor/lodestone-spec/fixtures/10_example.md  # L2 定点正文
target/release/mddag --projection vendor/lodestone-spec/fixtures/10_example.md     # 附录 A 投影标签
target/release/mddag --review vendor/lodestone-spec/fixtures/10_example.md         # 分歧域摘要
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
cargo test                            # 单元 + Golden Fixture + 确定性断言
UPDATE_GOLDEN=1 cargo test            # 重新生成快照（写入 spec submodule = 规范仓变更！须人工核对）
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
| 错误/警告码覆盖 | ✅ §11 全部 14 码（25 组 Fixture 一一对应） | spec 仓 `fixtures/MANIFEST.md` |
| L1/L2 加载语义 | ✅ `body_text()` 与派生字段逐字符一致（含围栏/空正文） | `tests/p1.rs` |

## 项目治理

本仓库以 [phyt-DNA](https://github.com/Jasonmilk/phyt-DNA) 方法论自生长：
仓库治理文档用中文，代码注释用英文。

- `docs/VISION.md` — 愿景与原子原则、哲学七律（逻辑闭环 / 极致复用 / 极致解耦 / 按需获取 / 按需加载 / 物理实时优先 / 确定性优先）
- `docs/DNA.md` — 不可变原则与项目特有铁律
- `docs/RNA.md` — 三层加载协议与 AI 协作铁律
- `docs/PLAN.md` — 当前生长阶段导航牌（新会话必读）
- `docs/decisions/` — 实现层 ADR（ADR-0001 / ADR-0002）

## License

MIT
