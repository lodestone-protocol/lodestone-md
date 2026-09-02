//! Document-level metadata (spec §3.1): behaviors are pinned; every failure
//! path converges on "warn + keep parsing".

use crate::diag::{self, Diagnostic};
use crate::jsonutil;
use crate::output::DocMetaOut;

/// Top-aligned, single-line, exact prefix (the two spaces match exactly).
pub const PREFIX: &str = "<!-- mddag: ";
pub const SUFFIX: &str = "-->";
pub const SUPPORTED_VERSION: &str = "1.3";

/// Extracts document-level metadata from the first physical line (BOM stripped).
///
/// Returns the metadata when adopted; otherwise the caller keeps parsing
/// best-effort.
pub fn extract(line1: Option<&str>, diag: &mut Vec<Diagnostic>) -> Option<DocMetaOut> {
    let line = line1?;
    // Extraction rule: adopt only when the first block is a top-aligned
    // single-line comment matching the exact prefix; otherwise it is an
    // ordinary HTML block (legal state, no warning).
    if !line.starts_with(PREFIX) || !line.ends_with(SUFFIX) {
        return None;
    }
    let body = &line[PREFIX.len()..line.len() - SUFFIX.len()];
    // The body MUST NOT contain "-->" (same constraint as node metadata);
    // a violation converges on W-DOC-META + ignore.
    if body.contains(SUFFIX) {
        diag.push(Diagnostic::warning(
            diag::W_DOC_META,
            None,
            None,
            "document-level metadata body contains \"-->\"; ignored",
        ));
        return None;
    }
    // JSON duplicate keys → W-DOC-META, ignore, keep parsing.
    if jsonutil::duplicate_keys(body) {
        diag.push(Diagnostic::warning(
            diag::W_DOC_META,
            None,
            None,
            "document-level metadata JSON has duplicate keys; ignored",
        ));
        return None;
    }
    let value: serde_json::Value = match serde_json::from_str(body) {
        Ok(v) => v,
        Err(_) => {
            diag.push(Diagnostic::warning(
                diag::W_DOC_META,
                None,
                None,
                "document-level metadata JSON parse failed; ignored",
            ));
            return None;
        }
    };
    let obj = match value.as_object() {
        Some(o) => o,
        None => {
            diag.push(Diagnostic::warning(
                diag::W_DOC_META,
                None,
                None,
                "document-level metadata JSON root is not an object; ignored",
            ));
            return None;
        }
    };
    match obj.get("version") {
        // Version not declared: no promise made, no warning, keep parsing.
        None => Some(DocMetaOut { version: None }),
        Some(serde_json::Value::String(s)) => {
            if s != SUPPORTED_VERSION {
                diag.push(Diagnostic::warning(
                    diag::W_VERSION_MISMATCH,
                    None,
                    None,
                    format!(
                        "document version {:?} is not supported ({:?}); parsing best-effort",
                        s, SUPPORTED_VERSION
                    ),
                ));
            }
            Some(DocMetaOut {
                version: Some(s.clone()),
            })
        }
        // Version of the wrong type → W-DOC-META, ignore.
        Some(_) => {
            diag.push(Diagnostic::warning(
                diag::W_DOC_META,
                None,
                None,
                "document-level metadata \"version\" is not a string; ignored",
            ));
            None
        }
    }
}
