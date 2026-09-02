//! Lodestone Protocol (MD-DAG) v1.3 Final reference parser.
//!
//! Single pipeline (DNA-specific rule 4): every output comes from one pass of
//! [`parse`] (spec §8). Parse determinism: the output is a pure function of
//! the document's current bytes; no global mutable state.

pub mod body;
pub mod diag;
pub mod docmeta;
pub mod edges;
pub mod ids;
pub mod jsonutil;
pub mod lines;
pub mod nodemeta;
pub mod output;
pub mod projection;
pub mod scanner;

use unicode_normalization::UnicodeNormalization;

use diag::Diagnostic;
use output::{Graph, NodeEntry, ParseResult};
use scanner::Heading;

use nodemeta::{MetaOutcome, STATUS_DRAFT};

/// Parses one Lodestone (MD-DAG) document.
pub fn parse(input: &str) -> ParseResult {
    // Strip the BOM (§3.1: document metadata sits after the BOM).
    let text = input.strip_prefix('\u{FEFF}').unwrap_or(input);
    let lines = lines::split_lines(text);
    let scan = scanner::scan(text);
    let in_code = |line: usize| scan.code_ranges.iter().any(|&(a, b)| line >= a && line <= b);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // §8 step 2: document-level metadata (§3.1 pinned behaviors).
    let doc_meta = docmeta::extract(lines.first().map(|s| s.as_str()), &mut diagnostics);

    // Boundaries: top-level ATX level-1 headings with zero indent (§4.1
    // narrowing: 1–3-space indented headings are body text).
    let boundaries: Vec<&Heading> = scan
        .headings
        .iter()
        .filter(|h| h.col == 1)
        .collect();

    // §3.2 node-0 scan: MUST NOT carry node metadata, MUST NOT be referenced;
    // scanned only to detect W-META-PLACEMENT.
    let node0_end = boundaries
        .first()
        .map(|h| h.line - 1)
        .unwrap_or(lines.len());
    for idx in 1..=node0_end {
        if in_code(idx) {
            continue;
        }
        // A first line shaped like document-level metadata belongs to §3.1;
        // do not double-warn it here.
        if idx == 1 && lines[0].starts_with(docmeta::PREFIX) {
            continue;
        }
        let t = lines[idx - 1].trim_start();
        if t.starts_with("<!--") && t.contains("mddag") {
            diagnostics.push(Diagnostic::warning(
                diag::W_META_PLACEMENT,
                None,
                None,
                format!(
                    "mddag-like comment at line {} is in the untitled preamble; ignored",
                    idx
                ),
            ));
        }
    }

    struct NodeWork {
        title: String,
        title_line: usize,
        end_line: usize,
        id: Option<String>,
        status: String,
        valid: bool,
        tags: Vec<String>,
        declared_edges: Vec<(String, String)>,
    }

    let mut works: Vec<NodeWork> = Vec::new();

    // §8 steps 3–5: node metadata three-layer validation, E-DUP-ID
    // prerequisite, derived fields.
    for (i, h) in boundaries.iter().enumerate() {
        let title_line = h.line;
        let end_line = if i + 1 < boundaries.len() {
            boundaries[i + 1].line - 1
        } else {
            lines.len()
        };

        let mut work = NodeWork {
            title: h.text.clone(),
            title_line,
            end_line,
            id: None,
            status: STATUS_DRAFT.to_string(),
            valid: true,
            tags: Vec::new(),
            declared_edges: Vec::new(),
        };

        let outcome = nodemeta::extract(&lines, title_line, end_line, &in_code, &mut diagnostics);
        match outcome {
            MetaOutcome::Invalid => {
                work.valid = false;
            }
            MetaOutcome::Absent => {
                let s = ids::slug(&h.text);
                if s.is_empty() {
                    diagnostics.push(Diagnostic::error(
                        diag::E_MISSING_ID,
                        None,
                        None,
                        format!(
                            "node at line {} has no usable id (empty slug from title {:?})",
                            title_line, h.text
                        ),
                    ));
                    work.valid = false;
                } else {
                    work.id = Some(s);
                }
            }
            MetaOutcome::Parsed(fields) => {
                match fields.id {
                    Some(Ok(s)) => work.id = Some(s),
                    Some(Err(())) => {
                        diagnostics.push(Diagnostic::error(
                            diag::E_MISSING_ID,
                            None,
                            None,
                            format!(
                                "node at line {} declared an id violating the charset/length rules",
                                title_line
                            ),
                        ));
                        work.valid = false;
                    }
                    None => {
                        let s = ids::slug(&h.text);
                        if s.is_empty() {
                            diagnostics.push(Diagnostic::error(
                                diag::E_MISSING_ID,
                                None,
                                None,
                                format!(
                                    "node at line {} has no usable id (empty slug from title {:?})",
                                    title_line, h.text
                                ),
                            ));
                            work.valid = false;
                        } else {
                            work.id = Some(s);
                        }
                    }
                }
                if work.valid {
                    work.status = fields.status;
                    work.tags = fields.tags;
                    work.declared_edges = fields.declared_edges;
                }
            }
        }

        // §4.2: title not in NFC form → W-NFC-VIOLATION (SHOULD).
        let nfc: String = h.text.chars().nfc().collect();
        if nfc != h.text {
            diagnostics.push(Diagnostic::warning(
                diag::W_NFC_VIOLATION,
                work.id.clone(),
                None,
                format!("title at line {} is not in NFC form", title_line),
            ));
        }

        works.push(work);
    }

    // §8 step 4: E-DUP-ID — every valid node sharing an id becomes invalid;
    // invalid nodes do not participate in detection, so no cascade.
    {
        let mut by_id: std::collections::BTreeMap<String, Vec<usize>> =
            std::collections::BTreeMap::new();
        for (i, w) in works.iter().enumerate() {
            if w.valid {
                if let Some(id) = &w.id {
                    by_id.entry(id.clone()).or_default().push(i);
                }
            }
        }
        for (id, idxs) in by_id {
            if idxs.len() > 1 {
                for &i in &idxs {
                    works[i].valid = false;
                    diagnostics.push(Diagnostic::error(
                        diag::E_DUP_ID,
                        Some(id.clone()),
                        None,
                        format!("duplicate node id {:?}; all nodes with this id are invalid", id),
                    ));
                }
            }
        }
    }

    // §8 step 5: derived fields (uniform for valid and invalid nodes).
    let nodes: Vec<NodeEntry> = works
        .iter()
        .map(|w| {
            let (chars, body_start, body_end) =
                body::compute(&lines, w.title_line, w.end_line, &in_code);
            NodeEntry {
                id: w.id.clone(),
                status: w.status.clone(),
                valid: w.valid,
                tags: w.tags.clone(),
                chars,
                body_start,
                body_end,
                title: w.title.clone(),
            }
        })
        .collect();

    // §8 steps 6–9: edge normalization, reference validation, soft checks,
    // effectiveness.
    let valid_pairs: Vec<(String, String)> = works
        .iter()
        .filter(|w| w.valid)
        .map(|w| (w.id.clone().unwrap_or_default(), w.status.clone()))
        .collect();
    let declared: Vec<edges::DeclaredEdge> = works
        .iter()
        .filter(|w| w.valid)
        .flat_map(|w| {
            w.declared_edges
                .iter()
                .map(|(to, relation)| edges::DeclaredEdge {
                    from_id: w.id.clone().unwrap_or_default(),
                    to: to.clone(),
                    relation: relation.clone(),
                })
        })
        .collect();
    let (edge_entries, graph_edge_list) =
        edges::process(&valid_pairs, &declared, &mut diagnostics);

    let graph = Graph {
        nodes: valid_pairs.into_iter().map(|(id, _)| id).collect(),
        edges: graph_edge_list,
    };

    // §8 step 10: output.
    ParseResult {
        doc_meta,
        nodes,
        edges: edge_entries,
        diagnostics,
        graph,
    }
}

/// Status constants re-exported for consumers.
pub mod states {
    pub use crate::nodemeta::{STATUS_ALIGNED, STATUS_CONVERGED, STATUS_DRAFT};
}

/// L2 targeted body text (§9 three-level loading): returns the body of the
/// node identified by `id`.
///
/// The text matches the derived fields: body lines joined by U+000A (no
/// trailing newline). An empty body is a legal state and yields Some("");
/// `BodyError::NodeNotFound` is returned when no node carries `id`.
pub fn body_text(input: &str, id: &str) -> Result<String, BodyError> {
    let result = parse(input);
    let node = result
        .nodes
        .iter()
        .find(|n| n.id.as_deref() == Some(id))
        .ok_or(BodyError::NodeNotFound)?;
    match (node.body_start, node.body_end) {
        (Some(lo), Some(hi)) => {
            let text = input.strip_prefix('\u{FEFF}').unwrap_or(input);
            let lines = lines::split_lines(text);
            Ok(lines[lo - 1..hi].join("\n"))
        }
        _ => Ok(String::new()),
    }
}

#[derive(Debug, PartialEq)]
pub enum BodyError {
    NodeNotFound,
}
