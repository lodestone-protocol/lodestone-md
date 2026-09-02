//! 节点元数据提取与三层校验（规范 §5）。
//!
//! 层序：形态层（位置/前缀/单行/SUFFIX）→ JSON 层（含重复键）→ 字段层（降级）。

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

/// 字段层校验通过后的字段集合。
pub struct Fields {
    /// None = 未声明；Some(Ok) = 合法声明；Some(Err) = 声明违反字符集/长度（§5.3：视为无 id）
    pub id: Option<Result<String, ()>>,
    pub status: String,
    /// (to, relation) 声明出边（形态合法者）
    pub declared_edges: Vec<(String, String)>,
    pub tags: Vec<String>,
}

pub enum MetaOutcome {
    /// 无元数据（含近似变体/位置靠后被拒）
    Absent,
    /// 三层校验通过（可能带字段级降级）
    Parsed(Fields),
    /// E-META-SYNTAX：节点无效
    Invalid,
}

/// 从节点区间 (title_line, end_line]（1-based，含端点）提取元数据。
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

    // 采纳前提：首个 mddag 注释恰好位于标题行后第一个非空行。
    if Some(md_lines[0]) != first_non_empty {
        // 位置靠后 / 缩进 / 变体：节点无元数据。
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
        // 近似前缀变体（缺空格）或缩进形态：W-META-PLACEMENT，无元数据。
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
        // 顶格以精确前缀开始但未以 --> 结尾（跨行或未闭合）：E-META-SYNTAX。
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

    // 采纳。注释体 = 两定界符之间子串。
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

    // 采纳后的后续 mddag 注释：忽略并报告 W-REDUNDANT-META。
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

/// JSON 层 + 字段层。返回 None 表示 JSON 层失败（节点无效）。
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

    // id 先行解析，供后续字段诊断携带 node_id。
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

    // status：非法值回退 draft。
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

    // edges：非法条目剔除，其余保留。
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

    // tags：非字符串数组则忽略。
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

    // updated：信息性字段，不参与协议层计算，忽略内容。
    // 未知字段：忽略。

    Some(Fields {
        id,
        status,
        declared_edges,
        tags,
    })
}
