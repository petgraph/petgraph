use std::collections::{
    HashMap,
    BinaryHeap,
};
use std::collections::hash_map::Hasher;
use std::collections::hash_map::Entry::{
    Occupied,
    Vacant,
};

use std::default::Default;
use std::hash::Hash;
use std::ops::{
    Add,
};

use super::MinScored;
use super::visit::{
    Visitable,
    VisitMap,
};

/// Dijkstra's shortest path algorithm.
pub fn dijkstra<'a, G, N, K, F, Edges>(graph: &'a G,
                                       start: N,
                                       goal: Option<N>,
                                       mut edges: F) -> HashMap<N, K> where
    G: Visitable<NodeId=N>,
    N: Clone + Eq + Hash<Hasher>,
    K: Default + Add<Output=K> + Copy + PartialOrd,
    F: FnMut(&'a G, N) -> Edges,
    Edges: Iterator<Item=(N, K)>,
    <G as Visitable>::Map: VisitMap<N>,
{
    let mut visited = graph.visit_map();
    let mut scores = HashMap::new();
    let mut predecessor = HashMap::new();
    let mut visit_next = BinaryHeap::new();
    let zero_score: K = Default::default();
    scores.insert(start.clone(), zero_score);
    visit_next.push(MinScored(zero_score, start));
    while let Some(MinScored(node_score, node)) = visit_next.pop() {
        if visited.is_visited(&node) {
            continue
        }
        for (next, edge) in edges(graph, node.clone()) {
            if visited.is_visited(&next) {
                continue
            }
            let mut next_score = node_score + edge;
            match scores.entry(next.clone()) {
                Occupied(ent) => if next_score < *ent.get() {
                    *ent.into_mut() = next_score;
                    predecessor.insert(next.clone(), node.clone());
                } else {
                    next_score = *ent.get();
                },
                Vacant(ent) => {
                    ent.insert(next_score);
                    predecessor.insert(next.clone(), node.clone());
                }
            }
            visit_next.push(MinScored(next_score, next));
        }
        if goal.as_ref() == Some(&node) {
            break
        }
        visited.visit(node);
    }
    scores
}
