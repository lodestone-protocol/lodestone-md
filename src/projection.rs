//! View projection (spec appendix A, non-normative): edge-state labels and
//! the human review surface.
//!
//! A projection is a read-only view deterministically computed from (status,
//! normalized edge set, diagnostics). The protocol specifies facts, not
//! presentation.

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

/// Appendix A condition table. Priority of the mutually exclusive
/// classification:
/// dangling (E-REF-NOT-FOUND) > cyclic (cycle edge set) > redundant (folded
/// with a warning) > coherent (effective) > pending (exactly one end aligned)
/// > nascent (neither end aligned).
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
    // Edges with unresolvable references already returned as dangling above;
    // here both endpoints resolve.
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

/// Projection of every normalized edge (document order).
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

/// Human review surface (spec §1 consumer role: inspect alignment state,
/// warnings and dispute domains).
#[derive(Serialize, Clone, Debug, Default)]
pub struct Review {
    /// Declared-aligned nodes / total nodes
    pub aligned_nodes: usize,
    pub total_nodes: usize,
    /// Dispute domains: every refute normalized edge (endpoint id + status)
    pub disputes: Vec<Dispute>,
    /// Dangling edges (unresolvable references)
    pub dangling: Vec<String>,
    /// Cycle edges
    pub cyclic: Vec<String>,
    /// Alignment debt (W-UPSTREAM-PENDING)
    pub upstream_pending: Vec<String>,
    /// Remaining warnings (W-*): code → occurrences
    pub other_warnings: Vec<(String, usize)>,
}

#[derive(Serialize, Clone, Debug)]
pub struct Dispute {
    pub from: String,
    pub from_status: String,
    pub to: String,
    pub to_status: String,
}

/// Builds the dispute-domain and review summary. Input must come from one
/// single parse pass (single pipeline).
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
        let input = include_str!("../vendor/lodestone-spec/fixtures/10_example.md");
        let r = parse(input);
        let p = project(&r);
        let get = |from: &str, to: &str| {
            p.iter()
                .find(|e| e.from == from && e.to == to)
                .map(|e| e.label)
                .unwrap()
        };
        // concl-01 (converged) -> exp-01 (aligned): exactly one aligned → pending
        assert_eq!(get("concl-01", "exp-01"), "pending");
        // counter-01 (draft) -> concl-01 (converged): neither aligned → nascent
        assert_eq!(get("counter-01", "concl-01"), "nascent");
        // plan-01 -> old-note: unresolvable reference → dangling
        assert_eq!(get("plan-01", "old-note"), "dangling");
    }

    #[test]
    fn coherent_and_cyclic() {
        let input = include_str!("../vendor/lodestone-spec/fixtures/cycle_global.md");
        let r = parse(input);
        let p = project(&r);
        let coherent = p.iter().filter(|e| e.label == "coherent").count();
        let cyclic = p.iter().filter(|e| e.label == "cyclic").count();
        assert_eq!(coherent, 1); // cc -> cd
        assert_eq!(cyclic, 3);

        let rev = review(&r);
        assert_eq!(rev.cyclic.len(), 3);
        assert_eq!(rev.aligned_nodes, 4);
    }

    #[test]
    fn redundant_label() {
        let input = include_str!("../vendor/lodestone-spec/fixtures/edges_derive_fold.md");
        let r = parse(input);
        let p = project(&r);
        // The surviving folded edge carries W-REDUNDANT-EDGE → redundant
        let d = p
            .iter()
            .find(|e| e.from == "der2" && e.to == "der1")
            .unwrap();
        assert_eq!(d.label, "redundant");
        // base -> der1: both aligned and effective → coherent
        let b = p
            .iter()
            .find(|e| e.from == "base" && e.to == "der1")
            .unwrap();
        assert_eq!(b.label, "coherent");
    }

    #[test]
    fn review_disputes() {
        let input = include_str!("../vendor/lodestone-spec/fixtures/10_example.md");
        let r = parse(input);
        let rev = review(&r);
        assert_eq!(rev.disputes.len(), 1);
        assert_eq!(rev.disputes[0].from, "counter-01");
        assert_eq!(rev.disputes[0].to, "concl-01");
        assert_eq!(rev.aligned_nodes, 2);
        assert_eq!(rev.total_nodes, 5);
    }
}
