mod compat;

use core::fmt::{Debug, Formatter};

use crate::{
    edge::{Direction, Edge},
    storage::{DirectedGraphStorage, GraphStorage},
};

/// Active node in a graph.
///
/// Node that is part of a graph.
///
/// You can access the node's id and weight, as well as its neighbours and connecting edges.
///
/// # Example
///
/// ```
/// use petgraph_dino::DiDinoGraph;
///
/// let mut graph = DiDinoGraph::new();
///
/// let a = *graph.insert_node("A").id();
/// # graph.insert_edge("A → A", &a, &a);
///
/// let node = graph.node(&a).unwrap();
///
/// assert_eq!(node.id(), &a);
/// assert_eq!(node.weight(), &"A");
/// ```
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Node<'a, S>
where
    S: GraphStorage,
{
    storage: &'a S,

    // TODO: MaybeOwned?
    id: &'a S::NodeId,
    // TODO: MaybeOwned?
    weight: &'a S::NodeWeight,
}

impl<S> Clone for Node<'_, S>
where
    S: GraphStorage,
{
    fn clone(&self) -> Self {
        *self
    }
}

impl<S> Copy for Node<'_, S> where S: GraphStorage {}

impl<S> Debug for Node<'_, S>
where
    S: GraphStorage,
    S::NodeId: Debug,
    S::NodeWeight: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Node")
            .field("id", &self.id)
            .field("weight", &self.weight)
            .finish()
    }
}

impl<'a, S> Node<'a, S>
where
    S: GraphStorage,
{
    /// Creates a new node.
    ///
    /// You should not need to use this directly, instead use [`Graph::node`].
    ///
    /// This is only for implementors of [`GraphStorage`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::Direction, node::Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// # graph.insert_edge("A → A", &a, &a);
    ///
    /// let node = graph.node(&a).unwrap();
    /// let copy = Node::new(graph.storage(), node.id(), node.weight());
    /// // ^ exact same as `let copy = *node;`
    /// ```
    pub const fn new(storage: &'a S, id: &'a S::NodeId, weight: &'a S::NodeWeight) -> Self {
        Self {
            storage,
            id,
            weight,
        }
    }

    /// The unique id of the node.
    ///
    /// This is guaranteed to be unique within the graph, the type returned depends on the
    /// implementation of [`GraphStorage`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::Direction, node::Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// # graph.insert_edge("A → A", &a, &a);
    ///
    /// let node = graph.node(&a).unwrap();
    ///
    /// assert_eq!(node.id(), &a);
    /// ```
    #[must_use]
    pub const fn id(&self) -> &'a S::NodeId {
        self.id
    }

    /// The weight of the node.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::Direction, node::Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// # graph.insert_edge("A → A", &a, &a);
    ///
    /// let node = graph.node(&a).unwrap();
    ///
    /// assert_eq!(node.weight(), &"A");
    /// ```
    #[must_use]
    pub const fn weight(&self) -> &'a S::NodeWeight {
        self.weight
    }
}

impl<'a, S> Node<'a, S>
where
    S: GraphStorage,
{
    /// Returns an iterator over the node's neighbours.
    ///
    /// This will return _all_ neighbours, regardless of direction.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let c = *graph.insert_node("C").id();
    ///
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    /// let bc = *graph.insert_edge("B → C", &b, &c).id();
    /// let ca = *graph.insert_edge("C → A", &c, &a).id();
    ///
    /// let node = graph.node(&a).unwrap();
    ///
    /// assert_eq!(
    ///     node.neighbours().map(|node| *node.id()).collect::<Vec<_>>(),
    ///     vec![b, c]
    /// );
    /// ```
    pub fn neighbours(&self) -> impl Iterator<Item = Node<'_, S>> {
        self.storage.node_neighbours(self.id)
    }

    /// Returns an iterator over the node's connecting edges.
    ///
    /// This will return _all_ connecting edges, regardless of direction.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let c = *graph.insert_node("C").id();
    ///
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    /// let bc = *graph.insert_edge("B → C", &b, &c).id();
    /// let ca = *graph.insert_edge("C → A", &c, &a).id();
    ///
    /// let node = graph.node(&a).unwrap();
    ///
    /// assert_eq!(
    ///     node.connections()
    ///         .map(|edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     vec![ab, ca]
    /// );
    /// ```
    pub fn connections(&self) -> impl Iterator<Item = Edge<'_, S>> {
        self.storage.node_connections(self.id)
    }
}

