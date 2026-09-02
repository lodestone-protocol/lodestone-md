//! 视图投影（规范附录 A，非规范性）：边状态标签与人类审查面。
//!
//! 投影是消费方可从（status、规范化边集合、诊断列表）确定性计算的只读视图。
//! 协议仅规范事实，不规范呈现。

use serde::Serialize;

use crate::diag;
use crate::nodemeta::STATUS_ALIGNED;
use crate::output::{EdgeEntry, ParseResult};

#[derive(Serialize, Clone, Copy, Debug, PartialEq, Eq)]
pub enum Label {
    Coherent,
    Pending,
    Nascent,
    Redundant,
    Dangling,
    Cyclic,
}

impl Label {
    pub fn as_str(self) -> &'static str {
        match self {
            Label::Coherent => "coherent",
            Label::Pending => "pending",
            Label::Nascent => "nascent",
            Label::Redundant => "redundant",
            Label::Dangling => "dangling",
            Label::Cyclic => "cyclic",
        }
    }
}

#[derive(Serialize, Clone, Debug)]
pub struct ProjectedEdge {
    pub from: String,
    pub to: String,
    pub relation: String,
    pub label: &'static str,
}

/// 附录 A 条件表。判定优先级（互斥分类）：
/// dangling（E-REF-NOT-FOUND）> cyclic（循环边全集）> redundant（携带折叠警告）
/// > coherent（生效）> pending（恰一端 aligned）> nascent（双端非 aligned）。
pub fn label(edge: &EdgeEntry, result: &ParseResult) -> Label {
    let edge_key = format!("{} -> {}", edge.from, edge.to);
    let carries = |code: &str| {
        result
            .diagnostics
            .iter()
            .any(|d| d.code == code && d.edge.as_deref() == Some(&edge_key))
    };
    if edge.failure.as_deref() == Some(diag::E_REF_NOT_FOUND) || carries(diag::E_REF_NOT_FOUND) {
        return Label::Dangling;
    }
    if edge.failure.as_deref() == Some(diag::E_CYCLE) || carries(diag::E_CYCLE) {
        return Label::Cyclic;
    }
    if carries(diag::W_REDUNDANT_EDGE) {
        return Label::Redundant;
    }
    if edge.effective {
        return Label::Coherent;
    }
    // 引用不可解析的边已在 dangling 分支返回；此处端点均可解析。
    let aligned = |id: &str| {
        result
            .nodes
            .iter()
            .any(|n| n.id.as_deref() == Some(id) && n.status == STATUS_ALIGNED)
    };
    match (aligned(&edge.from), aligned(&edge.to)) {
        (true, false) | (false, true) => Label::Pending,
        _ => Label::Nascent,
    }
}

/// 全部规范化边的投影视图（文档序）。
pub fn project(result: &ParseResult) -> Vec<ProjectedEdge> {
    result
        .edges
        .iter()
        .map(|e| ProjectedEdge {
            from: e.from.clone(),
            to: e.to.clone(),
            relation: e.relation.clone(),
            label: label(e, result).as_str(),
        })
        .collect()
}

