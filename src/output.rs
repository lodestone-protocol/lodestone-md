//! Output contract (spec §8.1–§8.4). Anti-corruption rule 2: contract is
//! frozen; extensions are append-only.
//!
//! `NodeEntry.title` is an append-only implementation extension placed after
//! the spec fields: invalid nodes must stay visible as placeholders (with
//! their heading), and the title is the first discovery anchor of the L1
//! skeleton.

use serde::Serialize;

use crate::diag::Diagnostic;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct DocMetaOut {
    /// Declared document version; null when not declared.
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
    /// Implementation extension (append-only): heading text.
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
