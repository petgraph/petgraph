use crate::algo::Measure;
use crate::visit::{IntoEdges, NodeCount, IntoNodeIdentifiers, EdgeRef};

use super::path::{Path, PredecessorMap, CostMap};

pub struct BellmanFord<G>
    where G: IntoEdges + NodeCount + IntoNodeIdentifiers
{
    graph: G,
}

impl<G> BellmanFord<G>
    where G: IntoEdges + NodeCount + IntoNodeIdentifiers
{
    pub fn new(graph: G) -> Self {
        BellmanFord { graph: graph }
    }

    pub fn edge_cost<F, K>(self, edge_cost: F) -> BellmanFordBuilder1<G, F, K>
        where F: Fn(G::EdgeRef) -> K,
              K: Measure
    {
        BellmanFordBuilder1 {
            graph: self.graph,
            edge_cost: edge_cost,
        }
    }
}

impl<G> BellmanFord<G>
    where G: IntoEdges + NodeCount + IntoNodeIdentifiers,
          G::EdgeWeight: Measure + Copy
{
    pub fn cost_map<C>(self, costs: C) -> BellmanFordBuilder2<G, fn(G::EdgeRef) -> G::EdgeWeight, G::EdgeWeight, C>
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

pub struct BellmanFordBuilder1<G, F, K>
    where G: IntoEdges + NodeCount + IntoNodeIdentifiers,
          F: Fn(G::EdgeRef) -> K,
          K: Measure
{
    graph: G,
    edge_cost: F,
}

impl<G, F, K> BellmanFordBuilder1<G, F, K>
    where G: IntoEdges + NodeCount + IntoNodeIdentifiers,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy
{
    pub fn cost_map<C>(self, costs: C) -> BellmanFordBuilder2<G, F, K, C>
        where C: CostMap<G, Cost = K>,
    {
        BellmanFordBuilder2 {
            graph: self.graph,
            edge_cost: self.edge_cost,
            costs: costs,
        }
    }
}

pub struct BellmanFordBuilder2<G, F, K, C>
    where G: IntoEdges + NodeCount + IntoNodeIdentifiers,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
{
    graph: G,
    edge_cost: F,
    costs: C,
}

impl<G, F, K, C> BellmanFordBuilder2<G, F, K, C>
    where G: IntoEdges + NodeCount + IntoNodeIdentifiers,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
{
    pub fn predecessor_map<P>(self, predecessors: P) -> ConfiguredBellmanFord<G, F, K, C, P>
        where P: PredecessorMap<G>,
    {
        ConfiguredBellmanFord {
            graph: self.graph,
            edge_cost: self.edge_cost,
            costs: self.costs,
            predecessors: predecessors,
        }
    }

    pub fn path_all(self, start: G::NodeId) -> Result<Path<G, K, C, ()>, NegativeCycle> {
        self.predecessor_map(()).path_all(start)
    }
}

pub struct ConfiguredBellmanFord<G, F, K, C, P>
    where G: IntoEdges + NodeCount + IntoNodeIdentifiers,
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

impl<G, F, K, C, P> ConfiguredBellmanFord<G, F, K, C, P>
    where G: IntoEdges + NodeCount + IntoNodeIdentifiers,
          F: Fn(G::EdgeRef) -> K,
          K: Measure + Copy,
          C: CostMap<G, Cost = K>,
          P: PredecessorMap<G>,
{
    pub fn path_all(self, start: G::NodeId) -> Result<Path<G, K, C, P>, NegativeCycle> {
        let mut costs = self.costs;
        let mut predecessors = self.predecessors;

        let result = bellman_ford_shortest_paths(self.graph,
                                                 start,
                                                 self.edge_cost,
                                                 &mut costs,
                                                 &mut predecessors);

        result.map(|_| Path::new(predecessors, costs, None))
    }
}

/// An algorithm error: a cycle of negative weights was found in the graph.
#[derive(Clone, Debug, PartialEq)]
pub struct NegativeCycle(());

fn bellman_ford_shortest_paths<G, K, EdgeF, DMap, PMap>(graph: G,
                                                        start: G::NodeId,
                                                        edge_cost: EdgeF,
                                                        costs: &mut DMap,
                                                        predecessors: &mut PMap)
                                                        -> Result<(), NegativeCycle>
    where G: NodeCount + IntoNodeIdentifiers + IntoEdges,
          EdgeF: Fn(G::EdgeRef) -> K,
          DMap: CostMap<G, Cost = K>,
          PMap: PredecessorMap<G>,
          K: Measure + Copy
{
    predecessors.initialize(graph);
    costs.initialize(graph, start);

    // scan up to |V| - 1 times.
    for _ in 1..graph.node_count() {
        let mut did_update = false;
        for i in graph.node_identifiers() {
            for edge in graph.edges(i) {
                let node = edge.source();
                let next = edge.target();

                let node_cost = costs[&node];
                let next_cost = node_cost + edge_cost(edge);
                let replaced = costs.consider(next, next_cost);
                if replaced {
                    predecessors.set(next, node);
                    did_update = true;
                }
            }
        }

        if !did_update {
            break;
        }
    }

    // check for negative weight cycle
    for node in graph.node_identifiers() {
        let node_cost = costs[&node];
        for edge in graph.edges(node) {
            let next = edge.target();
            if node_cost + edge_cost(edge) < costs[&next] {
                return Err(NegativeCycle(()));
            }
        }
    }

    Ok(())
}
