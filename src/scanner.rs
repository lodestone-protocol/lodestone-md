//! Node boundary scanning (spec §4.1): based on CommonMark 0.31.2 block-level
//! parsing (comrak, ADR-0001).
//!
//! A boundary is an ATX level-1 heading in the document's top-level block
//! sequence with zero leading whitespace (col == 1). Setext headings,
//! fenced/indented code block content, container content, and 1–3-space
//! indented headings are not boundaries.

use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, Options};

pub struct Heading {
    /// Physical line (1-based)
    pub line: usize,
    /// Physical column (1-based); the protocol requires col == 1 (zero indent)
    pub col: usize,
    /// Heading text (aggregated inline text, for slug derivation)
    pub text: String,
}

pub struct ScanOut {
    pub headings: Vec<Heading>,
    /// Code block (fenced and indented) physical line ranges [start, end],
    /// so metadata scanning can skip fenced content
    pub code_ranges: Vec<(usize, usize)>,
}

pub fn scan(text: &str) -> ScanOut {
    let arena = Arena::new();
    let root = parse_document(&arena, text, &Options::default());

    let mut headings = Vec::new();
    for child in root.children() {
        let ast = child.data.borrow();
        let sp = ast.sourcepos;
        if let NodeValue::Heading(h) = &ast.value {
            if h.level == 1 && !h.setext {
                let text = collect_text(child);
                headings.push(Heading {
                    line: sp.start.line,
                    col: sp.start.column,
                    text,
                });
            }
        }
    }

    let mut code_ranges = Vec::new();
    collect_code_ranges(root, &mut code_ranges);
    code_ranges.sort_unstable();
    code_ranges.dedup();
    headings.sort_by_key(|h| h.line);

    ScanOut {
        headings,
        code_ranges,
    }
}

fn collect_text<'a>(node: &'a AstNode<'a>) -> String {
    let mut out = String::new();
    collect_text_rec(node, &mut out);
    out
}

fn collect_text_rec<'a>(node: &'a AstNode<'a>, out: &mut String) {
    let ast = node.data.borrow();
    match &ast.value {
        NodeValue::Text(t) => out.push_str(t),
        NodeValue::Code(c) => out.push_str(&c.literal),
        _ => {}
    }
    for child in node.children() {
        collect_text_rec(child, out);
    }
}

fn collect_code_ranges<'a>(node: &'a AstNode<'a>, out: &mut Vec<(usize, usize)>) {
    for child in node.children() {
        {
            let ast = child.data.borrow();
            if let NodeValue::CodeBlock(_) = &ast.value {
                let sp = ast.sourcepos;
                out.push((sp.start.line, sp.end.line));
            }
        }
        collect_code_ranges(child, out);
    }
}
