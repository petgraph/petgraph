//! [`MatrixGraph`] is a graph data structure backed by an adjacency matrix. It can represent both
//! directed and undirected graphs and supports arbitrary associated data for nodes and edges.
//!
//! This module contains the implementation of [`MatrixGraph`].
extern crate alloc;

use alloc::{fmt, vec, vec::Vec};
use core::{
    cmp,
    convert::Infallible,
    error::Error,
    fmt::{Debug, Display},
    marker::PhantomData,
};

use fixedbitset::FixedBitSet;
use petgraph_core::{
    graph::{Directed, EditableGraph, Graph, Undirected},
    id::{Id, IndexId},
};
use slab::Slab;

use crate::private::Sealed;

mod directed;
mod undirected;

pub use directed::{DiMatrixEdgeId, DiMatrixGraph};
pub use undirected::{UnMatrixEdgeId, UnMatrixGraph};

/// `NodeIds` type for the [`Graph`] implementation of [`MatrixGraph`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MatrixGraphNodeId(usize);

impl Display for MatrixGraphNodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

impl Id for MatrixGraphNodeId {}

impl From<usize> for MatrixGraphNodeId {
    fn from(value: usize) -> Self {
        Self(value)
    }
}

impl From<u16> for MatrixGraphNodeId {
    fn from(value: u16) -> Self {
        Self(value as usize)
    }
}

impl From<u32> for MatrixGraphNodeId {
    fn from(value: u32) -> Self {
        Self(value as usize)
    }
}

impl TryFrom<u64> for MatrixGraphNodeId {
    type Error = <usize as TryFrom<u64>>::Error;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(Self(usize::try_from(value)?))
    }
}

impl IndexId for MatrixGraphNodeId {
    const MAX: Self = Self(usize::MAX);
    const MIN: Self = Self(0);

    fn as_u16(self) -> u16 {
        u16::try_from(self.0).unwrap()
    }

    fn as_u32(self) -> u32 {
        u32::try_from(self.0).unwrap()
    }

    fn as_u64(self) -> u64 {
        self.0 as u64
    }

    fn as_usize(self) -> usize {
        self.0
    }
}

mod private {
    pub trait Sealed {}

    impl<T> Sealed for super::NotZero<T> {}
    impl<T> Sealed for Option<T> {}
}

/// Wrapper trait for an `Option`.
///
/// Allows for user-defined structs to be input as containers for Edge Data in [`MatrixGraph`].
/// Wraps types that have a notion of "null" or "empty" value, which is used to mark the absence of
/// an edge in the graph.
///
/// Note: this trait is currently *sealed* and cannot be implemented for types outside this crate.
pub trait NicheWrapper:
    Default + Into<Option<<Self as NicheWrapper>::Wrapped>> + private::Sealed
{
    #[doc(hidden)]
    type Wrapped;

    #[doc(hidden)]
    fn new(value: Self::Wrapped) -> Self;

    #[doc(hidden)]
    fn as_ref(&self) -> Option<&Self::Wrapped>;

    #[doc(hidden)]
    fn as_mut(&mut self) -> Option<&mut Self::Wrapped>;

    #[doc(hidden)]
    fn is_null(&self) -> bool {
        self.as_ref().is_none()
    }
}

impl<T> NicheWrapper for Option<T> {
    type Wrapped = T;

    fn new(value: T) -> Self {
        Some(value)
    }

    fn as_ref(&self) -> Option<&Self::Wrapped> {
        self.as_ref()
    }

    fn as_mut(&mut self) -> Option<&mut Self::Wrapped> {
        self.as_mut()
    }
}

/// `NotZero` is used to optimize the memory usage of edge data `E` in a
/// [`MatrixGraph`], replacing the default `Option<E>` sentinel.
///
/// The [`Default`] value of the edge data (such as `0` for the numeric types) is used as the
/// sentinel to mark the absence of an edge. Edge data must therefore implement [`Default`] and
/// [`PartialEq`], and adding an edge whose data equals the default value panics.
///
/// Note that if you're already using the standard non-zero types (such as `NonZeroU32`), you don't
/// have to use this wrapper and can leave the default `Null` type argument.
pub struct NotZero<T>(T);

impl<T: Default + PartialEq> Default for NotZero<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

impl<T: Default + PartialEq> NicheWrapper for NotZero<T> {
    #[doc(hidden)]
    type Wrapped = T;

    #[doc(hidden)]
    fn new(value: T) -> Self {
        assert!(value != T::default());
        Self(value)
    }

    // implemented here for optimization purposes
    #[doc(hidden)]
    fn is_null(&self) -> bool {
        self.0 == T::default()
    }

