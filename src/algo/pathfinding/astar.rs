use std::collections::BinaryHeap;

use crate::algo::Measure;
use crate::scored::MinScored;
use crate::visit::{IntoEdges, VisitMap, Visitable, EdgeRef};

use super::path::{Path, PredecessorMap, CostMap, NoPredecessorMap};

/// Type safe builder style configuration of [`astar`](fn.astar.html).
///
/// Both the edge cost function and the predecessor map may be omitted.
///
/// The edge cost may only be omitted if the graph's weights implement
/// [`Measure`](../trait.Measure.html).
///
/// Omitting the predecessor map opts out of path tracking, speeding up the running time of the
/// algorithm on the other hand.
///
/// # Example
///
/// ```ignore
/// let path = Astar::new(graph)
///     .edge_cost(|e| *e.weight())
///     .estimate_cost(|e| *e.weight())
///     .cost_map(HashMap::new())
///     .predecessor_map(HashMap::new())
///     .path(start, end);
///
/// if let Some((cost, nodes)) = path.into_nodes() {
///     // there is a path
/// } else {
///     // no path
/// }
/// ```
pub struct Astar<G>
    where G: IntoEdges + Visitable
{
    graph: G,
}

impl<G> Astar<G>
    where G: IntoEdges + Visitable
{
    pub fn new(graph: G) -> Self {
        Astar { graph: graph }
    }

    pub fn edge_cost<F, K>(self, edge_cost: F) -> AstarBuilder1<G, F, K>
        where F: Fn(G::EdgeRef) -> K,
              K: Measure
    {
        AstarBuilder1 {
            graph: self.graph,
            edge_cost: edge_cost,
        }
    }
}

impl<G> Astar<G>
    where G: IntoEdges + Visitable,
          G::EdgeWeight: Measure + Copy
{
    pub fn estimate_cost<H>(self, estimate_cost: H) -> AstarBuilder2<G, fn(G::EdgeRef) -> G::EdgeWeight, H, G::EdgeWeight>
        where H: Fn(G::NodeId) -> G::EdgeWeight,
    {
        let edge_cost = edge_weight::<G> as fn(G::EdgeRef) -> G::EdgeWeight;
        self.edge_cost(edge_cost)
            .estimate_cost(estimate_cost)
    }
}

#[inline]
fn edge_weight<G>(edge: G::EdgeRef) -> G::EdgeWeight
    where G: IntoEdges,
          G::EdgeWeight: Clone
{
    edge.weight().clone()
}

pub struct AstarBuilder1<G, F, K>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          K: Measure
{
    graph: G,
    edge_cost: F,
}

impl<G, F, K> AstarBuilder1<G, F, K>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy
{
    pub fn estimate_cost<H>(self, estimate_cost: H) -> AstarBuilder2<G, F, H, K>
        where H: Fn(G::NodeId) -> K,
    {
        AstarBuilder2 {
            graph: self.graph,
            edge_cost: self.edge_cost,
            estimate_cost: estimate_cost,
        }
    }
}

pub struct AstarBuilder2<G, F, H, K>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          H: Fn(G::NodeId) -> K,
          K: Measure + Copy,
{
    graph: G,
    edge_cost: F,
    estimate_cost: H,
}

impl<G, F, H, K> AstarBuilder2<G, F, H, K>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          H: Fn(G::NodeId) -> K,
          K: Measure + Copy,
{
    pub fn cost_map<C>(self, costs: C) -> AstarBuilder3<G, F, H, K, C>
        where C: CostMap<G, Cost = K>,
    {
        AstarBuilder3 {
            graph: self.graph,
            edge_cost: self.edge_cost,
            estimate_cost: self.estimate_cost,
            costs: costs,
        }
    }
}

pub struct AstarBuilder3<G, F, H, K, C>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          H: Fn(G::NodeId) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
{
    graph: G,
    edge_cost: F,
    estimate_cost: H,
    costs: C,
}

