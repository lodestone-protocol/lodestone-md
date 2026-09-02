//! Edge pipeline (spec §6 / §7 / §8 steps 6–9): normalization, folding,
//! reference validation, soft checks, effectiveness and global cycle
//! detection.

use std::collections::HashMap;

use crate::diag::{self, Diagnostic};
use crate::nodemeta::STATUS_ALIGNED;
use crate::output::{EdgeEntry, GraphEdge};

pub const RELATION_DEPEND: &str = "depend";
pub const RELATION_DERIVE: &str = "derive";

/// A declared edge (originating from a valid node).
pub struct DeclaredEdge {
    pub from_id: String,
    pub to: String,
    pub relation: String,
}

struct Norm {
    from: String,
    to: String,
    relation: String,
    ref_ok: bool,
}

fn edge_str(from: &str, to: &str) -> String {
    format!("{} -> {}", from, to)
}

/// Returns (normalized edge-set entries, effective global-graph edges).
pub fn process(
    valid_nodes: &[(String, String)], // (id, status), document order
    declared: &[DeclaredEdge],
    diag: &mut Vec<Diagnostic>,
) -> (Vec<EdgeEntry>, Vec<GraphEdge>) {
    let status_of = |id: &str| -> Option<&str> {
        valid_nodes
            .iter()
            .find(|(i, _)| i == id)
            .map(|(_, s)| s.as_str())
    };

    // §7.2 normalization: derive transposes to depend; identical
    // (from, to, relation) triples fold into one, each extra triple reporting
    // W-REDUNDANT-EDGE.
    let mut norm: Vec<Norm> = Vec::new();
    for d in declared {
        let (from, to, relation) = if d.relation == RELATION_DERIVE {
            (d.to.clone(), d.from_id.clone(), RELATION_DEPEND.to_string())
        } else {
            (d.from_id.clone(), d.to.clone(), d.relation.clone())
        };
        if norm
            .iter()
            .any(|e| e.from == from && e.to == to && e.relation == relation)
        {
            diag.push(Diagnostic::warning(
                diag::W_REDUNDANT_EDGE,
                Some(from.clone()),
                Some(edge_str(&from, &to)),
                "duplicate edge after normalization; folded",
            ));
            continue;
        }
        // Reference validation (§8 step 7): both ends of a normalized edge
        // must resolve to valid nodes. After derive transposition the declared
        // target becomes the normalized source, so endpoint checking covers
        // the declared reference.
        let ref_ok = status_of(&from).is_some() && status_of(&to).is_some();
        norm.push(Norm {
            from,
            to,
            relation,
            ref_ok,
        });
    }

    // §8 step 7: E-REF-NOT-FOUND.
    for e in &norm {
        if !e.ref_ok {
            diag.push(Diagnostic::error(
                diag::E_REF_NOT_FOUND,
                Some(e.from.clone()),
                Some(edge_str(&e.from, &e.to)),
                "edge references a missing or invalid node",
            ));
        }
    }

    // §7.3 soft cycle detection (SHOULD): on the declared normalized edge set
    // (reference-resolvable only, regardless of alignment).
    let declared_pairs: Vec<(&str, &str)> = norm
        .iter()
        .filter(|e| e.ref_ok)
        .map(|e| (e.from.as_str(), e.to.as_str()))
        .collect();
    let declared_cycles = cycle_edge_marks(&declared_pairs);
    let declared_cycle_count = declared_cycles.iter().filter(|&&c| c).count();
    if declared_cycle_count > 0 {
        diag.push(Diagnostic::warning(
            diag::W_CYCLE_DECLARED,
            None,
            None,
            format!(
                "declared normalized edge set contains {} cyclic edge(s)",
                declared_cycle_count
            ),
        ));
    }

    // §6 constraint 3: an aligned node has a non-aligned upstream (normalized
    // depend out-edges).
    for e in &norm {
        if e.ref_ok
            && e.relation == RELATION_DEPEND
            && status_of(&e.from) == Some(STATUS_ALIGNED)
            && status_of(&e.to) != Some(STATUS_ALIGNED)
        {
            diag.push(Diagnostic::warning(
                diag::W_UPSTREAM_PENDING,
                Some(e.from.clone()),
                Some(edge_str(&e.from, &e.to)),
                "aligned node has a non-aligned upstream",
            ));
        }
    }

    // §6 constraint 4 / §7.3: deferred binding — both ends aligned and the
    // reference resolved makes an edge an effectiveness candidate; cycle
    // detection on the global graph (E-CYCLE) invalidates the whole cycle
    // edge set while the rest of the subgraph is emitted normally.
    let candidate_pairs: Vec<(usize, (&str, &str))> = norm
        .iter()
        .enumerate()
        .filter(|(_, e)| {
            e.ref_ok
                && status_of(&e.from) == Some(STATUS_ALIGNED)
                && status_of(&e.to) == Some(STATUS_ALIGNED)
        })
        .map(|(i, e)| (i, (e.from.as_str(), e.to.as_str())))
        .collect();
    let candidate_edge_marks = {
        let pairs: Vec<(&str, &str)> = candidate_pairs.iter().map(|(_, p)| *p).collect();
        cycle_edge_marks(&pairs)
    };

    let mut entries = Vec::with_capacity(norm.len());
    let mut graph_edges = Vec::new();
    for (i, e) in norm.iter().enumerate() {
        let cand_pos = candidate_pairs.iter().position(|(ci, _)| *ci == i);
        let (effective, failure) = if !e.ref_ok {
            (false, Some(diag::E_REF_NOT_FOUND.to_string()))
        } else if let Some(pos) = cand_pos {
            if candidate_edge_marks[pos] {
                diag.push(Diagnostic::error(
                    diag::E_CYCLE,
                    Some(e.from.clone()),
                    Some(edge_str(&e.from, &e.to)),
                    "edge participates in a cycle in the global graph; ineffective",
                ));
                (false, Some(diag::E_CYCLE.to_string()))
            } else {
                (true, None)
            }
        } else if e.relation == RELATION_DEPEND && status_of(&e.from) == Some(STATUS_ALIGNED) {
            // Source aligned but target not: the failure reason is recorded as
            // W-UPSTREAM-PENDING.
            (false, Some(diag::W_UPSTREAM_PENDING.to_string()))
        } else {
            // Alignment pending / nascent: merely not effective, no failure code.
            (false, None)
        };
        if effective {
            graph_edges.push(GraphEdge {
                from: e.from.clone(),
                to: e.to.clone(),
                relation: e.relation.clone(),
            });
        }
        entries.push(EdgeEntry {
            from: e.from.clone(),
            to: e.to.clone(),
            relation: e.relation.clone(),
            effective,
            failure,
        });
    }

    (entries, graph_edges)
}

