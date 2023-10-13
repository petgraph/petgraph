use error_stack::Result;

use crate::{
    attributes::{Attributes, NoValue},
    edge::EdgeMut,
    graph::Graph,
    id::{ArbitraryGraphId, GraphId, ManagedGraphId},
    node::NodeMut,
    storage::GraphStorage,
};

// TODO: own error

impl<S> Graph<S>
where
    S: GraphStorage,
{
    /// Try to insert a node with the given attributes.
    ///
    /// This is the fallible version of [`insert_node`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    /// let node = graph.try_insert_node(()).expect("unable to insert node");
    /// ```
    ///
    /// # Errors
    ///
    /// If any of the constraints that are required for the graph storage to be valid are violated.
    /// This may include things like parallel edges or self loops depending on implementation.
    ///
    /// Refer to the documentation of the underlying storage for more information.
    pub fn try_insert_node(
        &mut self,
        attributes: impl Into<Attributes<<S::NodeId as GraphId>::AttributeIndex, S::NodeWeight>>,
    ) -> Result<NodeMut<S>, S::Error> {
        let Attributes { id, weight } = attributes.into();

        let id = self.storage.next_node_id(id);
        self.storage.insert_node(id, weight)
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
    /// let mut graph = DiDinoGraph::new();
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
    /// standard library (through [`alloc::Vec::push`], or [`std::collections::HashMap::insert`]).
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
    pub fn insert_node(
        &mut self,
        attributes: impl Into<Attributes<<S::NodeId as GraphId>::AttributeIndex, S::NodeWeight>>,
    ) -> NodeMut<S> {
        self.try_insert_node(attributes)
            .expect("Invariant violation. Try using `try_insert_node` instead.")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::NodeId: ManagedGraphId,
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
    /// let mut graph = DiDinoGraph::new();
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
        weight: impl FnOnce(&S::NodeId) -> S::NodeWeight,
    ) -> Result<NodeMut<S>, S::Error> {
        let id = self.storage.next_node_id(NoValue::new());
        let weight = weight(&id);

        self.storage.insert_node(id, weight)
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
    /// let mut graph = DiDinoGraph::new();
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
    pub fn insert_node_with(
        &mut self,
        weight: impl FnOnce(&S::NodeId) -> S::NodeWeight,
    ) -> NodeMut<S> {
        self.try_insert_node_with(weight)
            .expect("Invariant violation. Try using `try_insert_node_with` instead.")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::NodeId: ArbitraryGraphId,
{
    /// Insert a node, or update the weight of an existing node.
    ///
    /// This is the fallible version of [`Self::upsert_node`].
    // TODO: Example
    /// # Errors
    ///
    /// The same errors as [`Self::try_insert_node`] may occur.
    ///
    /// # Panics
    ///
    /// If the storage is inconsistent.
    /// This should never happen, as this is an implementation error of the underlying
    /// [`GraphStorage`], which must guarantee that if one calls [`GraphStorage::contains_node`]
    /// with the id of the node to check if a node exists, and then calls
    /// [`GraphStorage::node_mut`] with the same id, it must return a node.
    pub fn try_upsert_node(
        &mut self,
        id: S::NodeId,
        weight: S::NodeWeight,
    ) -> Result<NodeMut<S>, S::Error> {
        // we cannot use `if let` here due to limitations of the borrow checker
        if self.storage.contains_node(&id) {
            let mut node = self
                .storage
                .node_mut(&id)
                .expect("inconsistent storage, node must exist");

            *node.weight_mut() = weight;

            Ok(node)
        } else {
            self.storage.insert_node(id, weight)
        }
    }

    /// Insert a node, or update the weight of an existing node.
    ///
    /// This is the infallible version of [`Self::try_upsert_node`], which will panic instead.
    ///
    /// # Panics
    ///
    /// The same panics as [`Self::try_upsert_node`] may occur, as well as the ones from
    /// [`Self::insert_node`].
    pub fn upsert_node(&mut self, id: S::NodeId, weight: S::NodeWeight) -> NodeMut<S> {
        self.try_upsert_node(id, weight)
            .expect("unable to insert node. Try using `try_upsert_node` instead.")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    pub fn try_insert_edge(
        &mut self,
        attributes: impl Into<Attributes<<S::EdgeId as GraphId>::AttributeIndex, S::EdgeWeight>>,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Result<EdgeMut<S>, S::Error> {
        let Attributes { id, weight } = attributes.into();

        let id = self.storage.next_edge_id(id);
        self.storage.insert_edge(id, weight, source, target)
    }

    pub fn insert_edge(
        &mut self,
        attributes: impl Into<Attributes<<S::EdgeId as GraphId>::AttributeIndex, S::EdgeWeight>>,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> EdgeMut<S> {
        self.try_insert_edge(attributes, source, target)
            .expect("unable to insert edge")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::EdgeId: ManagedGraphId,
{
    pub fn try_insert_edge_with(
        &mut self,
        weight: impl FnOnce(&S::EdgeId) -> S::EdgeWeight,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Result<EdgeMut<S>, S::Error> {
        let id = self.storage.next_edge_id(NoValue::new());
        let weight = weight(&id);

        self.storage.insert_edge(id, weight, source, target)
    }

    pub fn insert_edge_with(
        &mut self,
        weight: impl FnOnce(&S::EdgeId) -> S::EdgeWeight,
        source: &S::NodeId,
        target: &S::NodeId,
    ) -> EdgeMut<S> {
        self.try_insert_edge_with(weight, source, target)
            .expect("unable to insert edge")
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::EdgeId: ArbitraryGraphId,
{
    pub fn upsert_edge(
        &mut self,
        id: S::EdgeId,
        weight: S::EdgeWeight,

        source: &S::NodeId,
        target: &S::NodeId,
    ) -> Result<EdgeMut<S>, S::Error> {
        if self.storage.contains_edge(&id) {
            let mut edge = self
                .storage
                .edge_mut(&id)
                .expect("inconsistent storage, edge must exist");

            *edge.weight_mut() = weight;

            Ok(edge)
        } else {
            self.storage.insert_edge(id, weight, source, target)
        }
    }
}
