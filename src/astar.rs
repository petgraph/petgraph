use std::collections::{
    HashMap,
    BinaryHeap,
};
use std::collections::hash_map::Entry::{
    Occupied,
    Vacant,
};

use std::hash::Hash;

use scored::MinScored;
use super::visit::{
    Visitable,
    VisitMap,
    IntoEdges,
    EdgeRef,
};

use algo::Measure;

/// [Generic] A* shortest path algorithm.
///
/// Compute the length of the shortest path from `start` to the goal node.
///
/// The graph should be `Visitable` and implement `IntoEdges`. The function `edge_cost` should
/// return the cost for a particular edge, which is used to compute path costs.  The function
/// `estimate_cost` should return the estimated cost to the goal for a particular node, also used
/// to compute path costs. Edge costs must be non-negative.
///
/// Returns a `HashMap` that maps `NodeId` to path cost.
pub fn astar<G, F, H, K>(graph: G, start: G::NodeId, goal: Option<G::NodeId>,
                         mut edge_cost: F, mut estimate_cost: H)
    -> HashMap<G::NodeId, K>
    where G: IntoEdges + Visitable,
          G::NodeId: Eq + Hash,
          F: FnMut(G::EdgeRef) -> K,
          H: FnMut(G::NodeId) -> K,
          K: Measure + Copy,
{
    let mut visited = graph.visit_map();
    let mut visit_next = BinaryHeap::new();
    let mut scores = HashMap::new();

    let zero_score = K::default();
    scores.insert(start, zero_score);
    visit_next.push(MinScored(estimate_cost(start), start));

    while let Some(MinScored(_, node)) = visit_next.pop() {
        if visited.is_visited(&node) {
            continue
        }

        if goal.as_ref() == Some(&node) {
            break
        }

        for edge in graph.edges(node) {
            let next = edge.target();
            if visited.is_visited(&next) {
                continue
            }

            let node_score = *scores.get(&node).unwrap();
            let mut next_score = node_score + edge_cost(edge);

            // TODO: understand this section
            match scores.entry(next) {
                Occupied(ent) => if next_score < *ent.get() {
                    *ent.into_mut() = next_score;
                } else {
                    next_score = *ent.get();
                },
                Vacant(ent) => {
                    ent.insert(next_score);
                }
            }

            let next_estimate_score = next_score + estimate_cost(next);
            visit_next.push(MinScored(next_estimate_score, next));
        }
        visited.visit(node);
    }

    scores
}
