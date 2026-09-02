//! Node metadata extraction and three-layer validation (spec §5).
//!
//! Layer order: form (position/prefix/single-line/SUFFIX) → JSON (incl.
//! duplicate keys) → fields (degradation).

use crate::diag::{self, Diagnostic};
use crate::ids;
use crate::jsonutil;
use crate::lines::is_blank;

pub const PREFIX: &str = "<!-- mddag: ";
pub const SUFFIX: &str = "-->";

pub const STATUS_DRAFT: &str = "draft";
pub const STATUS_CONVERGED: &str = "converged";
pub const STATUS_ALIGNED: &str = "aligned";

pub const RELATIONS: [&str; 4] = ["depend", "derive", "support", "refute"];

/// Field set after the field layer validates (with degradation applied).
pub struct Fields {
    /// None = undeclared; Some(Ok) = legal; Some(Err) = declared id violates
    /// the charset/length rules (§5.3: treated as missing)
    pub id: Option<Result<String, ()>>,
    pub status: String,
    /// Declared out-edges (to, relation) that passed the form check
    pub declared_edges: Vec<(String, String)>,
    pub tags: Vec<String>,
}

pub enum MetaOutcome {
    /// No metadata (including near-prefix variants / misplaced positions)
    Absent,
    /// All three layers passed (may carry field-level degradations)
    Parsed(Fields),
    /// E-META-SYNTAX: the node is invalid
    Invalid,
}

/// Extracts metadata from the node range (title_line, end_line] (1-based,
/// inclusive end).
pub fn extract(
    lines: &[String],
    title_line: usize,
    end_line: usize,
    in_code: &dyn Fn(usize) -> bool,
    diag: &mut Vec<Diagnostic>,
) -> MetaOutcome {
    let is_md_comment = |idx: usize| -> bool {
        if in_code(idx) {
            return false;
        }
        let t = lines[idx - 1].trim_start();
        t.starts_with("<!--") && t.contains("mddag")
    };

    let first_non_empty = (title_line + 1..=end_line).find(|&i| !is_blank(&lines[i - 1]));
    let md_lines: Vec<usize> = (title_line + 1..=end_line)
        .filter(|&i| is_md_comment(i))
        .collect();

    if md_lines.is_empty() {
        return MetaOutcome::Absent;
    }

    // Adoption precondition: the first mddag comment sits exactly at the first
    // non-empty line after the heading.
    if Some(md_lines[0]) != first_non_empty {
        // Misplaced (later / indented / variant): the node has no metadata.
        for &i in &md_lines {
            diag.push(Diagnostic::warning(
                diag::W_META_PLACEMENT,
                None,
                None,
                format!(
                    "mddag-like comment at line {} is not the first non-empty line after the heading; ignored",
                    i
                ),
            ));
        }
        return MetaOutcome::Absent;
    }

    let cand = &lines[md_lines[0] - 1];
    if !cand.starts_with(PREFIX) {
        // Near-prefix variant (missing spaces) or indented form:
        // W-META-PLACEMENT, no metadata.
        diag.push(Diagnostic::warning(
            diag::W_META_PLACEMENT,
            None,
            None,
            format!(
                "mddag-like comment at line {} does not match the canonical form; ignored",
                md_lines[0]
            ),
        ));
        for &i in md_lines.iter().skip(1) {
            diag.push(Diagnostic::warning(
                diag::W_META_PLACEMENT,
                None,
                None,
                format!(
                    "mddag-like comment at line {} does not match the canonical form; ignored",
                    i
                ),
            ));
        }
        return MetaOutcome::Absent;
    }
    if !cand.ends_with(SUFFIX) {
        // Top-aligned with the exact prefix but not closed by "-->"
        // (multi-line or unclosed): E-META-SYNTAX.
        diag.push(Diagnostic::error(
            diag::E_META_SYNTAX,
            None,
            None,
            format!(
                "node metadata comment starting at line {} is not closed on a single line",
                md_lines[0]
            ),
        ));
        return MetaOutcome::Invalid;
    }

    // Adopted. The body is the substring between the two delimiters.
    let body = &cand[PREFIX.len()..cand.len() - SUFFIX.len()];
    if body.contains(SUFFIX) {
        diag.push(Diagnostic::error(
            diag::E_META_SYNTAX,
            None,
            None,
            "node metadata body contains \"-->\"",
        ));
        return MetaOutcome::Invalid;
    }

    // Later mddag comments after adoption: ignored, reported as
    // W-REDUNDANT-META.
    for &i in md_lines.iter().skip(1) {
        diag.push(Diagnostic::warning(
            diag::W_REDUNDANT_META,
            None,
            None,
            format!("redundant mddag comment at line {} ignored", i),
        ));
    }

    match parse_fields(body, diag) {
        Some(fields) => MetaOutcome::Parsed(fields),
        None => MetaOutcome::Invalid,
    }
}

