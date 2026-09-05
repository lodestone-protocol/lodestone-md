//! Runtime append operations (v2.0-draft §5.1): five operations that mutate
//! the document while keeping it parse-legal. Each operation returns the new
//! text plus an audit record line. Placement decisions (which lodestone an
//! utterance belongs to) are AI strategy — the protocol only constrains the
//! mutation to legal operations (auditable, never deterministic-free).
//!
//! Audit records: `- mddag-audit: <op> <slug>` — deterministic (no clock in
//! the pure function). The caller (e.g. Helix) attaches time/source at its
//! own audit layer. Audit lines do not participate in L0/L1/L2 projections.

use crate::diag::{Diag, E_ABSORB_ALIGNED, E_STATUS_TRANSITION};
use crate::doc::Status;
use crate::scan::scan;

/// Outcome of an append operation.
#[derive(Debug, Clone)]
pub struct OpOutcome {
    pub text: String,
    pub audit: String,
    pub diagnostics: Vec<Diag>,
}


/// add-node — create a new lodestone before the sediment zone (or at EOF).
/// `body` is the initial content (may be empty).
pub fn add_node(text: &str, slug: &str, title: &str, body: &str) -> OpOutcome {
    let audit = format!("- mddag-audit: add-node {slug}");
    let block = format!("\n# {title}\n- status: draft\n{body}\n");
    let new_text = insert_before_sediment(text, &block);
    OpOutcome { text: new_text, audit, diagnostics: Vec::new() }
}

/// absorb — append iron-filings text into an existing lodestone's body.
/// Rejects aligned lodestones (they only accept content via sediment).
pub fn absorb(text: &str, slug: &str, fragment: &str) -> OpOutcome {
    let doc = scan(text);
    let Some(l) = doc.lodestones.iter().find(|l| l.slug == slug) else {
        return OpOutcome { text: text.to_string(), audit: String::new(),
            diagnostics: vec![Diag::error("E-NO-SUCH-LODESTONE", 0, format!("磁石不存在: {slug}"))] };
    };
    if l.status == Status::Aligned {
        return OpOutcome { text: text.to_string(), audit: String::new(),
            diagnostics: vec![Diag::error(E_ABSORB_ALIGNED, l.title_line, format!("aligned 磁石 {slug} 不可 absorb——应走沉淀区"))] };
    }
    let audit = format!("- mddag-audit: absorb {slug}");
    // insert before the next lodestone title or EOF; after the last body line.
    let lines: Vec<&str> = text.lines().collect();
    let insert_at = if l.end_line <= l.start_line { l.end_line } else { l.end_line };
    let fragment_block = if fragment.trim().is_empty() { String::new() } else { format!("\n{fragment}\n") };
    let mut out = String::new();
    for (i, line) in lines.iter().enumerate() {
        let n = i + 1;
        out.push_str(line);
        out.push('\n');
        if n == insert_at {
            out.push_str(&fragment_block);
        }
    }
    // ensure trailing newline preserved shape
    if !text.ends_with('\n') {
        out.push('\n');
    }
    let _ = insert_at;
    OpOutcome { text: out, audit, diagnostics: Vec::new() }
}

/// advance-status — one-way state machine (draft -> converged -> aligned).
pub fn advance_status(text: &str, slug: &str) -> OpOutcome {
    let doc = scan(text);
    let Some(l) = doc.lodestones.iter().find(|l| l.slug == slug) else {
        return OpOutcome { text: text.to_string(), audit: String::new(),
            diagnostics: vec![Diag::error("E-NO-SUCH-LODESTONE", 0, format!("磁石不存在: {slug}"))] };
    };
    let Some(next) = l.status.advance() else {
        return OpOutcome { text: text.to_string(), audit: String::new(),
            diagnostics: vec![Diag::error(E_STATUS_TRANSITION, l.title_line, format!("{slug} 已是 aligned，不可再推进"))] };
    };
    let audit = format!("- mddag-audit: advance-status {slug}->{}", next.label());
    let out = replace_status_line(text, l, next);
    OpOutcome { text: out, audit, diagnostics: Vec::new() }
}