    #[doc(hidden)]
    fn as_ref(&self) -> Option<&Self::Wrapped> {
        if self.is_null() { None } else { Some(&self.0) }
    }

    #[doc(hidden)]
    fn as_mut(&mut self) -> Option<&mut Self::Wrapped> {
        if self.is_null() {
            None
        } else {
            Some(&mut self.0)
        }
    }
}

impl<T: Default + PartialEq> From<NotZero<T>> for Option<T> {
    fn from(not_zero: NotZero<T>) -> Self {
        if not_zero.is_null() {
            None
        } else {
            Some(not_zero.0)
        }
    }
}

/// Internal trait for extra methods for directed and undirected [`MatrixGraph`]s, to simplify their
/// implementations.
pub trait MatrixGraphExtras<N>: Sealed + Graph {
    /// Return the position of the edge from `node_a` to `node_b` in the flattened adjacency matrix.
    ///
    /// Returns `None` if either of the nodes doesn't exist.
    fn to_edge_position(
        &self,
        node_a: MatrixGraphNodeId,
        node_b: MatrixGraphNodeId,
    ) -> Option<usize>;

    /// Return the position of the edge from `node_a` to `node_b` in the flattened adjacency matrix.
    ///
    /// # Panics
    /// May panic if either of the nodes doesn't exist.
    fn to_edge_position_unchecked(
        &self,
        node_a: MatrixGraphNodeId,
        node_b: MatrixGraphNodeId,
    ) -> usize;

    /// Extends the capacity of the underlying flattened edge data vec for the node.
    fn extend_capacity_for_node(&mut self, new_node_capacity: usize, exact: bool);

    /// Remove `node` from the graph.
    ///
    /// Returns `None` if the node doesn't exist, otherwise returns the associated data of the
    /// removed node.
    fn remove_node_internal(&mut self, node: MatrixGraphNodeId) -> Option<N>;

    /// Return the source node of the edge with the given `EdgeId`.
    fn edge_id_to_source(edge_id: Self::EdgeId) -> MatrixGraphNodeId;

    /// Return the target node of the edge with the given `EdgeId`.
    fn edge_id_to_target(edge_id: Self::EdgeId) -> MatrixGraphNodeId;

    /// Return the `EdgeId` of the edge from `source` to `target`.
    fn edge_id_from_source_target(
        source: MatrixGraphNodeId,
        target: MatrixGraphNodeId,
    ) -> Self::EdgeId;
}

/// `MatrixGraph<N, E, Dir, Null>` is a graph using an adjacency matrix representation.
///
/// It uses a flattened 2D array to store edge data, and a separate storage for node data and
/// management of node indices. The graph can be instantiated as either directed or undirected.
/// The default being directed.
///
/// See [`DiMatrixGraph`] and [`UnMatrixGraph`] for the directed and undirected versions,
/// respectively.
///
/// Edge data can be of arbitrary type that has a notion of "null" or "empty" value, which is used
/// to mark the absence of an edge in the graph and allows for better memory optimization.
///
/// `MatrixGraph` is parameterized over:
/// - Associated data `N` for nodes and `E` for edges. The associated data can be of arbitrary type.
/// - Edge type `Dir` determines whether the graph edges are [`Directed`] or [`Undirected`].
/// - Nullable type `Null`, which denotes the edges' presence (defaults to `Option<E>`). You may
///   specify [`NotZero<E>`](struct.NotZero.html) if you want to use the default value of `E` (such
///   as 0) as a sentinel to mark the absence of an edge.
///
/// The graph uses **O(|V^2|)** space, with fast edge insertion & amortized node insertion, as well
/// as efficient graph search and graph algorithms on dense graphs.
///
/// For undirected graphs, only the lower triangular part of the adjacency matrix is stored. Since
/// the backing array stores edge data, it is recommended to box large edge data.
///
/// The graph uses [`MatrixGraphNodeId`] as `NodeIds` and [`DiMatrixEdgeId`] and [`UnMatrixEdgeId`]
/// as `EdgeIds` for directed and undirected graphs, respectively. `NodeIds`  are convertible to
/// `usize`, however not guaranteed to be contiguous. When removing nodes, the graph will reuse the
/// indices of removed nodes for new nodes filling in the gaps. For most use cases however, the
/// graph is assumed to have dense node indices.
#[derive(Clone)]
pub struct MatrixGraph<N, E, Dir = Directed, Null: NicheWrapper<Wrapped = E> = Option<E>> {
    /// Edge Data including presence information.
    flattened_edge_data: Vec<Null>,
    /// Node data and management of node indices.
    ///
    /// Nodes may be in `node_data` but not be within the current `node_capacity`, if they are
    /// isolated nodes. The flattened edge data and thus `node_capacity` is updated lazily,
    /// that is, only once an edge adjacent to such a node is actually created.
    node_data: Slab<N>,
    /// The current edge capacity with respect to the number of nodes. This is used to determine
    /// when the backing matrix needs to be resized.
    node_capacity: usize,
    /// The number of edges currently in the graph.
    edge_count: usize,
    directionality: PhantomData<Dir>,
}

