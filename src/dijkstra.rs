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

/// [Generic] Dijkstra's shortest path algorithm.
///
/// Compute the length of the shortest path from `start` to every reachable
/// node, as well as the shortest path tree.
///
/// The graph should be `Visitable` and implement `IntoEdges`. The function
/// `edge_cost` should return the cost for a particular edge, which is used
/// to compute path costs. Edge costs must be non-negative.
///
/// If `goal` is not `None`, then the algorithm terminates once the `goal` node's
/// cost is calculated.
///
/// Returns a tuple containing:
///
/// * `HashMap` that maps `NodeId` to path cost. The shortest path weight is the sum of the edge
/// weights along the shortest path.
/// * `HashMap` that maps each `NodeId` to its predecessor. The predecessor map records the edges
/// in the shortest path tree, the tree computed by the traversal of the graph.
/// Upon completion of the algorithm, the edges *(p[u],u)* for all *u* in *V* are in the tree.
/// The shortest path from vertex *s* to each vertex *v* in the graph consists of the vertices
/// *v, p[v], p[p[v]],* and so on until *s* is reached, in reverse order.
/// The tree is not guaranteed to be a minimum spanning tree. If *p[u]* does not exist, it is
/// either because u has not been visited as the algorithm
/// stopped or that it is not reachable from the source.
pub fn dijkstra<G, F, K>(graph: G, start: G::NodeId, goal: Option<G::NodeId>,
                         mut edge_cost: F)
    -> (HashMap<G::NodeId, K>, HashMap<G::NodeId,G::NodeId>)
    where G: IntoEdges + Visitable,
          G::NodeId: Eq + Hash,
          F: FnMut(G::EdgeRef) -> K,
          K: Measure + Copy,
{
    let mut visited = graph.visit_map();
    let mut scores = HashMap::new(); // distance map, holding the scores
    let mut predecessor = HashMap::new(); // predecessor map, holding the shortest path tree
    let mut visit_next = BinaryHeap::new();
    let zero_score = K::default();
    scores.insert(start, zero_score);
    visit_next.push(MinScored(zero_score, start));
    // we iterate over all nodes of the graph
    while let Some(MinScored(node_score, node)) = visit_next.pop() {
        // if the node has been visited (and then scored), we go the next node to visit
        if visited.is_visited(&node) {
            continue
        }
        // if the optional goal node has been reached, we terminate the algorithm
        if goal.as_ref() == Some(&node) {
            break
        }
        // for each node we explore connected nodes
        for edge in graph.edges(node) {
            let next = edge.target();
            // if the target node has been visited, we continue to explore other nodes
            if visited.is_visited(&next) {
                continue
            }
            let mut next_score = node_score + edge_cost(edge); // compute the total weight
            match scores.entry(next) {
                // if there is an existing entry, we see if we need to update it
                Occupied(ent) => if next_score < *ent.get() {
                    // the computed weight is lower than the previous one, we update it
                    // and store the predecessor to record the shortest path tree
                    *ent.into_mut() = next_score;
                    predecessor.insert(next.clone(), node.clone());
                } else {
                    // otherwise we keep the previously computed total weight
                    // and no need to update the predecessor map
                    next_score = *ent.get();
                },
                // if there is no entry, we create one
                Vacant(ent) => {
                    ent.insert(next_score);
                    predecessor.insert(next.clone(), node.clone());
                }
            }
            visit_next.push(MinScored(next_score, next));
        }
        visited.visit(node);
    }
    (scores, predecessor)
}
