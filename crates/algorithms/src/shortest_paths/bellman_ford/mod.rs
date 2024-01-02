//! An implementation of Bellman-Ford's shortest path algorithm.
mod error;
mod r#impl;
mod measure;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;

use error_stack::Result;
use petgraph_core::{
    edge::marker::{Directed, Undirected},
    node::NodeId,
    DirectedGraphStorage, Graph, GraphDirectionality, GraphStorage,
};

use self::r#impl::ShortestPathFasterImpl;
pub use self::{error::BellmanFordError, measure::BellmanFordMeasure};
use super::{
    common::{
        cost::{DefaultCost, GraphCost},
        transit::PredecessorMode,
    },
    Cost, DirectRoute, Route, ShortestDistance, ShortestPath,
};
use crate::{polyfill::IteratorExt, shortest_paths::common::connections::NodeConnections};

/// The order in which candidates are inserted into the queue.
///
/// The order in which candidates are inserted into the queue can have a significant impact on the
/// performance of the algorithm.
///
/// [`CandidateOrder::SmallFirst`] is the default, as it is an easy to implement heuristic, with
/// little overhead which can exhibit good performance improvements in practice.
///
/// # See Also
///
/// - <https://www.researchgate.net/publication/274174007_An_Improved_SPFA_Algorithm_for_Single-Source_Shortest_Path_Problem_Using_Forward_Star_Data_Structure>
// TODO: add citation about the different orders
#[derive(Default, Debug, Copy, Clone, PartialEq, Eq)]
pub enum CandidateOrder {
    /// # Naive
    ///
    /// Push the item to the back of the queue.
    Naive,

    /// # Small Label First (SSF)
    ///
    /// Checks if the current value is smaller than the next value, if that is the case, push it to
    /// the front, otherwise push it to the back.
    #[default]
    SmallFirst,

    /// # Large Label Last (LLL)
    ///
    /// Push the item to the back of the queue.
    /// Calculate the average value of the queue and as long as the next value larger than the
    /// average value, pop it from the front and push it to the back.
    LargeLast,
}

/// An implementation of Bellman-Ford's shortest path algorithm.
///
/// Bellman-Ford's algorithm is an algorithm for finding the shortest paths between a source node
/// and all reachable nodes in a graph.
/// Unlike Dijkstra's algorithm, Bellman-Ford's algorithm can be used on graphs with negative edge
/// weights, as long as the graph does not contain a negative cycle reachable from the source.
///
/// This implementation is generic over the directionality of the graph and the cost function.
///
/// Edge weights need to implement [`BellmanFordMeasure`], a trait that is automatically implemented
/// for all types that satisfy the constraints.
///
/// The internal implementation makes uses of Shortest Path Faster Algorithm (SPFA) which is a
/// variant of Bellman-Ford's algorithm, with an average time complexity of `O(|E|)` on random
/// graphs and a worst case complexity of `O(|V| * |E|)`.
pub struct BellmanFord<D, E> {
    direction: D,
    edge_cost: E,
    candidate_order: CandidateOrder,
    negative_cycle_heuristics: bool,
}

impl BellmanFord<Directed, DefaultCost> {
    /// Create a new instance of `BellmanFord` for a directed graph.
    ///
    /// If instantiated for a directed graph, [`BellmanFord`] will not implement [`ShortestPath`]
    /// and [`ShortestDistance`] on undirected graphs.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::{BellmanFord, ShortestPath};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let algorithm = BellmanFord::directed();
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = graph.insert_node("A").id();
    /// let b = graph.insert_node("B").id();
    ///
    /// graph.insert_edge(2, a, b);
    ///
    /// let path = algorithm.path_between(&graph, a, b);
    /// assert!(path.is_some());
    ///
    /// let path = algorithm.path_between(&graph, b, a);
    /// assert!(path.is_none());
    /// ```
    pub fn directed() -> Self {
        Self {
            direction: Directed,
            edge_cost: DefaultCost,
            candidate_order: CandidateOrder::default(),
            negative_cycle_heuristics: true,
        }
    }
}

impl BellmanFord<Undirected, DefaultCost> {
    /// Create a new instance of `BellmanFord` for an undirected graph.
    ///
    /// If instantiated for an undirected graph, [`BellmanFord`] will treat a directed graph as
    /// undirected.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::{BellmanFord, ShortestPath};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let algorithm = BellmanFord::undirected();
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = graph.insert_node("A").id();
    /// let b = graph.insert_node("B").id();
    ///
    /// graph.insert_edge(2, a, b);
    ///
    /// let path = algorithm.path_between(&graph, a, b);
    /// assert!(path.is_some());
    ///
    /// let path = algorithm.path_between(&graph, b, a);
    /// assert!(path.is_some());
    /// ```
    pub fn undirected() -> Self {
        Self {
            direction: Undirected,
            edge_cost: DefaultCost,
            candidate_order: CandidateOrder::default(),
            negative_cycle_heuristics: true,
        }
    }
}

