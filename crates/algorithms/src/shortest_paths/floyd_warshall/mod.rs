//! An implementation of the Floyd-Warshall shortest path algorithm.
mod error;
mod r#impl;
mod matrix;
mod measure;
#[cfg(test)]
mod tests;

use core::marker::PhantomData;

use error_stack::Result;
use petgraph_core::{
    edge::marker::{Directed, Undirected},
    id::LinearGraphId,
    Graph, GraphStorage,
};

use self::r#impl::{
    init_directed_edge_distance, init_directed_edge_predecessor, init_undirected_edge_distance,
    init_undirected_edge_predecessor, FloydWarshallImpl,
};
pub use self::{error::FloydWarshallError, measure::FloydWarshallMeasure};
use super::{
    common::{
        cost::{DefaultCost, GraphCost},
        route::{DirectRoute, Route},
        transit::PredecessorMode,
    },
    Cost, ShortestDistance, ShortestPath,
};

/// An implementation of the Floyd-Warshall shortest path algorithm.
///
/// The Floyd-Warshall algorithm is an algorithm for finding shortest paths in a weighted graph with
/// positive or negative edge weights (but with no negative cycles).
/// A single execution of the algorithm will find the lengths (summed weights) of the shortest paths
/// between all pairs of vertices, though it does not return details of the paths themselves.
///
/// The implementation chosen overcomes this limitation by storing the predecessor of each node in
/// an associated matrix.
///
/// The algorithm is implemented for both directed and undirected graphs (undirected graphs are
/// simply treated as directed graphs with the same edge in both directions).
///
/// Edge weights need to implement [`FloydWarshallMeasure`], a trait that is automatically
/// implemented for all types that satisfy the constraints. The graph id for edges need to be
/// linear, refer to the documentation of the graph type used for further information if graph ids
/// implement [`LinearGraphId`].
///
/// The time complexity of the algorithm is `O(|V|^3)`, where `|V|` is the number of nodes in the
/// graph.
pub struct FloydWarshall<D, E> {
    edge_cost: E,

    direction: PhantomData<fn() -> *const D>,
}

impl FloydWarshall<Directed, DefaultCost> {
    /// Creates a new instance of the Floyd-Warshall shortest path algorithm for directed graphs.
    ///
    /// If instantiated for a directed graph, the algorithm will not implement the [`ShortestPath`]
    /// and [`ShortestDistance`] traits for undirected graphs.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::{FloydWarshall, ShortestPath};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let algorithm = FloydWarshall::directed();
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    ///
    /// graph.insert_edge(7, &a, &b);
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
            edge_cost: DefaultCost,
            direction: PhantomData,
        }
    }
}

impl FloydWarshall<Undirected, DefaultCost> {
    /// Creates a new instance of the Floyd-Warshall shortest path algorithm for undirected graphs.
    ///
    /// If used on a directed graph, the algorithm will treat the graph as undirected.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::{FloydWarshall, ShortestPath};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let algorithm = FloydWarshall::undirected();
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    ///
    /// graph.insert_edge(7, &a, &b);
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
            edge_cost: DefaultCost,
            direction: PhantomData,
        }
    }
}

impl<D, E> FloydWarshall<D, E> {
    /// Sets the cost function to use for the algorithm.
    ///
    /// By default the algorithm will use the edge weight as cost, this enables the use of a custom
    /// edge cost function, which may transform the edge weight, which is initially incompatible
    /// with the [`FloydWarshall`] implementation into one that is.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_algorithms::shortest_paths::{FloydWarshall, ShortestPath};
    /// use petgraph_core::{base::MaybeOwned, Edge, GraphStorage};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// fn edge_cost<S>(edge: Edge<S>) -> MaybeOwned<usize>
    /// where
    ///     S: GraphStorage,
    ///     S::EdgeWeight: AsRef<str>,
    /// {
    ///     edge.weight().as_ref().len().into()
    /// }
    ///
    /// let algorithm = FloydWarshall::directed().with_edge_cost(edge_cost);
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    ///
    /// graph.insert_edge("AB", &a, &b);
    ///
    /// let path = algorithm.path_between(&graph, &a, &b);
    /// assert!(path.is_some());
    ///
    /// let path = algorithm.path_between(&graph, &b, &a);
    /// assert!(path.is_none());
    /// ```
    pub fn with_edge_cost<S, F>(self, edge_cost: F) -> FloydWarshall<D, F>
    where
        S: GraphStorage,
        F: GraphCost<S>,
    {
        FloydWarshall {
            edge_cost,
            direction: self.direction,
        }
    }
}

