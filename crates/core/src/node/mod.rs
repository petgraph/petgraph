//! # Nodes
//!
//! This module contains the three central node types used throughout the petgraph ecosystem:
//! * [`Node`]: Active node in a graph.
//! * [`NodeMut`]: Active (mutable) node in a graph.
//! * [`DetachedNode`]: Detached node from a graph.
//!
//! In application code [`DetachedNode`]s can easily be interchanged with [`Node`]s and vice-versa,
//! as long as all components are [`Clone`]able.
//!
//! You should prefer to use [`Node`]s whenever possible, as they are simply three references into
//! the underlying graph storage and provide convenient access to the node's neighbours and
//! connecting edges.
//!
//! [`NodeMut`]s are only needed when you need mutable access to the node's weight, and should be
//! used sparingly.
mod compat;

use core::{
    fmt::{Debug, Display, Formatter},
    hash::Hash,
};

use crate::{
    edge::{Direction, Edge},
    storage::{DirectedGraphStorage, GraphStorage},
};

/// ID of a node in a graph.
///
/// This is guaranteed to be unique within the graph, library authors and library consumers **must**
/// treat this as an opaque value akin to [`TypeId`].
///
/// The layout of the type is semver stable, but not part of the public API.
///
/// [`GraphStorage`] implementations may uphold additional invariants on the inner value and
/// code outside of the [`GraphStorage`] should **never** construct a [`NodeId`] directly.
///
/// Accessing a [`GraphStorage`] implementation with a [`NodeId`] not returned by an instance itself
/// is considered undefined behavior.
///
/// [`TypeId`]: core::any::TypeId
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(usize);

impl Display for NodeId {
    // we could also utilize a VTable here instead, that would allow for custom formatting
    // but that would be an additional pointer added to the type that must be carried around
    // that's about ~8 bytes on 64-bit systems
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        writeln!(f, "NodeId({})", self.0)
    }
}

// TODO: find a better way to gate these functions
impl NodeId {
    /// Creates a new [`NodeId`].
    ///
    /// # Note
    ///
    /// Using this outside of the [`GraphStorage`] implementation is considered undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::node::NodeId;
    ///
    /// let id = NodeId::new(0);
    /// ```
    // Hidden so that non-GraphStorage implementors are not tempted to use this.
    #[doc(hidden)]
    #[must_use]
    pub const fn new(id: usize) -> Self {
        Self(id)
    }

    /// Returns the inner value of the [`NodeId`].
    ///
    /// # Note
    ///
    /// Using this outside of the [`GraphStorage`] implementation is considered undefined behavior.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::node::NodeId;
    ///
    /// let id = NodeId::new(0);
    ///
    /// assert_eq!(id.into_inner(), 0);
    /// ```
    // Hidden so that non-GraphStorage implementors are not tempted to use this.
    #[doc(hidden)]
    #[must_use]
    pub const fn into_inner(self) -> usize {
        self.0
    }
}

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
pub struct Node<'a, S>
where
    S: GraphStorage,
{
    storage: &'a S,

    id: NodeId,
    weight: &'a S::NodeWeight,
}

impl<S> PartialEq for Node<'_, S>
where
    S: GraphStorage,
    S::NodeWeight: PartialEq,
{
    fn eq(&self, other: &Node<'_, S>) -> bool {
        (self.id, self.weight).eq(&(other.id, other.weight))
    }
}

impl<S> Eq for Node<'_, S>
where
    S: GraphStorage,
    S::NodeWeight: Eq,
{
}

impl<S> PartialOrd for Node<'_, S>
where
    S: GraphStorage,
    S::NodeWeight: PartialOrd,
{
    fn partial_cmp(&self, other: &Node<'_, S>) -> Option<core::cmp::Ordering> {
        (self.id, self.weight).partial_cmp(&(other.id, other.weight))
    }
}

impl<S> Ord for Node<'_, S>
where
    S: GraphStorage,
    S::NodeWeight: Ord,
{
    fn cmp(&self, other: &Node<'_, S>) -> core::cmp::Ordering {
        (self.id, self.weight).cmp(&(other.id, other.weight))
    }
}