impl<N, E, Dir, Null: NicheWrapper<Wrapped = E>> MatrixGraph<N, E, Dir, Null> {
    /// Remove all nodes and edges.
    pub fn clear(&mut self) {
        for edge in &mut self.flattened_edge_data {
            *edge = Default::default();
        }
        self.node_data.clear();
        self.edge_count = 0;
    }
}

impl<N, E, Dir, Null: NicheWrapper<Wrapped = E>> MatrixGraph<N, E, Dir, Null>
where
    Self: MatrixGraphExtras<N>,
{
    /// Create a new `MatrixGraph` with capacity sufficient to hold the specified number of nodes
    /// (and any edges between them) without resizing.
    #[must_use]
    pub fn with_capacity(node_capacity: usize) -> Self {
        let mut graph = Self {
            flattened_edge_data: vec![],
            node_capacity: 0,
            node_data: Slab::with_capacity(node_capacity),
            edge_count: 0,
            directionality: PhantomData,
        };

        if node_capacity > 0 {
            graph.extend_capacity_for_node(node_capacity, true);
        }

        graph
    }

    #[inline]
    fn extend_capacity_for_edge(&mut self, node_a: MatrixGraphNodeId, node_b: MatrixGraphNodeId) {
        let max_node = cmp::max(node_a, node_b);
        if max_node.0 >= self.node_capacity {
            self.extend_capacity_for_node(
                max_node
                    .0
                    .checked_add(1)
                    .unwrap_or_else(|| panic!("Matrix capacity overflow")),
                false,
            );
        }
    }
}

impl<N, E, Dir, Null: NicheWrapper<Wrapped = E>> Default for MatrixGraph<N, E, Dir, Null>
where
    Self: MatrixGraphExtras<N>,
{
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<N, E> MatrixGraph<N, E, Directed, Option<E>> {
    /// Create a new `MatrixGraph` with directed edges.
    ///
    /// This is a convenience method. Use `MatrixGraph::with_capacity` or `MatrixGraph::default` for
    /// a constructor that is generic in all the type parameters of `MatrixGraph`.
    #[must_use]
    pub fn new_directed() -> Self {
        Self::default()
    }
}

impl<N, E> MatrixGraph<N, E, Undirected, Option<E>> {
    /// Create a new `MatrixGraph` with undirected edges.
    ///
    /// This is a convenience method. Use `MatrixGraph::with_capacity` or `MatrixGraph::default` for
    /// a constructor that is generic in all the type parameters of `MatrixGraph`.
    #[must_use]
    pub fn new_undirected() -> Self {
        Self::default()
    }
}

pub struct MatrixGraphVisitMap<EdgeId> {
    visit_map: FixedBitSet,
    node_capacity: usize,
    phantom: PhantomData<EdgeId>,
}

pub struct MatrixGraphEdgeDataContainer<EdgeId, Data> {
    data: Vec<Data>,
    node_capacity: usize,
    default_value: Data,
    phantom: PhantomData<EdgeId>,
}

impl<N, E, Dir, Null: NicheWrapper<Wrapped = E>> EditableGraph for MatrixGraph<N, E, Dir, Null>
where
    Self: Graph<NodeId = MatrixGraphNodeId> + MatrixGraphExtras<N>,
{
    type AddEdgeError = AddEdgeError;
    type AddNodeError = Infallible;
    type OwnedEdgeData = E;
    type OwnedNodeData = N;
    type RemoveEdgeError = RemoveEdgeError<Self::EdgeId>;
    type RemoveNodeError = RemoveNodeError;

    fn add_node(&mut self, data: Self::OwnedNodeData) -> Result<Self::NodeId, Self::AddNodeError> {
        Ok(MatrixGraphNodeId(self.node_data.insert(data)))
    }

    fn add_edge(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
        data: Self::OwnedEdgeData,
    ) -> Result<Self::EdgeId, Self::AddEdgeError> {
        if !self.node_data.contains(source.0) {
            return Err(AddEdgeError::SourceNodeDoesNotExist(source));
        } else if !self.node_data.contains(target.0) {
            return Err(AddEdgeError::TargetNodeDoesNotExist(target));
        }
        self.extend_capacity_for_edge(source, target);
        // We can safely use the unchecked version here, since we just extended the capacity for the
        // edge.
        let position = self.to_edge_position_unchecked(source, target);
        let old_data = &mut self.flattened_edge_data[position];
        if old_data.is_null() {
            self.edge_count += 1;
            *old_data = Null::new(data);
        } else {
            return Err(AddEdgeError::EdgeAlreadyExists(source, target));
        }
        Ok(<Self as MatrixGraphExtras<N>>::edge_id_from_source_target(
            source, target,
        ))
    }

    fn remove_edge(
        &mut self,
        id: Self::EdgeId,
    ) -> Result<Self::OwnedEdgeData, Self::RemoveEdgeError> {
        let position = self
            .to_edge_position(
                <Self as MatrixGraphExtras<N>>::edge_id_to_source(id),
                <Self as MatrixGraphExtras<N>>::edge_id_to_target(id),
            )
            .ok_or(RemoveEdgeError::EdgeDoesNotExist(id))?;
        let old_data = core::mem::take(&mut self.flattened_edge_data[position]).into();
        let Some(edge_data) = old_data else {
            return Err(RemoveEdgeError::EdgeDoesNotExist(id));
        };
        self.edge_count -= 1;
        Ok(edge_data)
    }

    fn remove_node(
        &mut self,
        id: Self::NodeId,
    ) -> Result<Self::OwnedNodeData, Self::RemoveNodeError> {
        Self::remove_node_internal(self, id).ok_or(RemoveNodeError::NodeDoesNotExist(id))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemoveNodeError {
    /// The node does not exist in the graph.
    NodeDoesNotExist(MatrixGraphNodeId),
}

impl Display for RemoveNodeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NodeDoesNotExist(node_id) => {
                write!(f, "Node {node_id} does not exist in the graph")
            }
        }
    }
}