/// Iterative Kosaraju SCC: returns one mark per input pair, true when that
/// edge belongs to the cycle edge set (both ends in one non-trivial SCC, or a
/// self-loop).
fn cycle_edge_marks(pairs: &[(&str, &str)]) -> Vec<bool> {
    if pairs.is_empty() {
        return vec![];
    }
    let mut index: HashMap<&str, usize> = HashMap::new();
    for &(f, t) in pairs {
        let n = index.len();
        index.entry(f).or_insert(n);
        let n = index.len();
        index.entry(t).or_insert(n);
    }
    let n = index.len();
    let edges: Vec<(usize, usize)> = pairs
        .iter()
        .map(|&(f, t)| (index[f], index[t]))
        .collect();

    let mut adj = vec![Vec::new(); n];
    let mut radj = vec![Vec::new(); n];
    for &(u, v) in &edges {
        adj[u].push(v);
        radj[v].push(u);
    }

    // Pass one: iterative post-order
    let mut order = Vec::with_capacity(n);
    let mut visited = vec![false; n];
    let mut it = vec![0usize; n];
    for s in 0..n {
        if visited[s] {
            continue;
        }
        visited[s] = true;
        let mut stack = vec![s];
        while let Some(&u) = stack.last() {
            if it[u] < adj[u].len() {
                let v = adj[u][it[u]];
                it[u] += 1;
                if !visited[v] {
                    visited[v] = true;
                    stack.push(v);
                }
            } else {
                order.push(u);
                stack.pop();
            }
        }
    }

    // Pass two: collect SCCs on the reverse graph
    let mut comp = vec![usize::MAX; n];
    let mut comp_size: Vec<usize> = Vec::new();
    for &u in order.iter().rev() {
        if comp[u] != usize::MAX {
            continue;
        }
        let c = comp_size.len();
        let mut stack = vec![u];
        comp[u] = c;
        let mut size = 0usize;
        while let Some(x) = stack.pop() {
            size += 1;
            for &y in &radj[x] {
                if comp[y] == usize::MAX {
                    comp[y] = c;
                    stack.push(y);
                }
            }
        }
        comp_size.push(size);
    }

    edges
        .iter()
        .map(|&(u, v)| comp[u] == comp[v] && (comp_size[comp[u]] > 1 || u == v))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cycle_marks() {
        // a -> b -> c -> a and b -> d (d is not on a cycle)
        let pairs = vec![("a", "b"), ("b", "c"), ("c", "a"), ("b", "d")];
        let marks = cycle_edge_marks(&pairs);
        assert_eq!(marks, vec![true, true, true, false]);
    }

    #[test]
    fn self_loop_is_cycle() {
        let pairs = vec![("a", "a")];
        assert_eq!(cycle_edge_marks(&pairs), vec![true]);
    }

    #[test]
    fn no_cycle() {
        let pairs = vec![("a", "b"), ("b", "c")];
        assert_eq!(cycle_edge_marks(&pairs), vec![false, false]);
    }
}