/// compress — move an aligned lodestone's body into the sediment zone,
/// leaving summary + full-text link in place (v2.0-draft §3.5).
pub fn compress(text: &str, slug: &str, summary: &str) -> OpOutcome {
    let doc = scan(text);
    let Some(l) = doc.lodestones.iter().find(|l| l.slug == slug) else {
        return OpOutcome { text: text.to_string(), audit: String::new(),
            diagnostics: vec![Diag::error("E-NO-SUCH-LODESTONE", 0, format!("磁石不存在: {slug}"))] };
    };
    if l.status != Status::Aligned {
        return OpOutcome { text: text.to_string(), audit: String::new(),
            diagnostics: vec![Diag::error(E_STATUS_TRANSITION, l.title_line, format!("仅 aligned 可 compress，{slug} 当前 {}", l.status.label()))] };
    }
    let audit = format!("- mddag-audit: compress {slug}");
    // body region (excluding title line + status list) moves to sediment.
    let mut body_lines: Vec<String> = Vec::new();
    let mut in_status = true;
    let lines: Vec<&str> = text.lines().collect();
    let mut title_line = String::new();
    for (i, line) in lines.iter().enumerate() {
        let n = i + 1;
        if n < l.start_line || n > l.end_line { continue; }
        if n == l.title_line { title_line = line.to_string(); continue; }
        if in_status && n <= l.end_line {
            let t = line.trim_start();
            if t.starts_with("- ") || t.is_empty() {
                continue;
            }
            in_status = false;
        }
        if !in_status {
            body_lines.push(line.to_string());
        }
    }
    let new_lodestone = format!(
        "{title_line}\n- status: aligned\n- summary: {summary}\n[全文](#{slug}-full)\n"
    );
    let sediment_entry = format!("\n## {slug}-full\n{}", body_lines.join("\n"));
    let sed_start = doc.sediment.as_ref().map(|s| s.start_line).unwrap_or(usize::MAX);
    let out = rebuild_after_compress(text, l.start_line, l.end_line, &new_lodestone, sed_start, &sediment_entry);
    OpOutcome { text: out, audit, diagnostics: Vec::new() }
}

/// append-sediment — add/revise converged body inside the sediment zone.
pub fn append_sediment(text: &str, full_slug: &str, content: &str) -> OpOutcome {
    let doc = scan(text);
    let Some(sed) = &doc.sediment else {
        return OpOutcome { text: text.to_string(), audit: String::new(),
            diagnostics: vec![Diag::error("E-NO-SEDIMENT", 0, "文档无沉淀区——先 add-node + aligned + compress")] };
    };
    let audit = format!("- mddag-audit: append-sediment {full_slug}");
    let entry = format!("## {full_slug}\n{content}\n");
    let mut out = String::new();
    let mut inserted = false;
    let lines: Vec<&str> = text.lines().collect();
    // insert after the last sediment entry's content (i.e., before EOF or
    // before the audit heading if present). Simple: append at end of sediment.
    for (i, line) in lines.iter().enumerate() {
        let n = i + 1;
        out.push_str(line);
        out.push('\n');
        if n == sed.end_line && !inserted {
            out.push_str(&entry);
            inserted = true;
        }
    }
    if !inserted {
        out.push_str(&entry);
    }
    OpOutcome { text: out, audit, diagnostics: Vec::new() }
}

