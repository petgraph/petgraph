use std::collections::{
    HashMap,
    BinaryHeap,
};
use std::collections::hash_map::Entry::{
    Occupied,
    Vacant,
};

use std::default::Default;
use std::hash::Hash;
use std::ops::{
    Add,
};

use scored::MinScored;
use super::visit::{
    Visitable,
    VisitMap,
};

/// Dijkstra's shortest path algorithm.
///
/// Compute the length of the shortest path from `start` to every reachable
/// node.
///
/// The graph should be `Visitable`, and `edges` a closure that maps
/// a node identifier to an iterator of `(n, k)` pairs where `n` is an adjacent
/// node and `k` the edge weight.
///
/// If `goal` is not `None`, then the algorithm terminates once the `goal` node's
/// cost is calculated.
///
/// Returns a `HashMap` that maps `NodeId` to path cost.
pub fn dijkstra<'a, G: Visitable, K, F, Edges>(graph: &'a G,
                                               start: G::NodeId,
                                               goal: Option<G::NodeId>,
                                               mut edges: F) -> HashMap<G::NodeId, K> where
    G::NodeId: Eq + Hash,
    K: Default + Add<Output=K> + Copy + PartialOrd,
    F: FnMut(&'a G, G::NodeId) -> Edges,
    Edges: Iterator<Item=(G::NodeId, K)>,
{
    let mut visited = graph.visit_map();
    let mut scores = HashMap::new();
    //let mut predecessor = HashMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score: K = Default::default();
    scores.insert(start.clone(), zero_score);
    visit_next.push(MinScored(zero_score, start));
    while let Some(MinScored(node_score, node)) = visit_next.pop() {
        if visited.is_visited(&node) {
            continue
        }
        if goal.as_ref() == Some(&node) {
            break
        }
        for (next, edge) in edges(graph, node.clone()) {
            if visited.is_visited(&next) {
                continue
            }
            let mut next_score = node_score + edge;
            match scores.entry(next.clone()) {
                Occupied(ent) => if next_score < *ent.get() {
                    *ent.into_mut() = next_score;
                    //predecessor.insert(next.clone(), node.clone());
                } else {
                    next_score = *ent.get();
                },
                Vacant(ent) => {
                    ent.insert(next_score);
                    //predecessor.insert(next.clone(), node.clone());
                }
            }
            visit_next.push(MinScored(next_score, next));
        }
        visited.visit(node);
    }
    scores
}