/// 人类审查面（§1 消费方角色：检视对齐状态、警告列表与分歧域）。
#[derive(Serialize, Clone, Debug, Default)]
pub struct Review {
    /// 声明 aligned 的节点数 / 节点总数
    pub aligned_nodes: usize,
    pub total_nodes: usize,
    /// 分歧域：全部 refute 规范化边（端点 id + 各自 status）
    pub disputes: Vec<Dispute>,
    /// 悬挂边（dangling）：引用不可解析
    pub dangling: Vec<String>,
    /// 环边（cyclic）
    pub cyclic: Vec<String>,
    /// 对齐欠账（W-UPSTREAM-PENDING）
    pub upstream_pending: Vec<String>,
    /// 其余警告（W-*）概览：码 → 出现次数
    pub other_warnings: Vec<(String, usize)>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Dispute {
    pub from: String,
    pub from_status: String,
    pub to: String,
    pub to_status: String,
}

/// 生成分歧域与审查摘要。输入须为同一趟解析结果（单一管线）。
pub fn review(result: &ParseResult) -> Review {
    let status_of = |id: &str| -> String {
        result
            .nodes
            .iter()
            .find(|n| n.id.as_deref() == Some(id))
            .map(|n| n.status.clone())
            .unwrap_or_default()
    };

    let mut rev = Review {
        aligned_nodes: result
            .nodes
            .iter()
            .filter(|n| n.valid && n.status == STATUS_ALIGNED)
            .count(),
        total_nodes: result.nodes.len(),
        ..Review::default()
    };

    let mut warning_counts: std::collections::BTreeMap<String, usize> = Default::default();
    for e in &result.edges {
        let l = label(e, result);
        match l {
            Label::Dangling => rev.dangling.push(format!("{} -> {}", e.from, e.to)),
            Label::Cyclic => rev.cyclic.push(format!("{} -> {}", e.from, e.to)),
            _ => {}
        }
        if e.relation == "refute" {
            rev.disputes.push(Dispute {
                from: e.from.clone(),
                from_status: status_of(&e.from),
                to: e.to.clone(),
                to_status: status_of(&e.to),
            });
        }
    }
    for d in &result.diagnostics {
        if d.code == diag::W_UPSTREAM_PENDING {
            if let Some(e) = &d.edge {
                rev.upstream_pending.push(e.clone());
            }
        } else if d.level == "warning" {
            *warning_counts.entry(d.code.clone()).or_default() += 1;
        }
    }
    rev.other_warnings = warning_counts.into_iter().collect();
    rev
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parse;

    #[test]
    fn spec_10_projection() {
        let input = include_str!("../tests/fixtures/10_example.md");
        let r = parse(input);
        let p = project(&r);
        let get = |from: &str, to: &str| {
            p.iter()
                .find(|e| e.from == from && e.to == to)
                .map(|e| e.label)
                .unwrap()
        };
        // concl-01(converged) → exp-01(aligned)：恰一端 aligned → pending
        assert_eq!(get("concl-01", "exp-01"), "pending");
        // counter-01(draft) → concl-01(converged)：双端非 aligned → nascent
        assert_eq!(get("counter-01", "concl-01"), "nascent");
        // plan-01 → old-note：引用不可解析 → dangling
        assert_eq!(get("plan-01", "old-note"), "dangling");
    }

    #[test]
    fn coherent_and_cyclic() {
        let input = include_str!("../tests/fixtures/cycle_global.md");
        let r = parse(input);
        let p = project(&r);
        let coherent = p
            .iter()
            .filter(|e| e.label == "coherent")
            .count();
        let cyclic = p.iter().filter(|e| e.label == "cyclic").count();
        assert_eq!(coherent, 1); // cc -> cd
        assert_eq!(cyclic, 3);

        let rev = review(&r);
        assert_eq!(rev.cyclic.len(), 3);
        assert_eq!(rev.aligned_nodes, 4);
    }

    #[test]
    fn redundant_label() {
        let input = include_str!("../tests/fixtures/edges_derive_fold.md");
        let r = parse(input);
        let p = project(&r);
        // 折叠后的幸存边携带 W-REDUNDANT-EDGE → redundant
        let d = p
            .iter()
            .find(|e| e.from == "der2" && e.to == "der1")
            .unwrap();
        assert_eq!(d.label, "redundant");
        // base -> der1 双端 aligned 且生效 → coherent
        let b = p
            .iter()
            .find(|e| e.from == "base" && e.to == "der1")
            .unwrap();
        assert_eq!(b.label, "coherent");
    }

    #[test]
    fn review_disputes() {
        let input = include_str!("../tests/fixtures/10_example.md");
        let r = parse(input);
        let rev = review(&r);
        assert_eq!(rev.disputes.len(), 1);
        assert_eq!(rev.disputes[0].from, "counter-01");
        assert_eq!(rev.disputes[0].to, "concl-01");
        assert_eq!(rev.aligned_nodes, 2);
        assert_eq!(rev.total_nodes, 5);
    }
}
