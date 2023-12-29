//! An implementation of Dijkstra's shortest path algorithm.
mod error;
mod iter;
mod measure;
#[cfg(test)]
mod tests;

use alloc::vec::Vec;
use core::marker::PhantomData;

use error_stack::Result;
use petgraph_core::{
    edge::marker::{Directed, Undirected},
    id::{AttributeGraphId, FlaggableGraphId},
    DirectedGraphStorage, Graph, GraphDirectionality, GraphStorage, Node,
};

use self::iter::DijkstraIter;
pub use self::{error::DijkstraError, measure::DijkstraMeasure};
use super::{
    common::{
        connections::outgoing_connections,
        cost::{DefaultCost, GraphCost},
        route::{DirectRoute, Route},
        transit::PredecessorMode,
    },
    ShortestDistance, ShortestPath,
};
use crate::polyfill::IteratorExt;

/// An implementation of Dijkstra's shortest path algorithm.
///
/// Dijkstra's algorithm is an algorithm for finding the shortest paths between a source node and
/// all reachable nodes in a graph.
/// A limitation of Dijkstra's algorithm is that it does not work for graphs with negative edge
/// weights.
///
/// This implementation is generic over the directionality of the graph and the cost function.
///
/// Edge weights need to implement [`DijkstraMeasure`], a trait that is automatically implemented
/// for all types that satisfy the constraints.
///
/// This implementation makes use of a binary heap, giving a time complexity of `O(|E| + |V| log
/// |E|/|V| log |V|)`
pub struct Dijkstra<D, E>
where
    D: GraphDirectionality,
{
    edge_cost: E,

    direction: PhantomData<fn() -> *const D>,
}

impl Dijkstra<Directed, DefaultCost> {
    /// Create a new instance of `Dijkstra` for a directed graph.
    ///
    /// If instantiated for a directed graph, [`Dijkstra`] will not implement [`ShortestPath`] and
    /// [`ShortestDistance`] on undirected graphs.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::{Dijkstra, ShortestPath};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let algorithm = Dijkstra::directed();
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    ///
    /// graph.insert_edge(2, &a, &b);
    ///
    /// let path = algorithm.path_between(&graph, &a, &b);
    /// assert!(path.is_some());
    ///
    /// let path = algorithm.path_between(&graph, &b, &a);
    /// assert!(path.is_none());
    /// ```
    #[must_use]
    pub fn directed() -> Self {
        Self {
            direction: PhantomData,
            edge_cost: DefaultCost,
        }
    }
}

impl Dijkstra<Undirected, DefaultCost> {
    /// Create a new instance of `Dijkstra` for an undirected graph.
    ///
    /// If instantiated for an undirected graph, [`Dijkstra`] will treat a directed graph as an
    /// undirected graph.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::{Dijkstra, ShortestPath};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let algorithm = Dijkstra::undirected();
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    ///
    /// graph.insert_edge(2, &a, &b);
    ///
    /// let path = algorithm.path_between(&graph, &a, &b);
    /// assert!(path.is_some());
    ///
    /// let path = algorithm.path_between(&graph, &b, &a);
    /// assert!(path.is_some());
    /// ```
    #[must_use]
    pub fn undirected() -> Self {
        Self {
            direction: PhantomData,
            edge_cost: DefaultCost,
        }
    }
}

impl<D, E> Dijkstra<D, E>
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
    /// use petgraph_algorithms::shortest_paths::{Dijkstra, ShortestPath};
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
    /// let algorithm = Dijkstra::directed().with_edge_cost(edge_cost);
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    ///
    /// graph.insert_edge("AB", &a, &b);
    ///
    /// let path = algorithm.path_between(&graph, &a, &b);
    /// assert!(path.is_some());
    /// ```
    pub fn with_edge_cost<S, F>(self, edge_cost: F) -> Dijkstra<D, F>
    where
        S: GraphStorage,
        F: GraphCost<S>,
    {
        Dijkstra {
            direction: PhantomData,
            edge_cost,
        }
    }
}

impl<S, E> ShortestPath<S> for Dijkstra<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: FlaggableGraphId<S> + AttributeGraphId<S>,
    E: GraphCost<S>,
    E::Value: DijkstraMeasure,
{
    type Cost = E::Value;
    type Error = DijkstraError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.path_from(graph, target)?;

        Ok(iter.map(Route::reverse))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        DijkstraIter::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Record,
        )
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestPath<S> for Dijkstra<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: FlaggableGraphId<S> + AttributeGraphId<S>,
    E: GraphCost<S>,
    E::Value: DijkstraMeasure,
{
    type Cost = E::Value;
    type Error = DijkstraError;

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        DijkstraIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Record,
        )
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.path_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestDistance<S> for Dijkstra<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: FlaggableGraphId<S> + AttributeGraphId<S>,
    E: GraphCost<S>,
    E::Value: DijkstraMeasure,
{
    type Cost = E::Value;
    type Error = DijkstraError;

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = self.distance_from(graph, target)?;

        Ok(iter.map(DirectRoute::reverse))
    }

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = DijkstraIter::new(
            graph,
            &self.edge_cost,
            Node::<'graph, S>::connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Discard,
        )?;

        Ok(iter.map(From::from))
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}

impl<S, E> ShortestDistance<S> for Dijkstra<Directed, E>
where
    S: DirectedGraphStorage,
    S::NodeId: FlaggableGraphId<S> + AttributeGraphId<S>,
    E: GraphCost<S>,
    E::Value: DijkstraMeasure,
{
    type Cost = E::Value;
    type Error = DijkstraError;

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph <S as GraphStorage>::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>>, Self::Error> {
        let iter = DijkstraIter::new(
            graph,
            &self.edge_cost,
            outgoing_connections as fn(&Node<'graph, S>) -> _,
            source,
            PredecessorMode::Discard,
        )?;

        Ok(iter.map(From::from))
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = graph
            .nodes()
            .map(move |node| self.distance_from(graph, node.id()))
            .collect_reports::<Vec<_>>()?;

        Ok(iter.into_iter().flatten())
    }
}
