//! Large-document parse benchmark (P2 T2): builds a synthetic chain document
//! of N nodes and times a single parse pass plus skeleton scale.
//!
//! Run with `cargo run --release --example bench -- [nodes]`, default 5000
//! nodes. Coarse health check only, not a micro-benchmark (determinism first;
//! no statistics library needed).
//!
//! Baseline (post spec-submodule migration, lodestone-md@4d3723a, release,
//! 5000 nodes / ~870 KB): ~0.74–0.76 s single pass, single thread.

use std::time::Instant;

fn main() {
    let n: usize = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(5000);

    // Synthetic acyclic depend chain, alternating aligned/converged, each node
    // with tags plus ~30 CJK chars of body (~(heading + comment + body) x N).
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

    // Warm up, then time (two runs, report the second — a typical duration of
    // the deterministic path).
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
    // Even-numbered aligned nodes point at odd-numbered converged upstreams,
    // so the alignment-debt warning count is ~n/2
    println!(
        "W-UPSTREAM-PENDING: {}",
        result
            .diagnostics
            .iter()
            .filter(|d| d.code == "W-UPSTREAM-PENDING")
            .count()
    );
}
