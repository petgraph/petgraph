use crate::algo::Measure;
use crate::visit::{IntoEdges, Visitable, EdgeRef};

use super::astar::astar_shortest_paths;
use super::path::{Path, PredecessorMap, CostMap, NoPredecessorMap};

pub struct Dijkstra<G>
    where G: IntoEdges + Visitable
{
    graph: G,
}

impl<G> Dijkstra<G>
    where G: IntoEdges + Visitable
{
    pub fn new(graph: G) -> Self {
        Dijkstra { graph: graph }
    }

    pub fn edge_cost<F, K>(self, edge_cost: F) -> DijkstraBuilder1<G, F, K>
        where F: Fn(G::EdgeRef) -> K,
              K: Measure
    {
        DijkstraBuilder1 {
            graph: self.graph,
            edge_cost: edge_cost,
        }
    }
}

impl<G> Dijkstra<G>
    where G: IntoEdges + Visitable,
          G::EdgeWeight: Measure + Copy
{
    pub fn cost_map<C>(self, costs: C) -> DijkstraBuilder2<G, fn(G::EdgeRef) -> G::EdgeWeight, G::EdgeWeight, C>
        where C: CostMap<G, Cost = G::EdgeWeight>,
    {
        let edge_cost = edge_weight::<G> as fn(G::EdgeRef) -> G::EdgeWeight;
        self.edge_cost(edge_cost).cost_map(costs)
    }
}

#[inline]
fn edge_weight<G>(edge: G::EdgeRef) -> G::EdgeWeight
    where G: IntoEdges,
          G::EdgeWeight: Clone
{
    edge.weight().clone()
}

pub struct DijkstraBuilder1<G, F, K>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          K: Measure
{
    graph: G,
    edge_cost: F,
}

impl<G, F, K> DijkstraBuilder1<G, F, K>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy
{
    pub fn cost_map<C>(self, costs: C) -> DijkstraBuilder2<G, F, K, C>
        where C: CostMap<G, Cost = K>,
    {
        DijkstraBuilder2 {
            graph: self.graph,
            edge_cost: self.edge_cost,
            costs: costs,
        }
    }
}

pub struct DijkstraBuilder2<G, F, K, C>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
{
    graph: G,
    edge_cost: F,
    costs: C,
}

impl<G, F, K, C> DijkstraBuilder2<G, F, K, C>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
{
    pub fn predecessor_map<P>(self, predecessors: P) -> ConfiguredDijkstra<G, F, K, C, P>
        where P: PredecessorMap<G>,
    {
        ConfiguredDijkstra {
            graph: self.graph,
            edge_cost: self.edge_cost,
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

pub struct ConfiguredDijkstra<G, F, K, C, P>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
          P: PredecessorMap<G>,
{
    graph: G,
    edge_cost: F,
    predecessors: P,
    costs: C,
}

impl<G, F, K, C, P> ConfiguredDijkstra<G, F, K, C, P>
    where G: IntoEdges + Visitable,
          F: Fn(G::EdgeRef) -> K,
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
        let goal = dijkstra_shortest_paths(self.graph,
                                           start,
                                           goal_pred,
                                           self.edge_cost,
                                           &mut self.costs,
                                           &mut self.predecessors);

        Path::new(self.predecessors, self.costs, goal)
    }

    pub fn path_all(self, start: G::NodeId) -> Path<G, K, C, P> {
        self.path_with(start, |_: G::NodeId| false)
    }
}

pub fn dijkstra_shortest_paths<G, K, GoalP, EdgeF, DMap, PMap>(graph: G,
                                                               start: G::NodeId,
                                                               is_goal: GoalP,
                                                               edge_cost: EdgeF,
                                                               costs: &mut DMap,
                                                               predecessors: &mut PMap)
                                                               -> Option<G::NodeId>
    where G: IntoEdges + Visitable,
          GoalP: Fn(G::NodeId) -> bool,
          EdgeF: Fn(G::EdgeRef) -> K,
          DMap: CostMap<G, Cost = K>,
          PMap: PredecessorMap<G>,
          K: Measure + Copy
{
    astar_shortest_paths(graph,
                         start,
                         is_goal,
                         edge_cost,
                         |_| K::default(),
                         costs,
                         predecessors)
}
