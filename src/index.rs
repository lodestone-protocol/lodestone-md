//! Library-layer projection (v2.0-draft §5.2c / ADR-0003): the `.lodestone`
//! index. Scans a set of session documents, validates the cross-document
//! graph (path exists, target slug exists, no cross-document cycle), and
//! emits a deterministic mddag document that `scan` can re-parse — the
//! index is a snapshot, not a second protocol.
//!
//! Deterministic: no timestamps, no randomness; `keep` windowing belongs to
//! `library` (instant query), this index is full (physical-facts-first).
//! Inbound edges are derived on demand from the outgoing set (a bijection —
//! storing both would duplicate facts, violating energy thrift).

use std::collections::HashMap;

use crate::diag::{Diag, E_CROSS_MISSING, E_CROSS_SLUG, E_CYCLE_CROSS};
use crate::doc::CrossLink;
use crate::library::Session;

/// Project the whole library into `.lodestone` text. `Err` carries all
/// library-layer violations (path missing / slug missing / cycles).
pub fn project(sessions: &[Session]) -> Result<String, Vec<Diag>> {
    let mut diags: Vec<Diag> = Vec::new();
    let mut by_path: HashMap<&str, &Session> = HashMap::new();
    for s in sessions {
        by_path.insert(&s.path, s);
    }

    let mut edges: Vec<(&CrossLink, &str, &str)> = Vec::new(); // (link, from_path, to_path)
    let mut total_lodes = 0usize;
    for s in sessions {
        total_lodes += s.doc.lodestones.len();
        for cl in &s.doc.cross_links {
            let to_path = cl.path.as_str();
            if !by_path.contains_key(to_path) {
                diags.push(Diag::error(
                    E_CROSS_MISSING,
                    cl.line,
                    format!("跨文档引用目标文件不存在: {to_path}"),
                ));
                continue;
            }
            let target = by_path[to_path];
            let hit = target
                .doc
                .lodestones
                .iter()
                .any(|l| l.slug == cl.to_slug);
            if !hit {
                diags.push(Diag::error(
                    E_CROSS_SLUG,
                    cl.line,
                    format!("目标磁石不存在: {to_path}#{}", cl.to_slug),
                ));
                continue;
            }
            edges.push((cl, s.path.as_str(), to_path));
        }
    }

    if let Some((a, b)) = find_cross_cycle(&edges) {
        diags.push(Diag::error(
            E_CYCLE_CROSS,
            0,
            format!("跨文档引用循环: {a} → {b}"),
        ));
    }

    if !diags.is_empty() {
        return Err(diags);
    }

    let edge_count = edges.len();
    let session_count = sessions.len();
    let mut out = String::from("- session: library-index\n# 经历库\n- status: aligned\n");
    out.push_str(&format!(
        "- summary: {session_count} 会话 · {total_lodes} 磁石 · {edge_count} 磁力线\n"
    ));
    for s in sessions {
        let created = s
            .doc
            .meta
            .iter()
            .find(|(k, _)| k == "created")
            .map(|(_, v)| v.clone())
            .unwrap_or_default();
        let name = s.path.rsplit('/').next().unwrap_or(&s.path);
        let date = if created.is_empty() { "?" } else { created.as_str() };
        out.push_str(&format!("## {name}  [{date}]\n"));
        let mut lines: Vec<&CrossLink> = s.doc.cross_links.iter().collect();
        lines.sort_by(|a, b| (a.to_slug.as_str(), a.label.as_str()).cmp(&(b.to_slug.as_str(), b.label.as_str())));
        for cl in lines {
            out.push_str(&format!("[{}]({}#{})\n", cl.label, cl.path, cl.to_slug));
        }
    }
    Ok(out)
}

/// Inbound derivation (ADR-0003 D5): reverse of the outgoing set. Returns
/// every cross-document reference whose target is `path#slug`. Not stored —
/// derived on demand from the single source of fact (outgoing edges).
pub fn inbound<'a>(sessions: &'a [Session], path: &str, slug: &str) -> Vec<&'a CrossLink> {
    let mut out: Vec<&CrossLink> = Vec::new();
    for s in sessions {
        for cl in &s.doc.cross_links {
            if cl.path == path && cl.to_slug == slug {
                out.push(cl);
            }
        }
    }
    out
}

