//! 节点边界扫描（规范 §4.1）：基于 CommonMark 0.31.2 块级解析（comrak，ADR-0001）。
//!
//! 边界 = 文档顶层块序列中的 ATX 一级标题，且行首零空白缩进（col == 1）。
//! Setext 标题、围栏/缩进代码块内、容器块内、1–3 空格缩进的标题均不构成边界。

use comrak::nodes::{AstNode, NodeValue};
use comrak::{parse_document, Arena, Options};

pub struct Heading {
    /// 物理行号（1-based）
    pub line: usize,
    /// 物理列号（1-based）；协议要求零缩进即 col == 1
    pub col: usize,
    /// 标题文本（内联文本聚合，供 slug 派生）
    pub text: String,
}

pub struct ScanOut {
    pub headings: Vec<Heading>,
    /// 代码块（围栏与缩进）物理行区间 [start, end]，供元数据扫描避开围栏内容
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
