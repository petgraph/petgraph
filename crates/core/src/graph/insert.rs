use error_stack::{Result, ResultExt};

use crate::{
    edge::{EdgeId, EdgeMut},
    graph::Graph,
    id::{ArbitraryGraphId, GraphId, ManagedGraphId},
    node::{NodeId, NodeMut},
    storage::GraphStorage,
    Error,
};

impl<S> Graph<S>
where
    S: GraphStorage,
{
    /// Try to insert a node with the given attributes.
    ///
    /// This is the fallible version of [`Self::insert_node`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the edge weights, as we do insert any edges and
    /// // therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<_, ()>::new();
    /// let node = graph.try_insert_node(()).expect("unable to insert node");
    /// ```
    ///
    /// # Errors
    ///
    /// If any of the constraints that are required for the graph storage to be valid are violated.
    /// This may include things like parallel edges or self loops depending on implementation.
    ///
    /// Refer to the documentation of the underlying storage for more information.
    pub fn try_insert_node(&mut self, weight: S::NodeWeight) -> Result<NodeMut<S>, Error> {
        let id = self.storage.next_node_id();
        self.storage.insert_node(id, weight).change_context(Error)
    }

    /// Insert a node with the given attributes.
    ///
    /// Depending on the storage type, or if your code is generic over the storage type, you might
    /// want to consider using [`Self::try_insert_node`] instead.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// // we need to explicitly state the type of the edge weights, as we do insert any edges and
    /// // therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<_, ()>::new();
    /// let node = graph.insert_node(());
    /// ```
    ///
    /// # Rationale
    ///
    /// There are two functions for inserting a node (actually four).
    /// [`Self::insert_node`] and [`Self::try_insert_node`].
    ///
    /// Why is that the case?
    ///
    /// The reason is that some storage types might not be able to guarantee that the node can be
    /// inserted, but we still want to provide a convenient way to insert a node.
    /// Another reason is that this mirrors the API of other libraries and the std, such as the
    /// standard library (through [`Vec::push`], or
    /// [`std::collections::HashMap::insert`]).
    /// These may also panic and do not return a result.
    /// But(!) it is important to note that the constraints and reason why they may panic are quite
    /// different from the ones of a Graph, as the ones of a Graph can be considered a superset.
    /// Not only do they potentially fail on allocation failures, but certain constraints exist such
    /// as parallel edges or self loops, which may allow for panics more often than other
    /// libraries would.
    /// This is why we provide both functions, and you (the user) should decide which one to use.
    ///
    /// # Panics
    ///
    /// If any of the constraints that are required for the graph storage to be valid are violated.
    /// This may include things like parallel edges or self loops depending on implementation.
    ///
    /// Refer to the documentation of the underlying storage for more information.
    pub fn insert_node(&mut self, weight: S::NodeWeight) -> NodeMut<S> {
        self.try_insert_node(weight)
            .expect("Constraint violation. Try using `try_insert_node` instead.")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    /// Insert a node, where the weight is dependent on the id.
    ///
    /// This is similar to [`Self::try_insert_node`], but instead of providing the weight as a
    /// parameter, it can take into account the future id.
    ///
    /// This is the fallible version of [`Self::insert_node_with`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// struct Node {
    ///     id: NodeId,
    /// }
    ///
    /// // we need to explicitly state the type of the edge weights, as we do insert
    /// // any edges and therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<_, ()>::new();
    /// let node_id = *graph
    ///     .try_insert_node_with(|id| Node { id: *id })
    ///     .expect("unable to insert node")
    ///     .id();
    ///
    /// let node = graph.node(&node_id).expect("node must exist");
    ///
    /// assert_eq!(node.weight().id, node_id);
    /// ```
    ///
    /// # Errors
    ///
    /// If any of the constraints that are required for the graph storage to be valid are violated.
    /// This may include things like parallel edges or self loops depending on implementation.
    ///
    /// Refer to the documentation of the underlying storage for more information.
    pub fn try_insert_node_with(
        &mut self,
        weight: impl FnOnce(NodeId) -> S::NodeWeight,
    ) -> Result<NodeMut<S>, Error> {
        let id = self.storage.next_node_id();
        let weight = weight(id);

        self.storage.insert_node(id, weight).change_context(Error)
    }

    /// Insert a node, where the weight is dependent on the id.
    ///
    /// This is similar to [`Self::insert_node`], but instead of providing the weight as a
    /// parameter, it can take into account the future id.
    ///
    /// This is the infallible version of [`Self::try_insert_node_with`], which will panic instead.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// struct Node {
    ///     id: NodeId,
    /// }
    ///
    /// // we need to explicitly state the type of the edge weights, as we do insert any edges and
    /// // therefore cannot infer them.
    /// let mut graph = DiDinoGraph::<_, ()>::new();
    /// let node_id = *graph.insert_node_with(|id| Node { id: *id }).id();
    ///
    /// let node = graph.node(&node_id).expect("node must exist");
    ///
    /// assert_eq!(node.weight().id, node_id);
    /// ```
    ///
    /// # Panics
    ///
    /// If any of the constraints that are required for the graph storage to be valid are violated.
    /// This may include things like parallel edges or self loops depending on implementation.
    ///
    /// Refer to the documentation of the underlying storage for more information.
    pub fn insert_node_with(&mut self, weight: impl FnOnce(NodeId) -> S::NodeWeight) -> NodeMut<S> {
        self.try_insert_node_with(weight)
            .expect("Constraint violation. Try using `try_insert_node_with` instead.")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    /// Try to insert an edge with the given attributes.
    ///
    /// This is the fallible version of [`Self::insert_edge`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(()).id();
    /// let b = *graph.insert_node(()).id();
    ///
    /// let edge = graph
    ///     .try_insert_edge((), &a, &b)
    ///     .expect("unable to insert edge");
    /// ```
    ///
    /// # Errors
    ///
    /// If any of the constraints that are required for the graph storage to be valid are violated.
    /// This may include things like parallel edges or self loops depending on implementation.
    ///
    /// Refer to the documentation of the underlying storage for more information.
    pub fn try_insert_edge(
        &mut self,
        weight: S::EdgeWeight,
        source: NodeId,
        target: NodeId,
    ) -> Result<EdgeMut<S>, Error> {
        let id = self.storage.next_edge_id();
        self.storage
            .insert_edge(id, weight, source, target)
            .change_context(Error)
    }

    /// Insert an edge with the given attributes.
    ///
    /// Depending on the storage type, or if your code is generic over the storage type, you might
    /// want to consider using [`Self::try_insert_edge`] instead.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(()).id();
    /// let b = *graph.insert_node(()).id();
    ///
    /// let edge = graph.insert_edge((), &a, &b);
    /// ```
    ///
    /// # Rationale
    ///
    /// For the rationale behind splitting this into [`Self::insert_edge`] and
    /// [`Self::try_insert_edge`], see the documentation of [`Self::insert_node`].
    ///
    /// # Panics
    ///
    /// If any of the constraints that are required for the graph storage to be valid are violated.
    /// This may include things like parallel edges or self loops depending on implementation.
    pub fn insert_edge(
        &mut self,
        weight: S::EdgeWeight,
        source: NodeId,
        target: NodeId,
    ) -> EdgeMut<S> {
        self.try_insert_edge(weight, source, target)
            .expect("Constraint violation. Try using `try_insert_edge` instead.")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    /// Insert an edge, where the weight is dependent on the id.
    ///
    /// This is similar to [`Self::try_insert_edge`], but instead of providing the weight as a
    /// parameter, it will take into account the future id, by calling the given closure with the
    /// id.
    ///
    /// This is the fallible version of [`Self::insert_edge_with`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::{DiDinoGraph, EdgeId};
    ///
    /// struct Edge {
    ///     id: EdgeId,
    /// }
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(()).id();
    /// let b = *graph.insert_node(()).id();
    ///
    /// let edge_id = *graph
    ///     .try_insert_edge_with(|id| Edge { id: *id }, &a, &b)
    ///     .expect("unable to insert edge")
    ///     .id();
    ///
    /// let edge = graph.edge(&edge_id).expect("edge must exist");
    ///
    /// assert_eq!(edge.weight().id, edge_id);
    /// ```
    ///
    /// # Errors
    ///
    /// The same errors as [`Self::try_insert_edge`] may occur.
    pub fn try_insert_edge_with(
        &mut self,
        weight: impl FnOnce(EdgeId) -> S::EdgeWeight,
        source: NodeId,
        target: NodeId,
    ) -> Result<EdgeMut<S>, Error> {
        let id = self.storage.next_edge_id();
        let weight = weight(id);

        self.storage
            .insert_edge(id, weight, source, target)
            .change_context(Error)
    }

    /// Insert an edge, where the weight is dependent on the id.
    ///
    /// This is similar to [`Self::insert_edge`], but instead of providing the weight as a
    /// parameter, it will take into account the future id, by calling the given closure with the
    /// id.
    ///
    /// This is the infallible version of [`Self::try_insert_edge_with`], which will panic instead.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::{DiDinoGraph, EdgeId};
    ///
    /// struct Edge {
    ///     id: EdgeId,
    /// }
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let a = *graph.insert_node(()).id();
    /// let b = *graph.insert_node(()).id();
    ///
    /// let edge_id = *graph.insert_edge_with(|id| Edge { id: *id }, &a, &b).id();
    ///
    /// let edge = graph.edge(&edge_id).expect("edge must exist");
    ///
    /// assert_eq!(edge.weight().id, edge_id);
    /// ```
    ///
    /// # Panics
    ///
    /// The same panics as [`Self::insert_edge`] may occur.
    pub fn insert_edge_with(
        &mut self,
        weight: impl FnOnce(EdgeId) -> S::EdgeWeight,
        source: NodeId,
        target: NodeId,
    ) -> EdgeMut<S> {
        self.try_insert_edge_with(weight, source, target)
            .expect("Constraint violation. Try using `try_insert_edge_with` instead.")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::EdgeWeight: Clone,
{
    /// Insert an edge, or update the weight of an existing edge, if it exists.
    ///
    /// If multiple edges exist between the given nodes, all of them will be updated with the given
    ///
    /// Edges are treated as undirected, so the order of the nodes does not matter.
    ///
    /// Unlike [`Self::try_upsert_edge`], this will not return the edge, and instead return a list
    /// containing all affected edge identifiers.
    ///
    /// This is the fallible version of [`Self::upsert_edge`].
    // TODO: Example
    ///
    /// # Errors
    ///
    /// The same errors as [`Self::try_insert_edge`] may occur.
    pub fn try_upsert_edge(
        &mut self,
        weight: S::EdgeWeight,

        source: NodeId,
        target: NodeId,
    ) -> Result<Vec<EdgeId>, Error> {
        let mut affected = vec![];

        for mut edge in self.storage.edges_between_mut(source, target) {
            *edge.weight_mut() = weight.clone();
            affected.push(edge.id());
        }

        if !affected.is_empty() {
            return Ok(affected);
        }

        self.try_insert_edge(weight, source, target)
            .map(|edge| vec![edge.id()])
    }

    /// Insert an edge, or update the weight of an existing edge, if it exists.
    ///
    /// If multiple edges exist between the given nodes, all of them will be updated with the given
    /// value.
    ///
    /// This is the infallible version of [`Self::try_upsert_edge`], which will panic instead.
    ///
    /// # Panics
    ///
    /// The same panics as [`Self::try_upsert_edge`] may occur, as well as the ones from
    /// [`Self::insert_edge`].
    pub fn upsert_edge(
        &mut self,
        weight: S::EdgeWeight,

        source: NodeId,
        target: NodeId,
    ) -> Vec<EdgeId> {
        self.try_upsert_edge(weight, source, target)
            .expect("Constraint violation. Try using `try_upsert_edge` instead.")
    }

    /// Insert an edge, or update the weight of an existing edge, if it exists.
    ///
    /// If multiple edges exist between the given nodes, all of them will invoke the given closure
    /// with the edge.
    ///
    /// Unlike [`Self::try_upsert_edge_with`], this will not return the edge, and instead return a
    /// list containing all affected edge identifiers.
    ///
    /// This is the fallible version of [`Self::upsert_edge_with`].
    ///
    /// # Errors
    ///
    /// The same errors as [`Self::try_insert_edge`] may occur.
    pub fn try_upsert_edge_with(
        &mut self,
        on_update: impl FnMut(EdgeMut<S>) -> S::EdgeWeight,
        on_insert: impl FnOnce(EdgeId) -> S::EdgeWeight,

        source: NodeId,
        target: NodeId,
    ) -> Result<Vec<EdgeId>, Error> {
        let mut affected = vec![];

        for mut edge in self.storage.edges_between_mut(source, target) {
            *edge.weight_mut() = on_update(edge);
            affected.push(edge.id());
        }

        if !affected.is_empty() {
            return Ok(affected);
        }

        self.try_insert_edge_with(on_insert, source, target)
            .map(|edge| vec![edge.id()])
    }

    /// Insert an edge, or update the weight of an existing edge, if it exists.
    ///
    /// If multiple edges exist between the given nodes, all of them will invoke the given closure
    /// with the edge.
    ///
    /// This is the infallible version of [`Self::try_upsert_edge_with`], which will panic instead.
    ///
    /// # Panics
    ///
    /// The same panics as [`Self::try_upsert_edge_with`] may occur, as well as the ones from
    /// [`Self::insert_edge_with`].
    pub fn upsert_edge_with(
        &mut self,
        on_update: impl FnMut(EdgeMut<S>) -> S::EdgeWeight,
        on_insert: impl FnOnce(EdgeId) -> S::EdgeWeight,

        source: NodeId,
        target: NodeId,
    ) -> Vec<EdgeId> {
        self.try_upsert_edge_with(on_update, on_insert, source, target)
            .expect("Constraint violation. Try using `try_upsert_edge_with` instead.")
    }
}
