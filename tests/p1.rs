//! P1 消费端测试：L2 定点正文与三级加载语义。

use std::path::PathBuf;

#[test]
fn l2_body_text_matches_derived_fields() {
    let input = std::fs::read_to_string(fixture("10_example.md")).unwrap();
    // §10：concl-01 正文 = 「基于 [实验记录](#流体实验记录) 的推导。」共 23 码点
    let body = mddag::body_text(&input, "concl-01").unwrap();
    assert_eq!(body, "基于 [实验记录](#流体实验记录) 的推导。");
    assert_eq!(body.chars().count(), 23);

    // 多行节点（围栏内）：L2 返回围栏完整内容
    let cfg = mddag::body_text(&input, "exp-01-config").unwrap();
    assert!(cfg.starts_with("采集环境如下："));
    assert!(cfg.contains("# 通道配置（位于围栏内，不构成节点边界）"));
    assert!(cfg.ends_with("```"));
    assert_eq!(cfg.chars().count(), 74);

    // 空正文节点：chars=0 是合法状态
    let input2 = std::fs::read_to_string(fixture("body_blank_edges.md")).unwrap();
    assert_eq!(mddag::body_text(&input2, "empty-01").unwrap(), "");

    // id 不存在
    assert_eq!(
        mddag::body_text(&input, "no-such-node"),
        Err(mddag::BodyError::NodeNotFound)
    );
}

#[test]
fn l1_skeleton_is_full_contract() {
    // L1 骨架与全量输出同源于单一解析管线；节点表含全部派生字段（契约一致性）
    let input = std::fs::read_to_string(fixture("upstream_pending.md")).unwrap();
    let r = mddag::parse(&input);
    let full = serde_json::to_value(&r).unwrap();
    assert!(full["nodes"].is_array());
    assert!(
        full["edges"][0]["failure"].as_str() == Some("W-UPSTREAM-PENDING")
    );
}

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}
