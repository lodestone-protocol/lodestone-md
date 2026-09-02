//! Golden Fixture 测试（规范"遵循建议 E"）。
//!
//! 每个 `tests/fixtures/<name>.md` 对应 `<name>.json` 期望输出（§8 契约整体快照）。
//! `UPDATE_GOLDEN=1 cargo test` 重新生成期望文件；生成后必须人工核对（尤其 §10 数值）。

use std::env;
use std::fs;
use std::path::PathBuf;

fn fixture_paths() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut cases: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("fixtures directory missing")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().map(|x| x == "md").unwrap_or(false))
        .collect();
    cases.sort();
    cases
}

#[test]
fn golden_fixtures() {
    let cases = fixture_paths();
    assert!(!cases.is_empty(), "no fixtures found");
    let update = env::var("UPDATE_GOLDEN").is_ok();
    for case in &cases {
        let input = fs::read_to_string(case).expect("read fixture");
        let result = mddag::parse(&input);
        let actual = serde_json::to_string_pretty(&result).unwrap() + "\n";
        let expected_path = case.with_extension("json");
        if update {
            fs::write(&expected_path, &actual).expect("write golden");
        } else {
            let expected = fs::read_to_string(&expected_path)
                .unwrap_or_else(|_| panic!("missing expected JSON for {}", case.display()));
            assert_eq!(
                expected, actual,
                "fixture mismatch: {}",
                case.display()
            );
        }
    }
}

/// 确定性断言（VISION 承诺 1）：同一输入两次解析输出逐字节一致。
#[test]
fn parse_is_deterministic() {
    for case in fixture_paths() {
        let input = fs::read_to_string(&case).expect("read fixture");
        let a = serde_json::to_string(&mddag::parse(&input)).unwrap();
        let b = serde_json::to_string(&mddag::parse(&input)).unwrap();
        assert_eq!(a, b, "nondeterministic parse: {}", case.display());
    }
}

/// 规范 §10 示例是内置黄金基准：数值级断言（不依赖快照文件）。
#[test]
fn spec_10_example_values() {
    let input = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/10_example.md"),
    )
    .expect("read §10 fixture");
    let r = mddag::parse(&input);

    let expect: [(&str, bool, usize, usize, usize); 5] = [
        ("exp-01", true, 12, 5, 5),
        ("exp-01-config", true, 74, 9, 15),
        ("concl-01", true, 23, 19, 19),
        ("counter-01", true, 11, 23, 23),
        ("plan-01", true, 3, 27, 27),
    ];
    assert_eq!(r.nodes.len(), 5, "node count");
    for (i, (id, valid, chars, bs, be)) in expect.iter().enumerate() {
        let n = &r.nodes[i];
        assert_eq!(n.id.as_deref(), Some(*id), "node {} id", i);
        assert_eq!(n.valid, *valid, "node {} valid", i);
        assert_eq!(n.chars, *chars, "node {} chars", id);
        assert_eq!(n.body_start, Some(*bs), "node {} body_start", id);
        assert_eq!(n.body_end, Some(*be), "node {} body_end", id);
    }

    // 全局图为空（concl-01 未 aligned）；规范化边集合含全部 4 条声明边。
    assert!(r.graph.edges.is_empty(), "global graph must be empty");
    assert_eq!(r.edges.len(), 4, "normalized edge count");
    assert!(r
        .edges
        .iter()
        .all(|e| !e.effective), "no edge effective");

    // plan-01 -> old-note 失效于 E-REF-NOT-FOUND。
    assert!(r.diagnostics.iter().any(|d| d.code == "E-REF-NOT-FOUND"
        && d.edge.as_deref() == Some("plan-01 -> old-note")));
}
