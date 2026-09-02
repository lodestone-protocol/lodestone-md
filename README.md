# lodestone-md

> **Lodestone Protocol (MD-DAG) v1.3 Final 权威参考实现**
> 为人机对话的知识脱水与收敛而生长：节点如磁石吸附结论，边如磁力线登记依赖与分歧，draft → converged → aligned 标记成型阶段。

[![CI](https://github.com/lodestone-protocol/lodestone-md/actions/workflows/ci.yml/badge.svg)](https://github.com/lodestone-protocol/lodestone-md/actions/workflows/ci.yml)

## 是什么

- **协议**：在标准 Markdown 中以 HTML 注释嵌入 DAG 结构元数据（节点 / 边 / 状态机），渲染透明、机器可确定性解析、零平台依赖。规范全文见协议规范文档（v1.3 Final）。
- **本仓库**：协议的参考解析器（Rust）+ Golden Fixture 测试套件。解析输出是文档当前字节的纯函数（确定性优先）；§10 完整示例的数值经测试逐项钉死。

## 快速开始

```bash
cargo build --release
target/release/mddag tests/fixtures/10_example.md   # 输出 §8 契约 JSON
echo '# 我的节点' | target/release/mddag -          # stdin
```

库调用：

```rust
let result = mddag::parse(&markdown_text);
// result.nodes / result.edges / result.diagnostics / result.graph
```

## 三级加载（消费方原生读取模式）

| 级 | 内容 | 用途 |
|---|---|---|
| L1 骨架 | 节点表 + 规范化边集合 + 诊断 | 重建全局认知（本次输出即 L1） |
| L2 定点正文 | 按 `body_start` / `body_end` 行号读取 | 回答具体问题、追溯依赖链 |
| L3 全文 | 文档全部字节 | 查全率任务（审查 / 迁移 / 审计） |

## 测试

```bash
cargo test                        # 22 组 Golden Fixture + 单元测试 + 确定性断言
UPDATE_GOLDEN=1 cargo test        # 重新生成期望快照（须人工核对）
```

Fixture 覆盖规范 §11 全部 14 个错误/警告码与 §10 黄金基准，见 `tests/fixtures/`。

## 项目治理

本仓库以 [phyt-DNA](https://github.com/Jasonmilk/phyt-DNA) 方法论自生长：

- `docs/VISION.md` — 愿景与原子原则、哲学七律（逻辑闭环 / 极致复用 / 极致解耦 / 按需获取 / 按需加载 / 物理实时优先 / 确定性优先）
- `docs/DNA.md` — 不可变原则与项目特有铁律
- `docs/RNA.md` — 三层加载协议与 AI 协作铁律
- `docs/PLAN.md` — 当前生长阶段导航牌（新会话必读）
- `docs/decisions/` — ADR 决策记录（ADR-0001：Rust + comrak 选型）

## License

Apache-2.0
