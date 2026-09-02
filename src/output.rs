//! 输出契约（规范 §8.1–§8.4）。DNA 防腐化铁律 2：契约冻结，扩展 Append-Only。
//!
//! `NodeEntry.title` 为实现扩展字段（Append-Only，置于契约字段之后）：
//! 无效节点须"保留为占位（含标题）"，且 title 是 L1 骨架的第一发现锚点。

use serde::Serialize;

use crate::diag::Diagnostic;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct DocMetaOut {
    /// 文档级声明的 version；未声明时为 null。
    pub version: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct NodeEntry {
    pub id: Option<String>,
    pub status: String,
    pub valid: bool,
    pub tags: Vec<String>,
    pub chars: usize,
    pub body_start: Option<usize>,
    pub body_end: Option<usize>,
    /// 实现扩展（Append-Only）：节点标题文本。
    pub title: String,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct EdgeEntry {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub effective: bool,
    pub failure: Option<String>,
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct GraphEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
}

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct Graph {
    pub nodes: Vec<String>,
    pub edges: Vec<GraphEdge>,
}

#[derive(Serialize, Clone, Debug)]
pub struct ParseResult {
    pub doc_meta: Option<DocMetaOut>,
    pub nodes: Vec<NodeEntry>,
    pub edges: Vec<EdgeEntry>,
    pub diagnostics: Vec<Diagnostic>,
    pub graph: Graph,
}
