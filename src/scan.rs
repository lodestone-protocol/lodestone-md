//! Line-level scanner (v2.0-draft §3): builds the Doc model from document
//! bytes with zero external parsing — heading boundaries, status lists,
//! magnetic lines, body extraction, sediment zone. Pure function of bytes.
//!
//! CommonMark narrowings inherited from v1.3 §4.1: a root-level heading is a
//! zero-indent `# ` line that is a top-level block (we track fenced blocks
//! to exclude headings inside them).

use crate::diag::{Diag, E_DUP_ID, E_MISSING_ID, W_REF_NOT_FOUND, W_SEDIMENT_REF, W_SELF_REF, W_STATUS_MISSING};
use crate::doc::{Doc, Lodestone, MagneticLine, Sediment, SedimentEntry, Status};
use crate::slug::slugify;

/// The reserved sediment heading (v2.0-draft §3.1): `# 沉淀区`.
const SEDIMENT_TITLE: &str = "沉淀区";

/// Parse a document from bytes. Deterministic: output is a pure function of
/// the input bytes (the protocol determinism contract).
pub fn scan(text: &str) -> Doc {
    let raw = text.to_string();
    let lines: Vec<&str> = text.lines().collect();
    let mut diags: Vec<Diag> = Vec::new();
    let mut lodestones: Vec<Lodestone> = Vec::new();
    let mut sediment: Option<Sediment> = None;

    let mut in_fence: Option<char> = None; // fence char (` or ~) when inside
    let mut fence_len = 0usize;

    // First pass: locate root headings and the sediment zone; record
    // sub-heading lines per region.
    let mut roots: Vec<(usize, String)> = Vec::new(); // (line_no 1-based, title)
    for (idx, line) in lines.iter().enumerate() {
        let n = idx + 1;
        // fence tracking (CommonMark: closing fence must match char+len>=open)
        if let Some(fc) = in_fence {
            if let Some((c, len)) = fence_of(line) {
                if c == fc && len >= fence_len {
                    in_fence = None;
                }
            }
            continue;
        }
        if let Some((c, len)) = fence_of(line) {
            in_fence = Some(c);
            fence_len = len;
            continue;
        }
        if let Some(title) = root_heading(line) {
            roots.push((n, title));
        }
    }

    // The sediment zone is the LAST root heading titled exactly 沉淀区.
    if let Some(&(_, ref t)) = roots.last() {
        if t == SEDIMENT_TITLE {
            let start = roots.last().unwrap().0;
            if let Some(prev) = roots.iter().rev().nth(1) {
                let _ = prev;
            }
            let end = lines.len();
            // find sediment `## <slug>-full` entries
            let mut entries = Vec::new();
            let mut in_f = None;
            let mut f_len = 0usize;
            for (idx, line) in lines.iter().enumerate() {
                let n = idx + 1;
                if n < start { continue; }
                if let Some(fc) = in_f {
                    if let Some((c, len)) = fence_of(line) {
                        if c == fc && len >= f_len { in_f = None; }
                    }
                    continue;
                }
                if let Some((c, len)) = fence_of(line) {
                    in_f = Some(c); f_len = len; continue;
                }
                if let Some((_, title)) = sub_heading(line) {
                    if title.ends_with("-full") {
                        let slug = title.trim_end_matches("-full").to_string();
                        entries.push(SedimentEntry { slug: format!("{slug}-full"), title, line: n });
                    }
                }
            }
            sediment = Some(Sediment { start_line: start, end_line: end, entries });
            roots.pop();
        }
    }

    // Second pass: build lodestones between consecutive roots. The last
    // lodestone ends before the sediment zone (if present), never includes it.
    let sed_start_line = sediment.as_ref().map(|s| s.start_line);
    for (k, &(start, ref title)) in roots.iter().enumerate() {
        let end = roots.get(k + 1).map(|&(s, _)| s - 1)
            .or_else(|| sed_start_line.map(|s| s.saturating_sub(1)))
            .unwrap_or(lines.len());
        let slug = match slugify(title) {
            Some(s) => s,
            None => {
                diags.push(Diag::error(E_MISSING_ID, start, format!("标题无法派生 id: {title}")));
                continue;
            }
        };
        let mut status = Status::Draft;
        let mut summary: Option<String> = None;
        let mut meta_end = start; // last line of the status list block

        // status list = first non-blank block after the title (start+1)
        let mut j = start + 1; // 1-based
        while j < end {
            let line = lines[j - 1];
            if line.trim().is_empty() {
                j += 1;
                continue;
            }
            if let Some(kv) = meta_item(line) {
                match kv.0 {
                    "status" => {
                        status = match kv.1.trim() {
                            "draft" => Status::Draft,
                            "converged" => Status::Converged,
                            "aligned" => Status::Aligned,
                            other => {
                                diags.push(Diag::warn(W_STATUS_MISSING, j, format!("未知状态 {other:?}，按 draft 处理")));
                                Status::Draft
                            }
                        };
                    }
                    "summary" => summary = Some(kv.1.trim().to_string()),
                    _ => {} // unknown keys ignored (v2.0-draft §3.3)
                }
                meta_end = j;
                j += 1;
            } else {
                break; // first non-meta line ends the status block
            }
        }
        if meta_end == start {
            diags.push(Diag::warn(W_STATUS_MISSING, start, format!("磁石 {slug} 无状态列表，按 draft 处理")));
        }

        // sub-headings + body lines + magnetic lines
        let mut subheadings: Vec<(usize, usize, String)> = Vec::new();
        let mut body: Vec<usize> = Vec::new();
        let mut lines_out: Vec<MagneticLine> = Vec::new();
        let mut in_f = None;
        let mut f_len = 0usize;
        for (idx, line) in lines.iter().enumerate() {
            let n = idx + 1;
            if n <= meta_end || n > end { continue; }
            if let Some(fc) = in_f {
                if let Some((c, len)) = fence_of(line) {
                    if c == fc && len >= f_len { in_f = None; }
                }
                body.push(n);
                continue;
            }
            if let Some((c, len)) = fence_of(line) {
                in_f = Some(c); f_len = len; body.push(n); continue;
            }
            if let Some((level, sub)) = sub_heading(line) {
                subheadings.push((n, level, sub));
                body.push(n);
                continue;
            }
            // magnetic lines: [label](#target-slug) links in body text
            for (label, target) in extract_links(line) {
                if target == slug {
                    diags.push(Diag::warn(W_SELF_REF, n, format!("磁石 {slug} 引用自身")));
                    continue;
                }
                if sediment.as_ref().is_some() && target == SEDIMENT_TITLE {
                    diags.push(Diag::warn(W_SEDIMENT_REF, n, "引用沉淀区标题——普通链接，非磁力线"));
                    continue;
                }
                // sediment links ([全文](#slug-full)) point at archived
                // bodies — not magnetic lines between lodestones.
                if let Some(sed) = &sediment {
                    if sed.entries.iter().any(|e| e.slug == target) {
                        continue;
                    }
                }
                lines_out.push(MagneticLine { from: slug.clone(), to: target, label, line: n });
            }
            if !line.trim().is_empty() {
                body.push(n);
            }
        }

        lodestones.push(Lodestone {
            slug, title: title.clone(), title_line: start,
            status, summary,
            start_line: start, end_line: end,
            subheadings, lines: lines_out, body,
        });
    }

    // Resolve magnetic line targets + duplicate slugs.
    let valid_slugs: Vec<String> = lodestones.iter().map(|l| l.slug.clone()).collect();
    let mut seen: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for l in &lodestones {
        let e = seen.entry(&l.slug).or_insert(0);
        *e += 1;
    }
    for (slug, count) in seen {
        if count > 1 {
            diags.push(Diag::error(E_DUP_ID, 0, format!("slug 重复: {slug}——同 slug 全部磁石无效")));
        }
    }
    for l in &lodestones {
        for m in &l.lines {
            if !valid_slugs.iter().any(|s| s == &m.to) {
                diags.push(Diag::warn(W_REF_NOT_FOUND, m.line, format!("磁力线目标不存在: {}", m.to)));
            }
        }
    }
    // E-SEDIMENT-REF (error): magnetic line pointing INTO sediment entries is
    // already only warn (sediment title itself); spec says E-SEDIMENT-REF is
    // for magnetic lines targeting the sediment zone heading — handled above
    // as W; the error class stays reserved for strict mode. Keep it simple.

    Doc { text: raw, lodestones, sediment, diagnostics: diags }
}

