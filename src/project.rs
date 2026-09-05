//! Read protocol projections (v2.0-draft §4): L0 lodestone list, L1 single
//! lodestone expansion, L2 body fragment, sediment index. The output strings
//! are the contract face for agents — choices, not free text.

use crate::doc::{Doc, Status};

/// L0 — lodestone name list (global awareness). One line per lodestone:
/// `# <title>  [<status>]` plus a summary line for aligned ones. Sediment
/// zone is excluded unless requested.
pub fn l0(doc: &Doc) -> String {
    let mut out = String::new();
    for l in &doc.lodestones {
        out.push_str(&format!("# {}  [{}]\n", l.title, l.status.label()));
        if l.status == Status::Aligned {
            if let Some(s) = &l.summary {
                out.push_str(&format!("  {s}\n"));
            }
        }
    }
    out
}

/// L1 — one lodestone expanded: its sub-heading tree + summary.
pub fn l1(doc: &Doc, slug: &str) -> Option<String> {
    let l = doc.lodestones.iter().find(|l| l.slug == slug)?;
    let mut out = format!("# {}  [{}]\n", l.title, l.status.label());
    if let Some(s) = &l.summary {
        out.push_str(&format!("  summary: {s}\n"));
    }
    for (_, level, sub) in &l.subheadings {
        let indent = "  ".repeat(level.saturating_sub(2));
        out.push_str(&format!("{indent}{sub}\n"));
    }
    if l.subheadings.is_empty() {
        out.push_str("  （无子结构）\n");
    }
    Some(out)
}

/// L2 — body fragment of one lodestone (region minus title + status list).
/// Optionally narrowed to a sub-heading anchor.
pub fn l2(doc: &Doc, slug: &str, anchor: Option<&str>) -> Option<String> {
    let l = doc.lodestones.iter().find(|l| l.slug == slug)?;
    let mut lines: Vec<&str> = Vec::new();
    for &n in &l.body {
        lines.push(doc.text.lines().nth(n - 1).unwrap_or(""));
    }
    if let Some(a) = anchor {
        // narrow: start at the anchor sub-heading, end at next sub-heading
        let start = lines.iter().position(|l| l.trim_start().starts_with("## ") && l.contains(a))?;
        let mut end = lines.len();
        for (i, line) in lines[start + 1..].iter().enumerate() {
            if line.trim_start().starts_with("## ") {
                end = start + 1 + i;
                break;
            }
        }
        lines = lines[start..end].to_vec();
    }
    let mut out = String::new();
    for l in lines {
        out.push_str(l);
        out.push('\n');
    }
    Some(out)
}

/// Sediment index — the converged-body table of contents.
pub fn sediment_index(doc: &Doc) -> String {
    match &doc.sediment {
        None => "（无沉淀区）\n".to_string(),
        Some(s) => {
            let mut out = "# 沉淀区\n".to_string();
            for e in &s.entries {
                out.push_str(&format!("## {}\n", e.title));
            }
            out
        }
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::scan;

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
[依赖](#流体实验记录)

# 沉淀区
## 采集参数快照-full
（原始正文）
"#;

    #[test]
    fn l0_lists_lodestones_with_status() {
        let doc = scan(SAMPLE);
        let out = l0(&doc);
        assert!(out.contains("# 流体实验记录  [converged]"));
        assert!(out.contains("# 采集参数快照  [aligned]"));
        assert!(out.contains("采集环境为双通道压力/流速。"));
        assert!(!out.contains("沉淀区"));
    }

    #[test]
    fn l1_expands_subheadings_and_summary() {
        let doc = scan(SAMPLE);
        let out = l1(&doc, "采集参数快照").unwrap();
        assert!(out.contains("summary: 采集环境为双通道压力/流速。"));
    }

    #[test]
    fn l2_returns_body() {
        let doc = scan(SAMPLE);
        let out = l2(&doc, "实验结论", None).unwrap();
        assert!(out.contains("依赖"));
        assert!(!out.contains("- status:"));
    }

    #[test]
    fn sediment_index_lists_entries() {
        let doc = scan(SAMPLE);
        let out = sediment_index(&doc);
        assert!(out.contains("## 采集参数快照-full"));
    }
}
