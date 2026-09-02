//! P1 consumer tests: L2 targeted body text and three-level loading semantics.

use std::path::PathBuf;

#[test]
fn l2_body_text_matches_derived_fields() {
    let input = std::fs::read_to_string(fixture("10_example.md")).unwrap();
    // §10: the concl-01 body line has exactly 23 code points
    let body = mddag::body_text(&input, "concl-01").unwrap();
    assert_eq!(body, "基于 [实验记录](#流体实验记录) 的推导。");
    assert_eq!(body.chars().count(), 23);

    // Multi-line node (fenced): L2 returns the complete fenced content
    let cfg = mddag::body_text(&input, "exp-01-config").unwrap();
    assert!(cfg.starts_with("采集环境如下："));
    assert!(cfg.contains("# 通道配置（位于围栏内，不构成节点边界）"));
    assert!(cfg.ends_with("```"));
    assert_eq!(cfg.chars().count(), 74);

    // Empty-body node: chars = 0 is a legal state
    let input2 = std::fs::read_to_string(fixture("body_blank_edges.md")).unwrap();
    assert_eq!(mddag::body_text(&input2, "empty-01").unwrap(), "");

    // Unknown id
    assert_eq!(
        mddag::body_text(&input, "no-such-node"),
        Err(mddag::BodyError::NodeNotFound)
    );
}

#[test]
fn l1_skeleton_is_full_contract() {
    // L1 skeleton and full output come from one parse pass; the node table
    // carries all derived fields (contract consistency)
    let input = std::fs::read_to_string(fixture("upstream_pending.md")).unwrap();
    let r = mddag::parse(&input);
    let full = serde_json::to_value(&r).unwrap();
    assert!(full["nodes"].is_array());
    assert!(
        full["edges"][0]["failure"].as_str() == Some("W-UPSTREAM-PENDING")
    );
}

/// Fixture corpus lives in the lodestone-spec submodule (authoritative source).
fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("vendor/lodestone-spec/fixtures")
        .join(name)
}