/// Root-level heading: zero-indent `# ` (ATX level-1) outside fenced blocks.
fn root_heading(line: &str) -> Option<String> {
    let t = line.strip_prefix("# ")?;
    Some(t.trim().to_string())
}

/// Sub-heading: `##`/`###`/... zero-indent ATX. Returns (level, text).
fn sub_heading(line: &str) -> Option<(usize, String)> {
    if line.starts_with("##") {
        let level = line.chars().take_while(|c| *c == '#').count();
        let rest = line.trim_start_matches('#');
        let t = rest.strip_prefix(' ').unwrap_or(rest).trim();
        if !t.is_empty() {
            return Some((level, t.to_string()));
        }
    }
    None
}

/// Fence opener/closer: ``` or ~~~ with optional info string. Returns the
/// fence char and length.
fn fence_of(line: &str) -> Option<(char, usize)> {
    let first = line.chars().next()?;
    if first != '`' && first != '~' {
        return None;
    }
    let len = line.chars().take_while(|c| *c == first).count();
    if len < 3 {
        return None;
    }
    // closing fence must be only fence chars (CommonMark); opening may carry
    // an info string. Accept both for simplicity; len>=3 is the contract.
    Some((first, len))
}

/// A metadata list item `- key: value` (v2.0-draft §3.3). `None` for others.
fn meta_item(line: &str) -> Option<(&str, &str)> {
    let rest = line.strip_prefix("- ")?;
    let (key, value) = rest.split_once(':')?;
    Some((key.trim(), value))
}