impl<'a, S> Node<'a, S>
where
    S: DirectedGraphStorage,
{
    /// Returns an iterator over the node's neighbours in the given direction.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::Direction;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let c = *graph.insert_node("C").id();
    ///
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    /// let bc = *graph.insert_edge("B → C", &b, &c).id();
    /// let ca = *graph.insert_edge("C → A", &c, &a).id();
    ///
    /// let node = graph.node(&a).unwrap();
    ///
    /// assert_eq!(
    ///     node.directed_connections(Direction::Outgoing)
    ///         .map(|edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     vec![ab]
    /// );
    ///
    /// assert_eq!(
    ///     node.directed_connections(Direction::Incoming)
    ///         .map(|edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     vec![ca]
    /// );
    /// ```
    pub fn directed_connections(&self, direction: Direction) -> impl Iterator<Item = Edge<'_, S>> {
        self.storage.node_directed_connections(self.id, direction)
    }

    /// Returns an iterator over the node's neighbours in the given direction.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::Direction;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// let c = *graph.insert_node("C").id();
    ///
    /// let ab = *graph.insert_edge("A → B", &a, &b).id();
    /// let bc = *graph.insert_edge("B → C", &b, &c).id();
    /// let ca = *graph.insert_edge("C → A", &c, &a).id();
    ///
    /// let node = graph.node(&a).unwrap();
    ///
    /// assert_eq!(
    ///     node.directed_neighbours(Direction::Outgoing)
    ///         .map(|edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     vec![b]
    /// );
    ///
    /// assert_eq!(
    ///     node.directed_neighbours(Direction::Incoming)
    ///         .map(|edge| *edge.id())
    ///         .collect::<Vec<_>>(),
    ///     vec![c]
    /// );
    /// ```
    pub fn directed_neighbours(&self, direction: Direction) -> impl Iterator<Item = Node<'_, S>> {
        self.storage.node_directed_neighbours(self.id, direction)
    }
}

impl<S> Node<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::NodeWeight: Clone,
{
    /// Detaches the node from the graph.
    ///
    /// This will return a [`DetachedNode`], which can be reattached to the graph using
    /// [`Graph::from_parts`].
    ///
    /// This is especially useful in usecases where you want direct (mutable access) to both the
    /// weight and id or do not want to bother with the graph's lifetime.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::Direction;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// # graph.insert_edge("A → A", &a, &a);
    ///
    /// let node = graph.node(&a).unwrap().detach();
    ///
    /// assert_eq!(node.id, a);
    /// assert_eq!(node.weight, "A");
    /// ```
    #[must_use]
    pub fn detach(self) -> DetachedNode<S::NodeId, S::NodeWeight> {
        DetachedNode::new(self.id.clone(), self.weight.clone())
    }
}

/// Active (mutable) node in a graph.
///
/// Node that is part of a graph and has exclusive mutable access to the weight of the node.
///
/// To prevent multiple borrows of the same node, while still allowing for multiple borrows into the
/// same storage, this type does not carry a reference to the storage.
///
/// # Example
///
/// ```
/// use petgraph_dino::DiDinoGraph;
///
/// let mut graph = DiDinoGraph::new();
///
/// let mut a = graph.insert_node("A");
///
/// assert_eq!(a.weight(), &"A");
/// assert_eq!(a.weight_mut(), &mut "A");
///
/// # let a = *a.id();
/// # graph.insert_edge("A → A", &a, &a);
/// ```
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeMut<'a, S>
where
    S: GraphStorage,
{
    id: &'a S::NodeId,

    weight: &'a mut S::NodeWeight,
}