impl<S, E> ShortestPath<S> for FloydWarshall<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    E::Value: FloydWarshallMeasure,
{
    type Cost = E::Value;
    type Error = FloydWarshallError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(move |_, target_node| target_node.id() == target))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(move |source_node, _| source_node.id() == source))
    }

    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        let r#impl = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )
        .ok()?;

        r#impl.pick(source, target)
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(|_, _| true))
    }
}

impl<S, E> ShortestDistance<S> for FloydWarshall<Undirected, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    E::Value: FloydWarshallMeasure,
{
    type Cost = E::Value;
    type Error = FloydWarshallError;

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )?;

        Ok(iter
            .filter(move |_, target_node| target_node.id() == target)
            .map(From::from))
    }

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )?;

        Ok(iter
            .filter(move |source_node, _| source_node.id() == source)
            .map(From::from))
    }

    fn distance_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )
        .ok()?;

        iter.pick(source, target).map(Route::into_cost)
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_undirected_edge_distance::<S, E>,
            init_undirected_edge_predecessor::<S>,
        )?;

        Ok(iter.filter(|_, _| true).map(From::from))
    }
}

impl<S, E> ShortestPath<S> for FloydWarshall<Directed, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    E::Value: FloydWarshallMeasure,
{
    type Cost = E::Value;
    type Error = FloydWarshallError;

    fn path_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(move |_, target_node| target_node.id() == target))
    }

    fn path_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(move |source_node, _| source_node.id() == source))
    }

    // TODO: benchmark if the filter has an actual impact on performance
    fn path_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Route<'graph, S, Self::Cost>> {
        let r#impl = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )
        .ok()?;

        r#impl.pick(source, target)
    }

    fn every_path<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = Route<'graph, S, Self::Cost>> + 'this, Self::Error> {
        FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Record,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )
        .map(move |r#impl| r#impl.filter(|_, _| true))
    }
}

impl<S, E> ShortestDistance<S> for FloydWarshall<Directed, E>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Clone + Send + Sync + 'static,
    E: GraphCost<S>,
    E::Value: FloydWarshallMeasure,
{
    type Cost = E::Value;
    type Error = FloydWarshallError;

    fn distance_to<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        target: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )?;

        Ok(iter
            .filter(move |_, target_node| target_node.id() == target)
            .map(From::from))
    }

    fn distance_from<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )?;

        Ok(iter
            .filter(move |source_node, _| source_node.id() == source)
            .map(From::from))
    }

    fn distance_between<'graph>(
        &self,
        graph: &'graph Graph<S>,
        source: &'graph S::NodeId,
        target: &'graph S::NodeId,
    ) -> Option<Cost<Self::Cost>> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )
        .ok()?;

        iter.pick(source, target).map(Route::into_cost)
    }

    fn every_distance<'graph: 'this, 'this>(
        &'this self,
        graph: &'graph Graph<S>,
    ) -> Result<impl Iterator<Item = DirectRoute<'graph, S, Self::Cost>> + 'this, Self::Error> {
        let iter = FloydWarshallImpl::new(
            graph,
            &self.edge_cost,
            PredecessorMode::Discard,
            init_directed_edge_distance::<S, E>,
            init_directed_edge_predecessor::<S>,
        )?;

        Ok(iter.filter(|_, _| true).map(From::from))
    }
}