/// Detect a cross-document cycle in the edge graph. Nodes are documents;
/// an edge `A → B` exists when A carries at least one cross-link into B.
/// Returns one cycle pair (first found), if any.
fn find_cross_cycle<'a>(edges: &[(&'a CrossLink, &'a str, &'a str)]) -> Option<(&'a str, &'a str)> {
    let mut adj: HashMap<&str, Vec<&str>> = HashMap::new();
    for (_, from, to) in edges {
        adj.entry(from).or_default().push(to);
    }
    let mut state: HashMap<&str, u8> = HashMap::new(); // 0=unseen 1=visiting 2=done
    fn dfs<'a>(
        node: &'a str,
        adj: &HashMap<&'a str, Vec<&'a str>>,
        state: &mut HashMap<&'a str, u8>,
    ) -> Option<(&'a str, &'a str)> {
        state.insert(node, 1);
        if let Some(neigh) = adj.get(node) {
            let mut sorted = neigh.clone();
            sorted.sort_unstable();
            for n in sorted {
                match state.get(n) {
                    Some(1) => return Some((node, n)),
                    Some(2) => {}
                    _ => {
                        if let Some(c) = dfs(n, adj, state) {
                            return Some(c);
                        }
                    }
                }
            }
        }
        state.insert(node, 2);
        None
    }
    let mut keys: Vec<&str> = adj.keys().copied().collect();
    keys.sort_unstable();
    for k in keys {
        if let Some(c) = dfs(k, &adj, &mut state) {
            return Some(c);
        }
    }
    None
}

/// Validate that a stale index file still matches a fresh scan — the
/// byte-level consistency check (ADR-0003 §5 risk: index expiry).
pub fn check_fresh(sessions: &[Session], existing: &str) -> Result<(), String> {
    match project(sessions) {
        Ok(fresh) if fresh == existing => Ok(()),
        Ok(_) => Err("索引已过期：文档与 .lodestone 不一致，请重新生成（mddag index <dir>）".to_string()),
        Err(d) => Err(format!("索引校验失败：{}", describe(&d))),
    }
}

fn describe(d: &[Diag]) -> String {
    let mut s = String::new();
    for x in d {
        s.push_str(&format!("{} L{}: {}; ", x.code, x.line, x.message));
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scan::scan;

    fn session(path: &str, created: &str, body: &str) -> Session {
        let text = format!("- created: {created}\n{body}");
        Session { path: path.to_string(), doc: scan(&text) }
    }

    #[test]
    fn scan_parses_cross_doc_link_and_diags() {
        let doc = scan("# 甲\n[依赖](s2.md#实验记录)\n");
        assert_eq!(doc.cross_links.len(), 1);
        assert_eq!(doc.cross_links[0].path, "s2.md");
        assert_eq!(doc.cross_links[0].to_slug, "实验记录");
        assert_eq!(doc.cross_links[0].from_slug, "甲");
        assert!(doc.diagnostics.iter().any(|d| d.code == "W-CROSS-DOC"));
        // local link is NOT a cross link
        let doc2 = scan("# 甲\n[依赖](#乙)\n");
        assert_eq!(doc2.cross_links.len(), 0);
    }

    #[test]
    fn project_emits_deterministic_index() {
        let sessions = vec![
            session("s1.md", "2026-08-20", "# 旧\n[依赖](s2.md#实验记录)\n"),
            session("s2.md", "2026-09-01", "# 实验记录\n"),
        ];
        let out = project(&sessions).unwrap();
        assert!(out.starts_with("- session: library-index\n"));
        assert!(out.contains("## s1.md  [2026-08-20]"));
        assert!(out.contains("[依赖](s2.md#实验记录)"));
        assert!(out.contains("2 会话 · 2 磁石 · 1 磁力线"));
        // deterministic: same input -> same bytes
        assert_eq!(project(&sessions).unwrap(), out);
    }

    #[test]
    fn missing_target_file_or_slug_fails() {
        let sessions = vec![
            session("s1.md", "", "# 甲\n[依赖](nope.md#x)\n"),
        ];
        assert!(project(&sessions).is_err());
        let sessions = vec![
            session("s1.md", "", "# 甲\n[依赖](s2.md#不存在)\n"),
            session("s2.md", "", "# 乙\n"),
        ];
        assert!(project(&sessions).is_err());
    }

    #[test]
    fn cross_doc_cycle_detected() {
        let sessions = vec![
            session("s1.md", "", "# 甲\n[看乙](s2.md#乙)\n"),
            session("s2.md", "", "# 乙\n[看甲](s1.md#甲)\n"),
        ];
        assert!(project(&sessions).is_err());
    }

    #[test]
    fn inbound_derives_from_outgoing() {
        let sessions = vec![
            session("s1.md", "", "# 甲\n[依赖](s2.md#实验记录)\n"),
            session("s2.md", "", "# 实验记录\n"),
        ];
        let inb = inbound(&sessions, "s2.md", "实验记录");
        assert_eq!(inb.len(), 1);
        assert_eq!(inb[0].from_slug, "甲");
    }
}