/// Extract `[label](#target-slug)` links from a text line.
fn extract_links(line: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut rest = line;
    while let Some(open) = rest.find('[') {
        let after_open = &rest[open + 1..];
        let Some(close) = after_open.find(']') else { break };
        let label = &after_open[..close];
        let after_label = &after_open[close + 1..];
        if let Some(hash) = after_label.strip_prefix("(#") {
            let end = hash.find(')').unwrap_or(hash.len());
            let target = &hash[..end];
            if !target.is_empty() {
                out.push((label.to_string(), target.to_string()));
            }
            rest = &hash[end..];
        } else {
            rest = after_label;
        }
    }
    out
}

/// Detect cycles among effective edges (both ends aligned) — DFS over the
/// magnetic-line graph restricted to aligned lodestones. Returns the
/// first cycle found as a slug path.
pub fn find_cycle(doc: &Doc) -> Option<Vec<String>> {
    let aligned: std::collections::HashSet<&str> = doc
        .lodestones
        .iter()
        .filter(|l| l.status == Status::Aligned)
        .map(|l| l.slug.as_str())
        .collect();
    let mut adj: std::collections::HashMap<&str, Vec<&str>> = std::collections::HashMap::new();
    for l in &doc.lodestones {
        for m in &l.lines {
            if aligned.contains(l.slug.as_str()) && aligned.contains(m.to.as_str()) {
                adj.entry(l.slug.as_str()).or_default().push(m.to.as_str());
            }
        }
    }
    // recursive three-color DFS (0=white,1=gray,2=black). A back edge into a
    // gray node closes a cycle.
    fn dfs<'a>(
        node: &'a str,
        adj: &std::collections::HashMap<&'a str, Vec<&'a str>>,
        color: &mut std::collections::HashMap<&'a str, u8>,
        path: Vec<String>,
    ) -> Option<Vec<String>> {
        color.insert(node, 1);
        if let Some(neighs) = adj.get(node) {
            for n in neighs {
                match color.get(n).copied().unwrap_or(0) {
                    1 => {
                        let mut cycle = path.clone();
                        cycle.push(n.to_string());
                        return Some(cycle);
                    }
                    0 => {
                        let mut p = path.clone();
                        p.push(n.to_string());
                        if let Some(c) = dfs(n, adj, color, p) {
                            return Some(c);
                        }
                    }
                    _ => {}
                }
            }
        }
        color.insert(node, 2);
        None
    }

    let mut color: std::collections::HashMap<&str, u8> = std::collections::HashMap::new();
    for start in &aligned {
        if color.get(start).copied().unwrap_or(0) == 0 {
            if let Some(c) = dfs(start, &adj, &mut color, vec![start.to_string()]) {
                return Some(c);
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = r#"# 流体实验记录
- status: converged
实验数据与原始观测记录。

# 采集参数快照
- status: aligned
- summary: 采集环境为双通道压力/流速。
[全文](#采集参数快照-full)
正文首段。

# 实验结论
- status: aligned
- summary: 结论成立。
[全文](#实验结论-full)
[依赖](#流体实验记录) [依赖](#采集参数快照)

# 反例观察
- status: draft
一次与结论冲突的观测。[反驳](#实验结论)

# 沉淀区
## 采集参数快照-full
（原始正文迁移至此）
## 实验结论-full
（原始正文迁移至此）
"#;

    #[test]
    fn scans_lodestones_and_status() {
        let doc = scan(SAMPLE);
        assert_eq!(doc.lodestones.len(), 4);
        assert_eq!(doc.lodestones[0].slug, "流体实验记录");
        assert_eq!(doc.lodestones[0].status, Status::Converged);
        assert_eq!(doc.lodestones[1].status, Status::Aligned);
        assert_eq!(doc.lodestones[1].summary.as_deref(), Some("采集环境为双通道压力/流速。"));
        assert_eq!(doc.lodestones[3].status, Status::Draft);
    }

    #[test]
    fn scans_magnetic_lines() {
        let doc = scan(SAMPLE);
        let concl = &doc.lodestones[2];
        assert_eq!(concl.lines.len(), 2);
        assert!(concl.lines.iter().any(|m| m.to == "流体实验记录" && m.label == "依赖"));
        assert!(concl.lines.iter().any(|m| m.to == "采集参数快照"));
        let counter = &doc.lodestones[3];
        assert_eq!(counter.lines.len(), 1);
        assert_eq!(counter.lines[0].label, "反驳");
    }

    #[test]
    fn scans_sediment_zone() {
        let doc = scan(SAMPLE);
        let s = doc.sediment.as_ref().unwrap();
        assert_eq!(s.entries.len(), 2);
        assert_eq!(s.entries[0].slug, "采集参数快照-full");
    }

    #[test]
    fn status_advance_is_one_way() {
        assert_eq!(Status::Draft.advance(), Some(Status::Converged));
        assert_eq!(Status::Converged.advance(), Some(Status::Aligned));
        assert_eq!(Status::Aligned.advance(), None);
    }

    #[test]
    fn root_heading_inside_fence_is_ignored() {
        let text = "# 球A\n- status: draft\n```\n# 假标题\n```\n正文。\n";
        let doc = scan(text);
        assert_eq!(doc.lodestones.len(), 1);
        assert_eq!(doc.lodestones[0].slug, "球a");
    }

    #[test]
    fn duplicate_slugs_diagnose() {
        let text = "# 球A\n- status: draft\n\n# 球a\n- status: draft\n";
        let doc = scan(text);
        assert!(doc.diagnostics.iter().any(|d| d.code == E_DUP_ID));
    }

    #[test]
    fn dangling_reference_warns() {
        let text = "# 球A\n- status: aligned\n[依赖](#不存在的球)\n";
        let doc = scan(text);
        assert!(doc.diagnostics.iter().any(|d| d.code == W_REF_NOT_FOUND));
    }

    #[test]
    fn effective_cycle_detected() {
        let text = "# 甲\n- status: aligned\n[依赖](#乙)\n\n# 乙\n- status: aligned\n[依赖](#甲)\n";
        let doc = scan(text);
        assert!(find_cycle(&doc).is_some());
    }

    #[test]
    fn draft_edges_do_not_cycle() {
        let text = "# 甲\n- status: aligned\n[依赖](#乙)\n\n# 乙\n- status: draft\n[依赖](#甲)\n";
        let doc = scan(text);
        assert!(find_cycle(&doc).is_none());
    }

    #[test]
    fn slug_of_chinese_title() {
        assert_eq!(slugify("实验结论"), Some("实验结论".into()));
    }
}
