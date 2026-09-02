//! 大文档解析基准（P2 T2）：合成 N 节点链式文档，测量单趟解析耗时与骨架规模。
//!
//! 运行：`cargo run --release --example bench -- [nodes]`
//! 默认 5000 节点。仅为粗粒度体检，非微基准（确定性优先，无需统计库）。

use std::time::Instant;

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);

    // 合成：链式 depend 图（无环），节点交替 aligned/converged，
    // 每节点含 tags + 30 字正文，文件约 (标题 + 注释 + 正文) × N。
    let mut doc = String::from("<!-- mddag: {\"version\":\"1.3\"} -->\n\n");
    for i in 0..n {
        let status = if i % 2 == 0 { "aligned" } else { "converged" };
        let edge_field = if i > 0 {
            format!(
                ",\"edges\":[{{\"to\":\"node-{:04}\",\"relation\":\"depend\"}}]",
                i - 1
            )
        } else {
            String::new()
        };
        doc.push_str(&format!(
            "# 节点 {:04}\n<!-- mddag: {{\"id\":\"node-{:04}\",\"status\":\"{}\",\"tags\":[\"bench\"]{} }} -->\n",
            i, i, status, edge_field
        ));
        doc.push_str("正文内容若干字用于度量。\n\n");
    }

    // 预热 + 计时（两次，报告第二次——取确定性路径的典型耗时）
    let _ = mddag::parse(&doc);
    let t0 = Instant::now();
    let result = mddag::parse(&doc);
    let elapsed = t0.elapsed();

    println!("nodes: {} (bytes: {})", n, doc.len());
    println!(
        "parse: {:.2} ms | effective edges: {} | diagnostics: {}",
        elapsed.as_secs_f64() * 1000.0,
        result.edges.iter().filter(|e| e.effective).count(),
        result.diagnostics.len()
    );
    // 链中偶数位 aligned 节点指向奇数位 converged 上游 → 对齐欠账警告数 ≈ n/2
    println!(
        "W-UPSTREAM-PENDING: {}",
        result
            .diagnostics
            .iter()
            .filter(|d| d.code == "W-UPSTREAM-PENDING")
            .count()
    );
}
