//! Library projection (v2.0-draft §4/§5.2b): the cross-session window.
//! Scans a set of session documents (each = one experience), sorts by
//! `created` metadata, and projects an index: the `keep` most recent
//! sessions show their full L0 lodestone list; older sessions collapse to
//! one summary line (their edges have exited the global graph — the
//! hidden branch, retrievable on demand).
//!
//! Deterministic: `keep` is injected (never hardcoded; 12 is only an
//! example). Sorting ties break on the document path.

use crate::doc::Doc;
use crate::project::l0;

/// A session document ready for projection.
#[derive(Debug, Clone)]
pub struct Session {
    pub path: String,
    pub doc: Doc,
}

/// Project the library index. `keep` = number of sessions kept fully
/// visible (in-window); older ones collapse to summaries.
pub fn index(sessions: &[Session], keep: usize) -> String {
    // deterministic sort: (created, path) — created defaults to "" and
    // sorts first; consumers should write `- created:` on every session.
    let mut items: Vec<(&Session, String, String)> = sessions
        .iter()
        .map(|s| {
            let created = s
                .doc
                .meta
                .iter()
                .find(|(k, _)| k == "created")
                .map(|(_, v)| v.clone())
                .unwrap_or_default();
            (s, created, s.path.clone())
        })
        .collect();
    items.sort_by(|a, b| (a.1.as_str(), a.2.as_str()).cmp(&(b.1.as_str(), b.2.as_str())));

    let mut out = String::from("# 经历库\n");
    let total = items.len();
    for (i, (s, created, path)) in items.iter().enumerate() {
        let in_window = total.saturating_sub(i) <= keep;
        let name = path.rsplit('/').next().unwrap_or(path);
        let date = if created.is_empty() { "?" } else { created.as_str() };
        out.push_str(&format!("## {name}  [{date}]\n"));
        if in_window {
            let l = l0(&s.doc);
            for line in l.lines() {
                out.push_str(&format!("  {line}\n"));
            }
        } else {
            out.push_str(&format!(
                "  （窗口外 · 隐性分支 · {} 磁石 · 按需回忆）\n",
                s.doc.lodestones.len()
            ));
        }
    }
    out
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
    fn window_keeps_recent_full_l0() {
        let sessions = vec![
            session("s1.md", "2026-09-01", "# 旧会话\n- status: draft\n"),
            session("s2.md", "2026-09-05", "# 新会话\n- status: aligned\n- summary: 新结论。\n"),
        ];
        let out = index(&sessions, 1);
        assert!(out.contains("## s1.md  [2026-09-01]"));
        assert!(out.contains("窗口外 · 隐性分支"));
        assert!(out.contains("## s2.md  [2026-09-05]"));
        assert!(out.contains("# 新会话  [aligned]"));
        assert!(out.contains("新结论。"));
    }

    #[test]
    fn keep_zero_collapses_all() {
        let sessions = vec![
            session("s1.md", "2026-09-01", "# 甲\n- status: draft\n"),
            session("s2.md", "2026-09-02", "# 乙\n- status: draft\n"),
        ];
        let out = index(&sessions, 0);
        assert!(out.contains("窗口外"));
        assert!(!out.contains("# 甲  [draft]"));
        assert_eq!(out.matches("1 磁石").count(), 2);
    }

    #[test]
    fn missing_created_sorts_first_and_marks_question() {
        let sessions = vec![
            session("s1.md", "2026-09-02", "# 甲\n"),
            Session { path: "s2.md".into(), doc: scan("# 乙\n- status: draft\n") },
        ];
        let out = index(&sessions, 1);
        // s2 (no created) is oldest -> out of window
        assert!(out.contains("## s2.md  [?]"));
        assert!(out.contains("窗口外"));
        assert!(out.contains("## s1.md  [2026-09-02]"));
    }
}