/// strip — cut magnetic lines out of a document ("the edge leaves the
/// global graph"). Pure magnetic-line lines (a line whose trimmed content
/// is entirely `[label](#slug)` links) are removed; body text with inline
/// links is kept. `keep` lists slugs whose magnetic lines survive — the
/// caller passes the in-window slug set for cross-session dangling
/// protection. Audit records the number of edges cut.
pub fn strip(text: &str, keep: &[&str]) -> OpOutcome {
    let doc = scan(text);
    let mut cut = 0usize;
    let mut out = String::new();
    for line in text.lines() {
        let t = line.trim();
        if !t.is_empty() && is_pure_link_line(t) {
            let links = extract_links_inline(t);
            let all_kept = !links.is_empty()
                && links.iter().all(|(_, target)| keep.iter().any(|k| *k == target));
            if links.is_empty() || !all_kept {
                cut += 1;
                continue; // drop the whole magnetic-line line
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    let audit = if cut > 0 {
        format!("- mddag-audit: strip {cut} edges")
    } else {
        format!("- mddag-audit: strip 0 edges")
    };
    let _ = doc;
    OpOutcome { text: out, audit, diagnostics: Vec::new() }
}

/// True when the trimmed line is entirely magnetic-line links.
fn is_pure_link_line(t: &str) -> bool {
    let links = extract_links_inline(t);
    !links.is_empty() && reconstruct(links) == t
}

/// Extract links + reconstruct for the purity check.
fn extract_links_inline(t: &str) -> Vec<(String, String)> {
    crate::scan::extract_links_for_ops(t)
}

fn reconstruct(links: Vec<(String, String)>) -> String {
    links.into_iter().map(|(l, t)| format!("[{l}](#{t})")).collect::<Vec<_>>().join(" ")
}

/// decay — forget a target: remove a whole lodestone, a sediment entry,
/// or a line range inside a lodestone body. The audit line records what was
/// forgotten; the caller decides WHEN (TTL policy with its own clock).
pub fn decay(text: &str, target: &str) -> OpOutcome {
    let doc = scan(text);
    // 1) sediment entry `## <target>-full`
    if let Some(sed) = &doc.sediment {
        if let Some(e) = sed.entries.iter().find(|e| e.slug == target) {
            let audit = format!("- mddag-audit: decay {target}");
            let out = remove_block(text, e.line, find_block_end(text, e.line));
            return OpOutcome { text: out, audit, diagnostics: Vec::new() };
        }
    }
    // 2) whole lodestone
    if let Some(l) = doc.lodestones.iter().find(|l| l.slug == target) {
        let audit = format!("- mddag-audit: decay {target}");
        let out = remove_block(text, l.start_line, l.end_line);
        return OpOutcome { text: out, audit, diagnostics: Vec::new() };
    }
    OpOutcome { text: text.to_string(), audit: String::new(),
        diagnostics: vec![Diag::error("E-NO-SUCH-LODESTONE", 0, format!("磁石或沉淀条目不存在: {target}"))] }
}

/// decay_lines — forget a line range inside a lodestone's body (line numbers
/// come from a previous parse; caller re-validates after the edit).
pub fn decay_lines(text: &str, slug: &str, start: usize, end: usize) -> OpOutcome {
    let doc = scan(text);
    let Some(l) = doc.lodestones.iter().find(|l| l.slug == slug) else {
        return OpOutcome { text: text.to_string(), audit: String::new(),
            diagnostics: vec![Diag::error("E-NO-SUCH-LODESTONE", 0, format!("磁石不存在: {slug}"))] };
    };
    if start < l.start_line || end > l.end_line || start > end {
        return OpOutcome { text: text.to_string(), audit: String::new(),
            diagnostics: vec![Diag::error("E-BAD-RANGE", 0, format!("行范围越界: {start}-{end} (磁石 {}-{})", l.start_line, l.end_line))] };
    }
    let audit = format!("- mddag-audit: decay {slug} lines {start}-{end}");
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        if n >= start && n <= end {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if text.ends_with('\n') && out.ends_with('\n') {
        // keep the trailing-newline invariant
    }
    OpOutcome { text: out, audit, diagnostics: Vec::new() }
}

// ---------- helpers ----------

/// Replace the `- status:` line of a lodestone with the new status value.
fn replace_status_line(text: &str, l: &crate::doc::Lodestone, next: Status) -> String {
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        if n > l.start_line && n <= l.end_line {
            if let Some(rest) = line.trim_start().strip_prefix("- ") {
                if let Some((k, _)) = rest.split_once(':') {
                    if k.trim() == "status" {
                        let indent = &line[..line.len() - line.trim_start().len()];
                        out.push_str(&format!("{indent}- status: {}\n", next.label()));
                        // keep emitting the rest normally
                        continue;
                    }
                }
            }
            // stop at first non-meta line
            if !line.trim_start().starts_with("- ") && !line.trim().is_empty() {
                out.push_str(line);
                out.push('\n');
                // emit the remainder of the lodestone unchanged
                let mut rest2 = String::new();
                for l2 in text.lines().skip(i + 1) {
                    rest2.push_str(l2);
                    rest2.push('\n');
                }
                out.push_str(&rest2);
                return out;
            }
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// End line of a block starting at `start`: the line before the next root
/// heading, or EOF.
fn find_block_end(text: &str, start: usize) -> usize {
    let lines: Vec<&str> = text.lines().collect();
    let mut end = lines.len();
    for (i, line) in lines.iter().enumerate() {
        let n = i + 1;
        if n > start && line.starts_with("# ") {
            end = n - 1;
            break;
        }
    }
    end
}

/// Remove lines [start, end] (1-based, inclusive).
fn remove_block(text: &str, start: usize, end: usize) -> String {
    let mut out = String::new();
    for (i, line) in text.lines().enumerate() {
        let n = i + 1;
        if n >= start && n <= end {
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

/// Insert a block before the sediment zone (or append at EOF).
fn insert_before_sediment(text: &str, block: &str) -> String {
    let doc = scan(text);
    if let Some(sed) = &doc.sediment {
        let mut out = String::new();
        for (i, line) in text.lines().enumerate() {
            let n = i + 1;
            if n == sed.start_line {
                out.push_str(block);
            }
            out.push_str(line);
            out.push('\n');
        }
        out
    } else {
        let mut out = text.to_string();
        if !out.ends_with('\n') {
            out.push('\n');
        }
        out.push_str(block);
        out
    }
}

/// Clean rebuild for compress: lodestone region -> compressed block, sediment
/// entry appended at sediment start. (The naive loop above was abandoned —
/// this index-set rebuild is the source of truth.)
fn rebuild_after_compress(
    text: &str,
    l_start: usize,
    l_end: usize,
    new_lodestone: &str,
    sed_start: usize,
    sediment_entry: &str,
) -> String {
    let lines: Vec<&str> = text.lines().collect();
    let mut out = String::new();
    let mut skip_until = 0usize; // skip lines in (l_start, l_end]
    let mut appended = false;
    for (i, line) in lines.iter().enumerate() {
        let n = i + 1;
        if n == l_start {
            out.push_str(new_lodestone);
            skip_until = l_end;
            continue;
        }
        if n > l_start && n <= skip_until {
            continue;
        }
        if n == sed_start && !appended {
            out.push_str(line);
            out.push('\n');
            out.push_str(sediment_entry);
            appended = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    if !appended && sed_start == usize::MAX {
        // no sediment zone: create one at EOF
        out.push_str("\n# 沉淀区\n");
        out.push_str(sediment_entry);
    }
    out
}


#[cfg(test)]
mod tests {
    use super::*;

    const BASE: &str = "# 甲\n- status: draft\n正文甲。\n\n# 沉淀区\n";

    #[test]
    fn add_node_before_sediment() {
        let r = add_node(BASE, "乙", "乙", "正文乙。\n");
        assert!(r.text.contains("# 乙\n- status: draft\n正文乙。"));
        assert!(r.text.contains("# 沉淀区"));
        // sediment stays last
        let lines: Vec<&str> = r.text.lines().collect();
        let last = lines.iter().rev().find(|l| !l.trim().is_empty()).unwrap();
        assert_eq!(*last, "# 沉淀区");
    }

    #[test]
    fn absorb_appends_to_body() {
        let r = absorb(BASE, "甲", "新铁粉。\n");
        assert!(r.text.contains("新铁粉。"));
        assert!(r.text.contains("正文甲。"));
    }

    #[test]
    fn absorb_rejects_aligned() {
        let text = "# 甲\n- status: aligned\n- summary: 结论。\n[全文](#甲-full)\n";
        let r = absorb(text, "甲", "铁粉");
        assert!(r.diagnostics.iter().any(|d| d.code == E_ABSORB_ALIGNED));
        assert_eq!(r.text, text);
    }

    #[test]
    fn advance_status_one_way() {
        let r = advance_status(BASE, "甲");
        assert!(r.text.contains("- status: converged"));
        let r2 = advance_status(&r.text, "甲");
        assert!(r2.text.contains("- status: aligned"));
        let r3 = advance_status(&r2.text, "甲");
        assert!(r3.diagnostics.iter().any(|d| d.code == E_STATUS_TRANSITION));
    }

    #[test]
    fn compress_moves_body_to_sediment() {
        // build a full aligned lodestone with a real sediment zone
        let text = "# 甲\n- status: aligned\n正文要沉淀。\n\n# 沉淀区\n";
        let r = compress(text, "甲", "一句话摘要。");
        let out = &r.text;
        assert!(out.contains("- summary: 一句话摘要。"));
        assert!(out.contains("[全文](#甲-full)"));
        assert!(out.contains("## 甲-full"));
        assert!(out.contains("正文要沉淀。"));
        // the original body must not remain in the activity zone twice
        let activity = out.split("# 沉淀区").next().unwrap_or("");
        assert!(!activity.contains("正文要沉淀。"));
    }

    #[test]
    fn append_sediment_requires_zone() {
        let r = append_sediment("# 甲\n- status: draft\n", "甲-full", "内容");
        assert!(r.diagnostics.iter().any(|d| d.code == "E-NO-SEDIMENT"));
    }
}

#[cfg(test)]
mod decay_tests {
    use super::*;

    const DOC: &str = "# 甲\n- status: converged\n铁粉甲。\n\n# 乙\n- status: draft\n铁粉乙。\n\n# 沉淀区\n## 甲-full\n（归档正文）\n";

    #[test]
    fn decay_removes_lodestone() {
        let r = decay(DOC, "乙");
        assert!(!r.text.contains("# 乙"));
        assert!(r.text.contains("# 甲"));
        assert!(r.audit.contains("decay 乙"));
        // reparse legal
        let d = scan(&r.text);
        assert!(d.diagnostics.is_empty());
        assert_eq!(d.lodestones.len(), 1);
    }

    #[test]
    fn decay_removes_sediment_entry() {
        let r = decay(DOC, "甲-full");
        assert!(!r.text.contains("## 甲-full"));
        assert!(!r.text.contains("归档正文"));
        assert!(r.text.contains("# 甲"));
    }

    #[test]
    fn decay_lines_removes_range() {
        let r = decay_lines(DOC, "甲", 3, 3);
        assert!(!r.text.contains("铁粉甲。"));
        assert!(r.text.contains("# 甲"));
        assert!(r.audit.contains("lines 3-3"));
    }

    #[test]
    fn decay_unknown_target() {
        let r = decay(DOC, "不存在的");
        assert!(r.diagnostics.iter().any(|d| d.code == "E-NO-SUCH-LODESTONE"));
    }

    #[test]
    fn decay_lines_out_of_range() {
        let r = decay_lines(DOC, "甲", 1, 99);
        assert!(r.diagnostics.iter().any(|d| d.code == "E-BAD-RANGE"));
    }
}