impl<D, E> BellmanFord<D, E>
where
    D: GraphDirectionality,
{
    /// Set the cost function for the algorithm.
    ///
    /// By default the algorithm will use the edge weight as the cost, this function allows you to
    /// override that behaviour,
    /// transforming a previously unsupported graph weight into a supported one.
    ///
    /// For all supported functions see [`GraphCost`].
    ///
    /// # Example
    ///
    /// ```
    /// use numi::borrow::Moo;
    /// use petgraph_algorithms::shortest_paths::{BellmanFord, ShortestPath};
    /// use petgraph_core::{Edge, GraphStorage};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// fn edge_cost<S>(edge: Edge<S>) -> Moo<usize>
    /// where
    ///     S: GraphStorage,
    ///     S::EdgeWeight: AsRef<str>,
    /// {
    ///     edge.weight().as_ref().len().into()
    /// }
    ///
    /// let algorithm = BellmanFord::directed().with_edge_cost(edge_cost);
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = graph.insert_node("A").id();
    /// let b = graph.insert_node("B").id();
    ///
    /// graph.insert_edge("AB", a, b);
    ///
    /// let path = algorithm.path_between(&graph, a, b);
    /// assert!(path.is_some());
    /// ```
    pub fn with_edge_cost<S, F>(self, edge_cost: F) -> BellmanFord<D, F>
    where
        S: GraphStorage,
        F: GraphCost<S>,
    {
        BellmanFord {
            direction: self.direction,
            edge_cost,
            candidate_order: self.candidate_order,
            negative_cycle_heuristics: self.negative_cycle_heuristics,
        }
    }

    /// Set the candidate order for the algorithm.
    ///
    /// By default the algorithm uses the [`CandidateOrder::SmallFirst`] order, see
    /// [`CandidateOrder`] for the different pros and cons.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::{
    ///     bellman_ford::CandidateOrder, BellmanFord, ShortestPath,
    /// };
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let algorithm = BellmanFord::directed().with_candidate_order(CandidateOrder::LargeLast);
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = graph.insert_node("A").id();
    /// let b = graph.insert_node("B").id();
    ///
    /// graph.insert_edge(2, a, b);
    ///
    /// let path = algorithm.path_between(&graph, a, b);
    /// assert!(path.is_some());
    /// ```
    #[must_use]
    pub fn with_candidate_order(self, candidate_order: CandidateOrder) -> Self {
        Self {
            direction: self.direction,
            edge_cost: self.edge_cost,
            candidate_order,
            negative_cycle_heuristics: self.negative_cycle_heuristics,
        }
    }

    /// Enable or disable negative cycle heuristics.
    ///
    /// By default the algorithm will use negative cycle heuristics.
    /// Negative cycle heuristics help to detect negative cycles in the graph early.
    /// If the graph contains a negative cycle, the algorithm will return an error.
    ///
    /// In theory such heuristic can exhibit false-positives, but these haven't been observed in
    /// practice.
    /// The heuristic is based on the implementation of networkx and allows the algorithm to not
    /// exhibit worst case time complexity in the case of a negative cycle.
    ///
    /// It is recommended to leave this option enabled.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::{BellmanFord, ShortestPath};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let algorithm = BellmanFord::directed().with_negative_cycle_heuristics(false);
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = graph.insert_node("A").id();
    /// let b = graph.insert_node("B").id();
    ///
    /// graph.insert_edge(2, a, b);
    ///
    /// let path = algorithm.path_between(&graph, a, b);
    /// assert!(path.is_some());
    /// ```
    #[must_use]
    pub fn with_negative_cycle_heuristics(self, negative_cycle_heuristics: bool) -> Self {
        Self {
            direction: self.direction,
            edge_cost: self.edge_cost,
            candidate_order: self.candidate_order,
            negative_cycle_heuristics,
        }
    }
}

impl<S, E> ShortestPath<S> for BellmanFord<Undirected, E>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.path_from(graph, target)?;

        Ok(iter.map(Route::reverse))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>>, Self::Error> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            NodeConnections::undirected(graph.storage()),
            source,
            PredecessorMode::Record,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .map(ShortestPathFasterImpl::all)
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: NodeId,
        target: NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            NodeConnections::undirected(graph.storage()),
            source,
            PredecessorMode::Record,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .ok()?
        .between(target)
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter: Vec<_> = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()))
            .collect_reports()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestDistance<S> for BellmanFord<Undirected, E>
where
    S: GraphStorage,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.distance_from(graph, target)?;

        Ok(iter.map(DirectRoute::reverse))
    }

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            NodeConnections::undirected(graph.storage()),
            source,
            PredecessorMode::Discard,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )?;

        Ok(iter.all().map(Into::into))
    }

    fn distance_between(
        &self,
        graph: &Graph<S>,
        source: NodeId,
        target: NodeId,
    ) -> Option<Cost<Self::Cost>> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            NodeConnections::undirected(graph.storage()),
            source,
            PredecessorMode::Discard,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .ok()?
        .between(target)
        .map(Route::into_cost)
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter: Vec<_> = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()))
            .collect_reports()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestPath<S> for BellmanFord<Directed, E>
where
    S: DirectedGraphStorage,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            NodeConnections::directed(graph.storage()),
            source,
            PredecessorMode::Record,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .map(ShortestPathFasterImpl::all)
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: NodeId,
        target: NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            NodeConnections::directed(graph.storage()),
            source,
            PredecessorMode::Record,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .ok()?
        .between(target)
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter: Vec<_> = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()))
            .collect_reports()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestDistance<S> for BellmanFord<Directed, E>
where
    S: DirectedGraphStorage,
    E: GraphCost<S>,
    E::Value: BellmanFordMeasure,
{
    type Cost = E::Value;
    type Error = BellmanFordError;

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            NodeConnections::directed(graph.storage()),
            source,
            PredecessorMode::Discard,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )?;

        Ok(iter.all().map(Into::into))
    }

    fn distance_between(
        &self,
        graph: &Graph<S>,
        source: NodeId,
        target: NodeId,
    ) -> Option<Cost<Self::Cost>> {
        ShortestPathFasterImpl::new(
            graph,
            &self.edge_cost,
            NodeConnections::directed(graph.storage()),
            source,
            PredecessorMode::Discard,
            self.candidate_order,
            self.negative_cycle_heuristics,
        )
        .ok()?
        .between(target)
        .map(Route::into_cost)
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter: Vec<_> = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()))
            .collect_reports()?;

        Ok(iter.into_iter().flatten())
    }
}