impl Error for RemoveNodeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AddEdgeError {
    /// The source node does not exist in the graph.
    SourceNodeDoesNotExist(MatrixGraphNodeId),
    /// The target node does not exist in the graph.
    TargetNodeDoesNotExist(MatrixGraphNodeId),
    /// The edge already exists in the graph.
    EdgeAlreadyExists(MatrixGraphNodeId, MatrixGraphNodeId),
}

impl Display for AddEdgeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SourceNodeDoesNotExist(node_id) => {
                write!(f, "Source node {node_id} does not exist in the graph")
            }
            Self::TargetNodeDoesNotExist(node_id) => {
                write!(f, "Target node {node_id} does not exist in the graph")
            }
            Self::EdgeAlreadyExists(source, target) => {
                write!(
                    f,
                    "Edge from {source} to {target} already exists in the graph",
                )
            }
        }
    }
}

impl Error for AddEdgeError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RemoveEdgeError<EdgeId> {
    /// The edge does not exist in the graph.
    EdgeDoesNotExist(EdgeId),
}

impl<EdgeId: Display> Display for RemoveEdgeError<EdgeId> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EdgeDoesNotExist(edge_id) => {
                write!(f, "Edge {edge_id} does not exist in the graph")
            }
        }
    }
}

impl<EdgeId: Display + Debug> Error for RemoveEdgeError<EdgeId> {}

/// This is a helper type for functions that return different types of iterators with the same item
/// type. It is taken from the `itertools` crate, as it is the only type we need from the crate and
/// thus we can avoid pulling in the entire dependency just for this.
enum Either<L, R> {
    /// A value of type `L`.
    Left(L),
    /// A value of type `R`.
    Right(R),
}

impl<L, R> Iterator for Either<L, R>
where
    L: Iterator,
    R: Iterator<Item = L::Item>,
{
    type Item = L::Item;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            Self::Left(left) => left.next(),
            Self::Right(right) => right.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use petgraph_core::graph::DirectedGraph;

    use super::*;

    #[test]
    fn test_new() {
        let graph = MatrixGraph::<i32, i32>::new_directed();
        assert_eq!(graph.node_count(), 0);
        assert_eq!(graph.edge_count(), 0);
    }

    #[test]
    fn test_node_id_reuse() {
        let mut graph = MatrixGraph::<char, ()>::new_directed();
        let node_a = graph.add_node('a').unwrap();
        let node_b = graph.add_node('b').unwrap();
        let node_c = graph.add_node('c').unwrap();

        assert!(node_a < node_b && node_b < node_c);
        assert_eq!(
            graph.nodes().map(|node| node.id).collect::<Vec<_>>(),
            vec![node_a, node_b, node_c]
        );

        graph.remove_node(node_b).unwrap();
        assert_eq!(
            graph.remove_node(node_b),
            Err(RemoveNodeError::NodeDoesNotExist(node_b))
        );

        let node_bb = graph.add_node('B').unwrap();
        assert_eq!(node_b, node_bb);
        assert_eq!(
            graph.nodes().map(|node| node.id).collect::<Vec<_>>(),
            vec![node_a, node_b, node_c]
        );
    }
}