/// JSON layer + field layer. None means the JSON layer failed (node invalid).
fn parse_fields(body: &str, diag: &mut Vec<Diagnostic>) -> Option<Fields> {
    if jsonutil::duplicate_keys(body) {
        diag.push(Diagnostic::error(
            diag::E_META_SYNTAX,
            None,
            None,
            "node metadata JSON has duplicate keys",
        ));
        return None;
    }
    let value: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => {
            diag.push(Diagnostic::error(
                diag::E_META_SYNTAX,
                None,
                None,
                "node metadata JSON parse failed",
            ));
            return None;
        }
    };
    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            diag.push(Diagnostic::error(
                diag::E_META_SYNTAX,
                None,
                None,
                "node metadata JSON root is not an object",
            ));
            return None;
        }
    };

    // Parse id first so later field diagnostics can carry the node_id.
    let id: Option<Result<String, ()>> = match obj.get("id") {
        None => None,
        Some(serde_json::Value::String(s)) => {
            if ids::is_valid_declared_id(s) {
                Some(Ok(s.clone()))
            } else {
                Some(Err(()))
            }
        }
        Some(_) => Some(Err(())),
    };
    let node_id: Option<String> = match &id {
        Some(Ok(s)) => Some(s.clone()),
        _ => None,
    };

    let field_diag = |message: String| Diagnostic::error(diag::E_META_FIELD, node_id.clone(), None, message);

    // status: illegal values fall back to draft.
    let status = match obj.get("status") {
        None => STATUS_DRAFT.to_string(),
        Some(serde_json::Value::String(s))
            if matches!(s.as_str(), STATUS_DRAFT | STATUS_CONVERGED | STATUS_ALIGNED) =>
        {
            s.clone()
        }
        Some(_) => {
            diag.push(field_diag("invalid \"status\"; fallback to \"draft\"".to_string()));
            STATUS_DRAFT.to_string()
        }
    };

    // edges: illegal entries are dropped, the rest are kept.
    let mut declared_edges = Vec::new();
    match obj.get("edges") {
        None => {}
        Some(serde_json::Value::Array(arr)) => {
            for item in arr {
                match item.as_object() {
                    Some(e) => {
                        let to = e.get("to").and_then(|v| v.as_str());
                        let rel = e.get("relation").and_then(|v| v.as_str());
                        match (to, rel) {
                            (Some(t), Some(r)) if RELATIONS.contains(&r) => {
                                declared_edges.push((t.to_string(), r.to_string()));
                            }
                            _ => diag.push(field_diag(
                                "edge entry missing or invalid \"to\"/\"relation\"; edge dropped"
                                    .to_string(),
                            )),
                        }
                    }
                    None => diag.push(field_diag(
                        "edge entry is not an object; edge dropped".to_string(),
                    )),
                }
            }
        }
        Some(_) => diag.push(field_diag("\"edges\" is not an array; field ignored".to_string())),
    }

    // tags: ignored unless an array of strings.
    let tags = match obj.get("tags") {
        None => Vec::new(),
        Some(serde_json::Value::Array(a)) => {
            if a.iter().all(|v| v.is_string()) {
                a.iter()
                    .filter_map(|v| v.as_str().map(|s| s.to_string()))
                    .collect()
            } else {
                diag.push(field_diag(
                    "\"tags\" is not an array of strings; field ignored".to_string(),
                ));
                Vec::new()
            }
        }
        Some(_) => {
            diag.push(field_diag(
                "\"tags\" is not an array of strings; field ignored".to_string(),
            ));
            Vec::new()
        }
    };

    // updated: informational field, takes no part in protocol-level
    // computation; its content is ignored.
    // Unknown fields: ignored.

    Some(Fields {
        id,
        status,
        declared_edges,
        tags,
    })
}