impl<'a, S> NodeMut<'a, S>
where
    S: GraphStorage,
{
    /// Creates a new node.
    ///
    /// You should not need to use this directly, instead use [`Graph::node_mut`] or
    /// [`Graph::insert_node`].
    ///
    /// This is only for implementors of [`GraphStorage`].
    pub fn new(id: &'a S::NodeId, weight: &'a mut S::NodeWeight) -> Self {
        Self { id, weight }
    }

    /// The unique id of the node.
    ///
    /// This is guaranteed to be unique within the graph, the type returned depends on the
    /// implementation of [`GraphStorage`].
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::Direction, node::Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// # graph.insert_edge("A → A", &a, &a);
    ///
    /// let node = graph.node_mut(&a).unwrap();
    ///
    /// assert_eq!(node.id(), &a);
    /// ```
    #[must_use]
    pub const fn id(&self) -> &'a S::NodeId {
        self.id
    }

    /// The weight of the node.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::Direction, node::Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// # graph.insert_edge("A → A", &a, &a);
    ///
    /// let node = graph.node_mut(&a).unwrap();
    ///
    /// assert_eq!(node.weight(), &"A");
    /// ```
    #[must_use]
    pub fn weight(&self) -> &S::NodeWeight {
        self.weight
    }

    /// The (mutable) weight of the node.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::{edge::Direction, node::Node};
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// # graph.insert_edge("A → A", &a, &a);
    ///
    /// let mut node = graph.node_mut(&a).unwrap();
    ///
    /// assert_eq!(node.weight_mut(), &mut "A");
    /// ```
    pub fn weight_mut(&mut self) -> &mut S::NodeWeight {
        self.weight
    }
}

impl<S> NodeMut<'_, S>
where
    S: GraphStorage,
    S::NodeId: Clone,
    S::NodeWeight: Clone,
{
    /// Detaches the node from the graph.
    ///
    /// This will return a [`DetachedNode`], which can be reattached to the graph using
    /// [`Graph::from_parts`].
    ///
    /// This is especially useful in usecases where you want direct (mutable access) to both the
    /// weight and id or do not want to bother with the graph's lifetime.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::edge::Direction;
    /// use petgraph_dino::DiDinoGraph;
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// # graph.insert_edge("A → A", &a, &a);
    ///
    /// let node = graph.node_mut(&a).unwrap().detach();
    ///
    /// assert_eq!(node.id, a);
    /// assert_eq!(node.weight, "A");
    /// ```
    #[must_use]
    pub fn detach(&self) -> DetachedNode<S::NodeId, S::NodeWeight> {
        DetachedNode::new(self.id.clone(), self.weight.clone())
    }
}

/// Detached node from a graph.
///
/// This is a node that has been detached from a graph, it can be reattached using
/// [`Graph::from_parts`].
///
/// This is especially useful in usecases where you want direct (mutable access) to both the weight
/// and the id or do not want to bother with the graph's lifetime.
///
/// # Example
///
/// Detaches the node from the graph.
///
/// This will return a [`DetachedNode`], which can be reattached to the graph using
/// [`Graph::from_parts`].
///
/// This is especially useful in use-cases where you want direct (mutable access) to both the
/// weight and id or do not want to bother with the graph's lifetime.
///
/// # Example
///
/// ```
/// use petgraph_core::edge::Direction;
/// use petgraph_dino::DiDinoGraph;
///
/// let mut graph = DiDinoGraph::new();
///
/// let a = *graph.insert_node("A").id();
/// # graph.insert_edge("A → A", &a, &a);
///
/// let node = graph.node(&a).unwrap().detach();
///
/// assert_eq!(node.id, a);
/// assert_eq!(node.weight, "A");
/// ```
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetachedNode<N, W> {
    pub id: N,

    pub weight: W,
}

impl<N, W> DetachedNode<N, W> {
    /// Creates a new detached node.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::node::DetachedNode;
    ///
    /// let node = DetachedNode::new(0, "A");
    /// ```
    pub const fn new(id: N, weight: W) -> Self {
        Self { id, weight }
    }
}