impl<S> Hash for Node<'_, S>
where
    S: GraphStorage,
    S::NodeWeight: Hash,
{
    fn hash<H: core::hash::Hasher>(&self, state: &mut H) {
        (self.id, self.weight).hash(state);
    }
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
    /// # Contract
    ///
    /// The `id` must be valid for the given `storage` and the `weight` must be valid for the given
    /// `id`.
    ///
    /// The contract is not enforced, due to concerns of recursive calls on [`GraphStorage::node`]
    /// and [`GraphStorage::contains_node`].
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
    ///
    /// [`Graph::node`]: crate::graph::Graph::node
    pub const fn new(storage: &'a S, id: NodeId, weight: &'a S::NodeWeight) -> Self {
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
    pub const fn id(&self) -> NodeId {
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
    pub fn neighbours(&self) -> impl Iterator<Item = Node<'a, S>> + 'a {
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
    pub fn connections(&self) -> impl Iterator<Item = Edge<'a, S>> + 'a {
        self.storage.node_connections(self.id)
    }

    // TODO: implement degree in petgraph-dino and test
    /// Returns the number of edges connected to the node.
    ///
    /// This is also known as the node's degree or valency.
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
    /// assert_eq!(node.degree(), 2);
    /// ```
    #[must_use]
    pub fn degree(&self) -> usize {
        self.storage.node_degree(self.id)
    }

    /// Change the underlying storage of the node.
    ///
    /// Should only be used when layering multiple graph storages on top of each other by graph
    /// storage implementors.
    ///
    /// # Safety
    ///
    /// Can lead to undefined behaviour if the id is not valid for the given storage.
    pub unsafe fn change_storage_unchecked<S2>(self) -> Node<'a, S2>
    where
        S2: GraphStorage<NodeWeight = S::NodeWeight>,
    {
        Node::new(self.storage, self.id, self.weight)
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
    pub fn directed_connections(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<'a, S>> + 'a {
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
    pub fn directed_neighbours(
        &self,
        direction: Direction,
    ) -> impl Iterator<Item = Node<'a, S>> + 'a {
        self.storage.node_directed_neighbours(self.id, direction)
    }
}

impl<S> Node<'_, S>
where
    S: GraphStorage,
    S::NodeWeight: Clone,
{
    /// Detaches the node from the graph.
    ///
    /// > **Note:** This will _not_ remove the node from the graph, it will only detach the this
    /// > instance of the node removing the lifetime dependency on the graph.
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
    ///
    /// [`Graph::from_parts`]: crate::graph::Graph::from_parts
    #[must_use]
    pub fn detach(self) -> DetachedNode<S::NodeWeight> {
        DetachedNode::new(self.id, self.weight.clone())
    }
}

/// Active (mutable) node in a graph.
///
/// Node that is part of a graph and has exclusive mutable access to the weight of the node.
///
/// To prevent multiple borrows of the same node, while still allowing for multiple borrows into the
/// same storage, this type does not carry a reference to the storage itself.
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
    id: NodeId,

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
    ///
    /// # Contract
    ///
    /// The `id` must be valid for the given `storage` and the `weight` must be valid for the given
    /// `id`.
    ///
    ///
    /// [`Graph::node_mut`]: crate::graph::Graph::node_mut
    /// [`Graph::insert_node`]: crate::graph::Graph::insert_node
    pub fn new(id: NodeId, weight: &'a mut S::NodeWeight) -> Self {
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
    pub const fn id(&self) -> NodeId {
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

    /// Change the underlying storage of the node.
    ///
    /// Should only be used when layering multiple graph storages on top of each other by graph
    /// storage implementors.
    ///
    /// # Safety
    ///
    /// Can lead to undefined behaviour if the id is not valid for the given storage.
    pub unsafe fn change_storage_unchecked<S2>(self) -> NodeMut<'a, S2>
    where
        S2: GraphStorage<NodeWeight = S::NodeWeight>,
    {
        NodeMut::new(self.id, self.weight)
    }
}

impl<S> NodeMut<'_, S>
where
    S: GraphStorage,
    S::NodeWeight: Clone,
{
    /// Detaches the node from the graph.
    ///
    /// > **Note:** This will _not_ remove the node from the graph, it will only detach the this
    /// > instance of the node removing the lifetime dependency on the graph.
    ///
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
    /// let node = graph.node_mut(&a).unwrap().detach();
    ///
    /// assert_eq!(node.id, a);
    /// assert_eq!(node.weight, "A");
    /// ```
    ///
    /// [`Graph::from_parts`]: crate::graph::Graph::from_parts
    #[must_use]
    pub fn detach(&self) -> DetachedNode<S::NodeWeight> {
        DetachedNode::new(self.id, self.weight.clone())
    }
}

/// Detached node from a graph.
///
/// This node is no longer considered to be part of a graph, but can be reattached using
/// [`Graph::from_parts`].
///
/// Especially useful in cases of decomposition ([`Graph::into_parts`]) or if one does not want to
/// deal with the graph's lifetime or needs a node to outlive the graph.
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
///
/// [`Graph::into_parts`]: crate::graph::Graph::into_parts
/// [`Graph::from_parts`]: crate::graph::Graph::from_parts
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DetachedNode<W> {
    /// The unique id of the node.
    pub id: NodeId,

    /// The weight of the node.
    pub weight: W,
}

impl<W> DetachedNode<W> {
    /// Creates a new detached node.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::node::DetachedNode;
    ///
    /// let node = DetachedNode::new(0, "A");
    /// ```
    pub const fn new(id: NodeId, weight: W) -> Self {
        Self { id, weight }
    }
}
