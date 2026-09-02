//! Golden Fixture tests (spec "recommendation E").
//!
//! Each `tests/fixtures/<name>.md` has a matching `<name>.json` expected
//! output (a full §8 contract snapshot). `UPDATE_GOLDEN=1 cargo test`
//! regenerates the expected files; generated files must be reviewed by hand
//! (especially the §10 figures).

use std::env;
use std::fs;
use std::path::PathBuf;

fn fixture_paths() -> Vec<PathBuf> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let mut cases: Vec<PathBuf> = fs::read_dir(&dir)
        .expect("fixtures directory missing")
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.extension().map(|x| x == "md").unwrap_or(false)
                && p.file_name().and_then(|n| n.to_str()) != Some("MANIFEST.md")
        })
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

/// Determinism assertion (VISION promise 1): two parses of one input produce
/// byte-identical output.
#[test]
fn parse_is_deterministic() {
    for case in fixture_paths() {
        let input = fs::read_to_string(&case).expect("read fixture");
        let a = serde_json::to_string(&mddag::parse(&input)).unwrap();
        let b = serde_json::to_string(&mddag::parse(&input)).unwrap();
        assert_eq!(a, b, "nondeterministic parse: {}", case.display());
    }
}

fn fixture_path(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

/// Review decision P1: cross-implementation anchor for Simple vs Full Case
/// Mapping. Under Unicode 15.1 Simple Lowercase Mapping, U+0130 lowercases to
/// U+0069, so the id is exactly "istanbul" (no U+0307 combining dot).
#[test]
fn slug_istanbul_simple_lowercase() {
    let r = mddag::parse(&fs::read_to_string(fixture_path("slug_istanbul.md")).unwrap());
    assert_eq!(r.nodes.len(), 1);
    let n = &r.nodes[0];
    assert!(n.valid);
    assert_eq!(n.id.as_deref(), Some("istanbul"));
    assert!(!r.nodes[0].id.as_deref().unwrap_or("").contains('\u{0307}'));
}

/// Review decision setext: a Setext H1 (`===` underline) is not a node
/// boundary (§4.1 recognizes ATX headings only).
#[test]
fn setext_h1_is_not_a_boundary() {
    let r = mddag::parse(&fs::read_to_string(fixture_path("boundary_setext.md")).unwrap());
    assert_eq!(r.nodes.len(), 1, "setext heading must not split a node");
    assert_eq!(r.nodes[0].id.as_deref(), Some("a"));
}

/// Review decision blank gap: with a blank line between the heading and the
/// comment, the comment is still the "first non-empty line" and must be
/// adopted (§5.1).
#[test]
fn metadata_accepted_across_blank_gap() {
    let r = mddag::parse(&fs::read_to_string(fixture_path("meta_blank_gap.md")).unwrap());
    assert_eq!(r.nodes.len(), 1);
    assert_eq!(r.nodes[0].id.as_deref(), Some("c"));
    assert_eq!(r.nodes[0].status, "aligned");
    assert!(
        r.diagnostics.iter().all(|d| d.code != "W-META-PLACEMENT"),
        "blank gap between title and metadata must not warn"
    );
}

/// The spec §10 example is a built-in golden baseline: numeric assertions
/// (independent of snapshot files).
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

    // Global graph empty (concl-01 not aligned); the normalized edge set
    // keeps all 4 declared edges.
    assert!(r.graph.edges.is_empty(), "global graph must be empty");
    assert_eq!(r.edges.len(), 4, "normalized edge count");
    assert!(r
        .edges
        .iter()
        .all(|e| !e.effective), "no edge effective");

    // plan-01 -> old-note fails with E-REF-NOT-FOUND.
    assert!(r.diagnostics.iter().any(|d| d.code == "E-REF-NOT-FOUND"
        && d.edge.as_deref() == Some("plan-01 -> old-note")));
}