impl<G, F, H, K, C> AstarBuilder3<G, F, H, K, C>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          H: Fn(G::NodeId) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
{
    pub fn predecessor_map<P>(self, predecessors: P) -> ConfiguredAstar<G, F, H, K, C, P>
        where P: PredecessorMap<G>,
    {
        ConfiguredAstar {
            graph: self.graph,
            edge_cost: self.edge_cost,
            estimate_cost: self.estimate_cost,
            costs: self.costs,
            predecessors: predecessors,
        }
    }

    pub fn path(self, start: G::NodeId, end: G::NodeId) -> Path<G, K, C, NoPredecessorMap> {
        self.predecessor_map(NoPredecessorMap).path(start, end)
    }

    pub fn path_with<Pred>(self, start: G::NodeId, goal_pred: Pred) -> Path<G, K, C, NoPredecessorMap>
        where Pred: Fn(G::NodeId) -> bool
    {
        self.predecessor_map(NoPredecessorMap).path_with(start, goal_pred)
    }

    pub fn path_all(self, start: G::NodeId) -> Path<G, K, C, NoPredecessorMap> {
        self.predecessor_map(NoPredecessorMap).path_all(start)
    }
}

pub struct ConfiguredAstar<G, F, H, K, C, P>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          H: Fn(G::NodeId) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
          P: PredecessorMap<G>,
{
    graph: G,
    edge_cost: F,
    estimate_cost: H,
    predecessors: P,
    costs: C,
}

impl<G, F, H, K, C, P> ConfiguredAstar<G, F, H, K, C, P>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          H: Fn(G::NodeId) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
          P: PredecessorMap<G>,
{
    pub fn path(self, start: G::NodeId, end: G::NodeId) -> Path<G, K, C, P> {
        self.path_with(start, |node| end == node)
    }

    pub fn path_with<Pred>(mut self, start: G::NodeId, goal_pred: Pred) -> Path<G, K, C, P>
        where Pred: Fn(G::NodeId) -> bool
    {
        let goal = astar_shortest_paths(self.graph,
                                           start,
                                           goal_pred,
                                           self.edge_cost,
                                           self.estimate_cost,
                                           &mut self.costs,
                                           &mut self.predecessors);

        Path::new(self.predecessors, self.costs, goal)
    }

    pub fn path_all(self, start: G::NodeId) -> Path<G, K, C, P> {
        self.path_with(start, |_: G::NodeId| false)
    }
}

pub fn astar_shortest_paths<G, K, GoalP, EdgeF, EdgeH, DMap, PMap>(graph: G,
                                                                   start: G::NodeId,
                                                                   is_goal: GoalP,
                                                                   edge_cost: EdgeF,
                                                                   estimate_cost: EdgeH,
                                                                   costs: &mut DMap,
                                                                   predecessors: &mut PMap)
                                                                   -> Option<G::NodeId>
    where G: IntoEdges + Visitable,
          GoalP: Fn(G::NodeId) -> bool,
          EdgeF: Fn(G::EdgeRef) -> K,
          EdgeH: Fn(G::NodeId) -> K,
          DMap: CostMap<G, Cost = K>,
          PMap: PredecessorMap<G>,
          K: Measure + Copy,
{
    predecessors.initialize(graph);
    costs.initialize(graph, start);

    let mut visited = graph.visit_map();
    let mut visit_next = BinaryHeap::new();

    visit_next.push(MinScored(<_>::default(), start));

    while let Some(MinScored(_, node)) = visit_next.pop() {
        if is_goal(node) {
            return Some(node);
        }

        // Don't visit the same node several times, as the first time it was visited it was using
        // the shortest available path.
        if !visited.visit(node) {
            continue;
        }

        // This lookup can be unwrapped without fear of panic since the node was necessarily scored
        // before adding him to `visit_next`.
        let node_cost = *costs.get(&node).unwrap();

        for edge in graph.edges(node) {
            let next = edge.target();
            if visited.is_visited(&next) {
                continue;
            }

            let next_cost = node_cost + edge_cost(edge);

            let replaced = costs.consider(next, next_cost);
            if replaced {
                predecessors.set(next, node);
            }

            let next_estimate_cost = next_cost + estimate_cost(next);
            visit_next.push(MinScored(next_estimate_cost, next));
        }
    }

    None
}
