//! Lodestone Protocol (MD-DAG) v1.3 Final 参考解析器。
//!
//! 单一解析管线（DNA 特有铁律 4）：所有输出出自 [`parse`] 一趟结果（规范 §8）。
//! 解析确定性：输出是文档当前字节的纯函数，无全局可变状态。

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

/// 解析一个 Lodestone (MD-DAG) 文档。
pub fn parse(input: &str) -> ParseResult {
    // BOM 剥离（§3.1：文档级元数据位置在 BOM 之后）。
    let text = input.strip_prefix('\u{FEFF}').unwrap_or(input);
    let lines = lines::split_lines(text);
    let scan = scanner::scan(text);
    let in_code = |line: usize| scan.code_ranges.iter().any(|&(a, b)| line >= a && line <= b);

    let mut diagnostics: Vec<Diagnostic> = Vec::new();

    // §8 步骤 2：文档级元数据（§3.1 行为钉死）。
    let doc_meta = docmeta::extract(lines.first().map(|s| s.as_str()), &mut diagnostics);

    // 边界：顶层 ATX 一级标题且零缩进（§4.1 收窄：1–3 空格缩进视为正文）。
    let boundaries: Vec<&Heading> = scan
        .headings
        .iter()
        .filter(|h| h.col == 1)
        .collect();

    // §3.2 节点 0 扫描：MUST NOT 携带节点元数据，MUST NOT 被引用；仅检测 W-META-PLACEMENT。
    let node0_end = boundaries
        .first()
        .map(|h| h.line - 1)
        .unwrap_or(lines.len());
    for idx in 1..=node0_end {
        if in_code(idx) {
            continue;
        }
        // 首行若为文档级元数据形态，归 §3.1 管辖，不在节点 0 重复告警。
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

    // §8 步骤 3–5：节点元数据三层校验、E-DUP-ID 前置、派生字段。
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

        // §4.2：标题未采用 NFC 归一化形态 → W-NFC-VIOLATION（SHOULD）。
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

    // §8 步骤 4：E-DUP-ID——同 id 的全部有效节点判无效；无效节点不参与检测，不级联。
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

    // §8 步骤 5：派生字段（对有效与无效节点统一适用）。
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

    // §8 步骤 6–9：边规范化、引用校验、软校验、生效判定。
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

    // §8 步骤 10：输出。
    ParseResult {
        doc_meta,
        nodes,
        edges: edge_entries,
        diagnostics,
        graph,
    }
}

/// 供消费方使用的状态常量再导出。
pub mod states {
    pub use crate::nodemeta::{STATUS_ALIGNED, STATUS_CONVERGED, STATUS_DRAFT};
}

/// L2 定点正文（§9 三级加载）：按节点 id 返回正文文本。
///
/// 文本与派生字段一致：正文区间内各行以 U+000A 连接（不含末行换行）。
/// id 不存在或正文为空时返回 None 之外，仍返回 Some("")（空正文是合法状态）。
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
