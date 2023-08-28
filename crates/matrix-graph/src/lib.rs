//! `MatrixGraph<N, E, Ty, NullN, NullE, Ix>` is a graph datastructure backed by an adjacency
//! matrix.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

extern crate alloc;

use alloc::{vec, vec::Vec};
use core::{
    cmp,
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
};

use fixedbitset::FixedBitSet;
use funty::Integral;
use indexmap::IndexSet;
use petgraph_core::{
    data::Build,
    deprecated::IntoWeightedEdge,
    edge::{Directed, Direction, EdgeType, Undirected},
    index::{FromIndexType, IndexType, IntoIndexType, SafeCast},
    visit::{
        Data, EdgeCount, GetAdjacencyMatrix, GraphBase, GraphProp, IntoEdgeReferences, IntoEdges,
        IntoEdgesDirected, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers,
        IntoNodeReferences, NodeCount, NodeIndexable, Visitable,
    },
};

// The following types are used to control the max size of the adjacency matrix. Since the maximum
// size of the matrix vector's is the square of the maximum number of nodes, the number of nodes
// should be reasonably picked.
type DefaultIx = u16;

/// Node identifier.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeIndex<Ix = DefaultIx>(pub(crate) Ix);

impl<Ix> NodeIndex<Ix>
where
    Ix: IndexType,
{
    #[must_use]
    pub const fn new(value: Ix) -> Self {
        Self(value)
    }

    #[must_use]
    pub fn from_usize(value: usize) -> Self {
        Self(Ix::from_usize(value))
    }

    fn into_usize(self) -> usize {
        self.0.as_usize()
    }
}

impl<Ix> FromIndexType for NodeIndex<Ix>
where
    Ix: IndexType,
{
    type Index = Ix;

    fn from_index(index: Self::Index) -> Self {
        Self(index)
    }
}

impl<Ix> IntoIndexType for NodeIndex<Ix>
where
    Ix: IndexType,
{
    type Index = Ix;

    fn into_index(self) -> Self::Index {
        self.0
    }
}

// SAFETY: we simply call the inner type, which already implements `SafeCast`.
unsafe impl<Ix> SafeCast<usize> for NodeIndex<Ix>
where
    Ix: IndexType,
{
    fn cast(self) -> usize {
        self.0.cast()
    }
}

// TODO: Rework
mod private {
    pub trait Sealed {}

    impl<T> Sealed for super::NotZero<T> {}
    impl<T> Sealed for Option<T> {}
}

/// Wrapper trait for an `Option`, allowing user-defined structs to be input as containers when
/// defining a null element.
///
/// Note: this trait is currently *sealed* and cannot be implemented for types outside this crate.
pub trait Nullable: Default + Into<Option<<Self as Nullable>::Wrapped>> + private::Sealed {
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

impl<T> Nullable for Option<T> {
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

/// `NotZero` is used to optimize the memory usage of edge weights `E` in a
/// [`MatrixGraph`](struct.MatrixGraph.html), replacing the default `Option<E>` sentinel.
///
/// Pre-requisite: edge weight should implement [`Zero`](trait.Zero.html).
///
/// Note that if you're already using the standard non-zero types (such as `NonZeroU32`), you don't
/// have to use this wrapper and can leave the default `Null` type argument.
pub struct NotZero<T>(T);

impl<T: Zero> Default for NotZero<T> {
    fn default() -> Self {
        NotZero(T::zero())
    }
}

impl<T: Zero> Nullable for NotZero<T> {
    #[doc(hidden)]
    type Wrapped = T;

    #[doc(hidden)]
    fn new(value: T) -> Self {
        assert!(!value.is_zero());
        NotZero(value)
    }

    // implemented here for optimization purposes
    #[doc(hidden)]
    fn is_null(&self) -> bool {
        self.0.is_zero()
    }

    #[doc(hidden)]
    fn as_ref(&self) -> Option<&Self::Wrapped> {
        if !self.is_null() { Some(&self.0) } else { None }
    }

    #[doc(hidden)]
    fn as_mut(&mut self) -> Option<&mut Self::Wrapped> {
        if !self.is_null() {
            Some(&mut self.0)
        } else {
            None
        }
    }
}

impl<T: Zero> From<NotZero<T>> for Option<T> {
    fn from(not_zero: NotZero<T>) -> Self {
        if !not_zero.is_null() {
            Some(not_zero.0)
        } else {
            None
        }
    }
}

/// Base trait for types that can be wrapped in a [`NotZero`](struct.NotZero.html).
///
/// Implementors must provide a singleton object that will be used to mark empty edges in a
/// [`MatrixGraph`](struct.MatrixGraph.html).
///
/// Note that this trait is already implemented for the base numeric types.
pub trait Zero {
    /// Return the singleton object which can be used as a sentinel value.
    fn zero() -> Self;

    /// Return true if `self` is equal to the sentinel value.
    fn is_zero(&self) -> bool;
}

macro_rules! not_zero_impl {
    ($t:ty, $z:expr) => {
        impl Zero for $t {
            fn zero() -> Self {
                $z as $t
            }

            #[allow(clippy::float_cmp)]
            fn is_zero(&self) -> bool {
                self == &Self::zero()
            }
        }
    };
}

macro_rules! not_zero_impls {
    ($($t:ty),*) => {
        $(
            not_zero_impl!($t, 0);
        )*
    }
}

not_zero_impls!(u8, u16, u32, u64, usize);
not_zero_impls!(i8, i16, i32, i64, isize);
not_zero_impls!(f32, f64);

/// Short version of `NodeIndex::new` (with Ix = `DefaultIx`)
#[inline]
#[deprecated(since = "0.1.0", note = "use `NodeIndex::new` instead")]
pub fn node_index(ax: usize) -> NodeIndex {
    NodeIndex::new(u16::from_usize(ax))
}

/// `MatrixGraph<N, E, Ty, Null>` is a graph datastructure using an adjacency matrix
/// representation.
///
/// `MatrixGraph` is parameterized over:
///
/// - Associated data `N` for nodes and `E` for edges, called *weights*. The associated data can be
///   of arbitrary type.
/// - Edge type `Ty` that determines whether the graph edges are directed or undirected.
/// - Nullable type `Null`, which denotes the edges' presence (defaults to `Option<E>`). You may
///   specify [`NotZero<E>`](struct.NotZero.html) if you want to use a sentinel value (such as 0) to
///   mark the absence of an edge.
/// - Index type `Ix` that sets the maximum size for the graph (defaults to `DefaultIx`).
///
/// The graph uses **O(|V^2|)** space, with fast edge insertion & amortized node insertion, as well
/// as efficient graph search and graph algorithms on dense graphs.
///
/// This graph is backed by a flattened 2D array. For undirected graphs, only the lower triangular
/// matrix is stored. Since the backing array stores edge weights, it is recommended to box large
/// edge weights.
#[derive(Clone)]
pub struct MatrixGraph<N, E, Ty = Directed, Null: Nullable<Wrapped = E> = Option<E>, Ix = DefaultIx>
{
    node_adjacencies: Vec<Null>,
    node_capacity: usize,
    nodes: IdStorage<N>,
    nb_edges: usize,
    ty: PhantomData<Ty>,
    ix: PhantomData<Ix>,
}

/// A `MatrixGraph` with directed edges.
pub type DiMatrix<N, E, Null = Option<E>, Ix = DefaultIx> = MatrixGraph<N, E, Directed, Null, Ix>;

/// A `MatrixGraph` with undirected edges.
pub type UnMatrix<N, E, Null = Option<E>, Ix = DefaultIx> = MatrixGraph<N, E, Undirected, Null, Ix>;

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType>
    MatrixGraph<N, E, Ty, Null, Ix>
{
    /// Create a new `MatrixGraph` with estimated capacity for nodes.
    pub fn with_capacity(node_capacity: usize) -> Self {
        let mut m = Self {
            node_adjacencies: vec![],
            node_capacity: 0,
            nodes: IdStorage::with_capacity(node_capacity),
            nb_edges: 0,
            ty: PhantomData,
            ix: PhantomData,
        };

        debug_assert!(node_capacity <= <Ix as Integral>::MAX.cast());
        if node_capacity > 0 {
            m.extend_capacity_for_node(NodeIndex::new(Ix::from_usize(node_capacity - 1)), true);
        }

        m
    }

    #[inline]
    fn to_edge_position(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<usize> {
        if cmp::max(a.cast(), b.cast()) >= self.node_capacity {
            return None;
        }
        Some(self.to_edge_position_unchecked(a, b))
    }

    #[inline]
    fn to_edge_position_unchecked(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> usize {
        to_linearized_matrix_position::<Ty>(a.into_usize(), b.into_usize(), self.node_capacity)
    }

    /// Remove all nodes and edges.
    pub fn clear(&mut self) {
        for edge in self.node_adjacencies.iter_mut() {
            *edge = Default::default();
        }
        self.nodes.clear();
        self.nb_edges = 0;
    }

    /// Return the number of nodes (vertices) in the graph.
    ///
    /// Computes in **O(1)** time.
    #[inline]
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// Return the number of edges in the graph.
    ///
    /// Computes in **O(1)** time.
    #[inline]
    pub fn edge_count(&self) -> usize {
        self.nb_edges
    }

    /// Return whether the graph has directed edges or not.
    #[inline]
    pub fn is_directed(&self) -> bool {
        Ty::is_directed()
    }

    /// Add a node (also called vertex) with associated data `weight` to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Return the index of the new node.
    ///
    /// **Panics** if the MatrixGraph is at the maximum number of nodes for its index type.
    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        NodeIndex::new(Ix::from_usize(self.nodes.add(weight)))
    }

    /// Remove `a` from the graph.
    ///
    /// Computes in **O(V)** time, due to the removal of edges with other nodes.
    ///
    /// **Panics** if the node `a` does not exist.
    pub fn remove_node(&mut self, a: NodeIndex<Ix>) -> N {
        for id in self.nodes.iter_ids() {
            let position = self.to_edge_position(a, NodeIndex::new(Ix::from_usize(id)));
            if let Some(pos) = position {
                self.node_adjacencies[pos] = Default::default();
            }

            if Ty::is_directed() {
                let position = self.to_edge_position(NodeIndex::new(Ix::from_usize(id)), a);
                if let Some(pos) = position {
                    self.node_adjacencies[pos] = Default::default();
                }
            }
        }

        self.nodes.remove(a.into_usize())
    }

    #[inline]
    fn extend_capacity_for_node(&mut self, min_node: NodeIndex<Ix>, exact: bool) {
        self.node_capacity = extend_linearized_matrix::<Ty, _>(
            &mut self.node_adjacencies,
            self.node_capacity,
            min_node.into_usize() + 1,
            exact,
        );
    }

    #[inline]
    fn extend_capacity_for_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) {
        let min_node = cmp::max(a, b);
        if min_node.into_usize() >= self.node_capacity {
            self.extend_capacity_for_node(min_node, false);
        }
    }

    /// Update the edge from `a` to `b` to the graph, with its associated data `weight`.
    ///
    /// Return the previous data, if any.
    ///
    /// Computes in **O(1)** time, best case.
    /// Computes in **O(|V|^2)** time, worst case (matrix needs to be re-allocated).
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> Option<E> {
        self.extend_capacity_for_edge(a, b);
        let p = self.to_edge_position_unchecked(a, b);
        let old_weight = mem::replace(&mut self.node_adjacencies[p], Null::new(weight));
        if old_weight.is_null() {
            self.nb_edges += 1;
        }
        old_weight.into()
    }

    /// Add an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Computes in **O(1)** time, best case.
    /// Computes in **O(|V|^2)** time, worst case (matrix needs to be re-allocated).
    ///
    /// **Panics** if any of the nodes don't exist.
    /// **Panics** if an edge already exists from `a` to `b`.
    ///
    /// **Note:** `MatrixGraph` does not allow adding parallel (“duplicate”) edges. If you want to
    /// avoid this, use [`.update_edge(a, b, weight)`](#method.update_edge) instead.
    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) {
        let old_edge_id = self.update_edge(a, b, weight);
        assert!(old_edge_id.is_none());
    }

    /// Remove the edge from `a` to `b` to the graph.
    ///
    /// **Panics** if any of the nodes don't exist.
    /// **Panics** if no edge exists between `a` and `b`.
    pub fn remove_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> E {
        let p = self
            .to_edge_position(a, b)
            .expect("No edge found between the nodes.");
        let old_weight = mem::take(&mut self.node_adjacencies[p]).into().unwrap();
        let old_weight: Option<_> = old_weight.into();
        self.nb_edges -= 1;
        old_weight.unwrap()
    }

    /// Return true if there is an edge between `a` and `b`.
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn has_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        if let Some(p) = self.to_edge_position(a, b) {
            return !self.node_adjacencies[p].is_null();
        }
        false
    }

    /// Access the weight for node `a`.
    ///
    /// Also available with indexing syntax: `&graph[a]`.
    ///
    /// **Panics** if the node doesn't exist.
    pub fn node_weight(&self, a: NodeIndex<Ix>) -> &N {
        &self.nodes[a.into_usize()]
    }

    /// Access the weight for node `a`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[a]`.
    ///
    /// **Panics** if the node doesn't exist.
    pub fn node_weight_mut(&mut self, a: NodeIndex<Ix>) -> &mut N {
        &mut self.nodes[a.into_usize()]
    }

    /// Access the weight for edge `e`.
    ///
    /// Also available with indexing syntax: `&graph[e]`.
    ///
    /// **Panics** if no edge exists between `a` and `b`.
    pub fn edge_weight(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> &E {
        let p = self
            .to_edge_position(a, b)
            .expect("No edge found between the nodes.");
        self.node_adjacencies[p]
            .as_ref()
            .expect("No edge found between the nodes.")
    }

    /// Access the weight for edge `e`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[e]`.
    ///
    /// **Panics** if no edge exists between `a` and `b`.
    pub fn edge_weight_mut(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> &mut E {
        let p = self
            .to_edge_position(a, b)
            .expect("No edge found between the nodes.");
        self.node_adjacencies[p]
            .as_mut()
            .expect("No edge found between the nodes.")
    }

    /// Return an iterator of all nodes with an edge starting from `a`.
    ///
    /// - `Directed`: Outgoing edges from `a`.
    /// - `Undirected`: All edges from or to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is [`NodeIndex<Ix>`](../graph/struct.NodeIndex.html).
    pub fn neighbors(&self, a: NodeIndex<Ix>) -> Neighbors<Ty, Null, Ix> {
        Neighbors(Edges::on_columns(
            a.into_usize(),
            &self.node_adjacencies,
            self.node_capacity,
        ))
    }

    /// Return an iterator of all edges of `a`.
    ///
    /// - `Directed`: Outgoing edges from `a`.
    /// - `Undirected`: All edges connected to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `(NodeIndex<Ix>, NodeIndex<Ix>, &E)`.
    pub fn edges(&self, a: NodeIndex<Ix>) -> Edges<Ty, Null, Ix> {
        Edges::on_columns(a.into_usize(), &self.node_adjacencies, self.node_capacity)
    }

    /// Create a new `MatrixGraph` from an iterable of edges.
    ///
    /// Node weights `N` are set to default values.
    /// Edge weights `E` may either be specified in the list,
    /// or they are filled with default values.
    ///
    /// Nodes are inserted automatically to match the edges.
    ///
    /// ```
    /// use petgraph_matrix_graph::{MatrixGraph, NodeIndex};
    ///
    /// let graph = MatrixGraph::<(), i32>::from_edges(&[
    ///     (NodeIndex::from_usize(0), NodeIndex::from_usize(1)),
    ///     (NodeIndex::from_usize(0), NodeIndex::from_usize(2)),
    ///     (NodeIndex::from_usize(0), NodeIndex::from_usize(3)),
    ///     (NodeIndex::from_usize(1), NodeIndex::from_usize(2)),
    ///     (NodeIndex::from_usize(1), NodeIndex::from_usize(3)),
    ///     (NodeIndex::from_usize(2), NodeIndex::from_usize(3)),
    /// ]);
    /// ```
    pub fn from_edges<I>(iterable: I) -> Self
    where
        I: IntoIterator,
        I::Item: IntoWeightedEdge<E>,
        <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex<Ix>>,
        N: Default,
    {
        let mut g = Self::default();
        g.extend_with_edges(iterable);
        g
    }

    /// Extend the graph from an iterable of edges.
    ///
    /// Node weights `N` are set to default values.
    /// Edge weights `E` may either be specified in the list,
    /// or they are filled with default values.
    ///
    /// Nodes are inserted automatically to match the edges.
    pub fn extend_with_edges<I>(&mut self, iterable: I)
    where
        I: IntoIterator,
        I::Item: IntoWeightedEdge<E>,
        <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex<Ix>>,
        N: Default,
    {
        for elt in iterable {
            let (source, target, weight) = elt.into_weighted_edge();
            let (source, target) = (source.into(), target.into());
            let nx = cmp::max(source, target);
            while nx.into_usize() >= self.node_count() {
                self.add_node(N::default());
            }
            self.add_edge(source, target, weight);
        }
    }
}

impl<N, E, Null: Nullable<Wrapped = E>, Ix: IndexType> MatrixGraph<N, E, Directed, Null, Ix> {
    /// Return an iterator of all neighbors that have an edge between them and
    /// `a`, in the specified direction.
    /// If the graph's edges are undirected, this is equivalent to *.neighbors(a)*.
    ///
    /// - `Outgoing`: All edges from `a`.
    /// - `Incoming`: All edges to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is [`NodeIndex<Ix>`](../graph/struct.NodeIndex.html).
    pub fn neighbors_directed(
        &self,
        a: NodeIndex<Ix>,
        d: Direction,
    ) -> Neighbors<Directed, Null, Ix> {
        if d == Direction::Outgoing {
            self.neighbors(a)
        } else {
            Neighbors(Edges::on_rows(
                a.into_usize(),
                &self.node_adjacencies,
                self.node_capacity,
            ))
        }
    }

    /// Return an iterator of all edges of `a`, in the specified direction.
    ///
    /// - `Outgoing`: All edges from `a`.
    /// - `Incoming`: All edges to `a`.
    ///
    /// Produces an empty iterator if the node `a` doesn't exist.<br>
    /// Iterator element type is `(NodeIndex<Ix>, NodeIndex<Ix>, &E)`.
    pub fn edges_directed(&self, a: NodeIndex<Ix>, d: Direction) -> Edges<Directed, Null, Ix> {
        if d == Direction::Outgoing {
            self.edges(a)
        } else {
            Edges::on_rows(a.into_usize(), &self.node_adjacencies, self.node_capacity)
        }
    }
}

/// Iterator over the node identifiers of a graph.
///
/// Created from a call to [`.node_identifiers()`][1] on a [`MatrixGraph`][2].
///
/// [1]: ../visit/trait.IntoNodeIdentifiers.html#tymethod.node_identifiers
/// [2]: struct.MatrixGraph.html
#[derive(Debug, Clone)]
pub struct NodeIdentifiers<'a, Ix> {
    iter: IdIterator<'a>,
    ix: PhantomData<Ix>,
}

impl<'a, Ix: IndexType> NodeIdentifiers<'a, Ix> {
    fn new(iter: IdIterator<'a>) -> Self {
        Self {
            iter,
            ix: PhantomData,
        }
    }
}

impl<'a, Ix: IndexType> Iterator for NodeIdentifiers<'a, Ix> {
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(Ix::from_usize).map(NodeIndex::new)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Iterator over all nodes of a graph.
///
/// Created from a call to [`.node_references()`][1] on a [`MatrixGraph`][2].
///
/// [1]: ../visit/trait.IntoNodeReferences.html#tymethod.node_references
/// [2]: struct.MatrixGraph.html
#[derive(Debug, Clone)]
pub struct NodeReferences<'a, N: 'a, Ix> {
    nodes: &'a IdStorage<N>,
    iter: IdIterator<'a>,
    ix: PhantomData<Ix>,
}

impl<'a, N: 'a, Ix> NodeReferences<'a, N, Ix> {
    fn new(nodes: &'a IdStorage<N>) -> Self {
        NodeReferences {
            nodes,
            iter: nodes.iter_ids(),
            ix: PhantomData,
        }
    }
}

impl<'a, N: 'a, Ix: IndexType> Iterator for NodeReferences<'a, N, Ix> {
    type Item = (NodeIndex<Ix>, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|i| (NodeIndex::new(Ix::from_usize(i)), &self.nodes[i]))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

/// Iterator over all edges of a graph.
///
/// Created from a call to [`.edge_references()`][1] on a [`MatrixGraph`][2].
///
/// [1]: ../visit/trait.IntoEdgeReferences.html#tymethod.edge_references
/// [2]: struct.MatrixGraph.html
#[derive(Debug, Clone)]
pub struct EdgeReferences<'a, Ty: EdgeType, Null: 'a + Nullable, Ix> {
    row: usize,
    column: usize,
    node_adjacencies: &'a [Null],
    node_capacity: usize,
    ty: PhantomData<Ty>,
    ix: PhantomData<Ix>,
}

impl<'a, Ty: EdgeType, Null: 'a + Nullable, Ix> EdgeReferences<'a, Ty, Null, Ix> {
    fn new(node_adjacencies: &'a [Null], node_capacity: usize) -> Self {
        EdgeReferences {
            row: 0,
            column: 0,
            node_adjacencies,
            node_capacity,
            ty: PhantomData,
            ix: PhantomData,
        }
    }
}

impl<'a, Ty: EdgeType, Null: Nullable, Ix: IndexType> Iterator
    for EdgeReferences<'a, Ty, Null, Ix>
{
    type Item = (NodeIndex<Ix>, NodeIndex<Ix>, &'a Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (row, column) = (self.row, self.column);
            if row >= self.node_capacity {
                return None;
            }

            // By default, advance the column. Reset and advance the row if the column overflows.
            //
            // Note that for undirected graphs, we don't want to yield the same edge twice,
            // therefore the maximum column length should be the index new after the row index.
            self.column += 1;
            let max_column_len = if !Ty::is_directed() {
                row + 1
            } else {
                self.node_capacity
            };
            if self.column >= max_column_len {
                self.column = 0;
                self.row += 1;
            }

            let p = to_linearized_matrix_position::<Ty>(row, column, self.node_capacity);
            if let Some(e) = self.node_adjacencies[p].as_ref() {
                return Some((
                    NodeIndex::new(Ix::from_usize(row)),
                    NodeIndex::new(Ix::from_usize(column)),
                    e,
                ));
            }
        }
    }
}

/// Iterator over the neighbors of a node.
///
/// Iterator element type is `NodeIndex<Ix>`.
///
/// Created with [`.neighbors()`][1], [`.neighbors_directed()`][2].
///
/// [1]: struct.MatrixGraph.html#method.neighbors
/// [2]: struct.MatrixGraph.html#method.neighbors_directed
#[derive(Debug, Clone)]
pub struct Neighbors<'a, Ty: EdgeType, Null: 'a + Nullable, Ix>(Edges<'a, Ty, Null, Ix>);

impl<'a, Ty: EdgeType, Null: Nullable, Ix: IndexType> Iterator for Neighbors<'a, Ty, Null, Ix> {
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, b, _)| b)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.0.size_hint()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NeighborIterDirection {
    Rows,
    Columns,
}

/// Iterator over the edges of from or to a node
///
/// Created with [`.edges()`][1], [`.edges_directed()`][2].
///
/// [1]: struct.MatrixGraph.html#method.edges
/// [2]: struct.MatrixGraph.html#method.edges_directed
#[derive(Debug, Clone)]
pub struct Edges<'a, Ty: EdgeType, Null: 'a + Nullable, Ix> {
    iter_direction: NeighborIterDirection,
    node_adjacencies: &'a [Null],
    node_capacity: usize,
    row: usize,
    column: usize,
    ty: PhantomData<Ty>,
    ix: PhantomData<Ix>,
}

impl<'a, Ty: EdgeType, Null: 'a + Nullable, Ix> Edges<'a, Ty, Null, Ix> {
    fn on_columns(row: usize, node_adjacencies: &'a [Null], node_capacity: usize) -> Self {
        Edges {
            iter_direction: NeighborIterDirection::Columns,
            node_adjacencies,
            node_capacity,
            row,
            column: 0,
            ty: PhantomData,
            ix: PhantomData,
        }
    }

    fn on_rows(column: usize, node_adjacencies: &'a [Null], node_capacity: usize) -> Self {
        Edges {
            iter_direction: NeighborIterDirection::Rows,
            node_adjacencies,
            node_capacity,
            row: 0,
            column,
            ty: PhantomData,
            ix: PhantomData,
        }
    }
}

impl<'a, Ty: EdgeType, Null: Nullable, Ix: IndexType> Iterator for Edges<'a, Ty, Null, Ix> {
    type Item = (NodeIndex<Ix>, NodeIndex<Ix>, &'a Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        use self::NeighborIterDirection::*;

        loop {
            let (row, column) = (self.row, self.column);
            if row >= self.node_capacity || column >= self.node_capacity {
                return None;
            }

            match self.iter_direction {
                Rows => self.row += 1,
                Columns => self.column += 1,
            }

            let p = to_linearized_matrix_position::<Ty>(row, column, self.node_capacity);
            if let Some(e) = self.node_adjacencies[p].as_ref() {
                let (a, b) = match self.iter_direction {
                    Rows => (column, row),
                    Columns => (row, column),
                };

                return Some((
                    NodeIndex::new(Ix::from_usize(a)),
                    NodeIndex::new(Ix::from_usize(b)),
                    e,
                ));
            }
        }
    }
}

#[inline]
fn to_linearized_matrix_position<Ty: EdgeType>(row: usize, column: usize, width: usize) -> usize {
    if Ty::is_directed() {
        to_flat_square_matrix_position(row, column, width)
    } else {
        to_lower_triangular_matrix_position(row, column)
    }
}

#[inline]
fn extend_linearized_matrix<Ty: EdgeType, T: Default>(
    node_adjacencies: &mut Vec<T>,
    old_node_capacity: usize,
    new_capacity: usize,
    exact: bool,
) -> usize {
    if old_node_capacity >= new_capacity {
        return old_node_capacity;
    }
    if Ty::is_directed() {
        extend_flat_square_matrix(node_adjacencies, old_node_capacity, new_capacity, exact)
    } else {
        extend_lower_triangular_matrix(node_adjacencies, new_capacity)
    }
}

#[inline]
fn to_flat_square_matrix_position(row: usize, column: usize, width: usize) -> usize {
    row * width + column
}

#[inline]
fn extend_flat_square_matrix<T: Default>(
    node_adjacencies: &mut Vec<T>,
    old_node_capacity: usize,
    new_node_capacity: usize,
    exact: bool,
) -> usize {
    // Grow the capacity by exponential steps to avoid repeated allocations.
    // Disabled for the with_capacity constructor.
    let new_node_capacity = if exact {
        new_node_capacity
    } else {
        const MIN_CAPACITY: usize = 4;
        cmp::max(new_node_capacity.next_power_of_two(), MIN_CAPACITY)
    };

    // Optimization: when resizing the matrix this way we skip the first few grows to make
    // small matrices a bit faster to work with.

    ensure_len(node_adjacencies, new_node_capacity.pow(2));
    for c in (1..old_node_capacity).rev() {
        let pos = c * old_node_capacity;
        let new_pos = c * new_node_capacity;
        // Move the slices directly if they do not overlap with their new position
        if pos + old_node_capacity <= new_pos {
            debug_assert!(pos + old_node_capacity < node_adjacencies.len());
            debug_assert!(new_pos + old_node_capacity < node_adjacencies.len());
            let ptr = node_adjacencies.as_mut_ptr();
            // SAFETY: pos + old_node_capacity <= new_pos, so this won't overlap
            unsafe {
                let old = ptr.add(pos);
                let new = ptr.add(new_pos);
                core::ptr::swap_nonoverlapping(old, new, old_node_capacity);
            }
        } else {
            for i in (0..old_node_capacity).rev() {
                node_adjacencies.as_mut_slice().swap(pos + i, new_pos + i);
            }
        }
    }

    new_node_capacity
}

#[inline]
fn to_lower_triangular_matrix_position(row: usize, column: usize) -> usize {
    let (row, column) = if row > column {
        (row, column)
    } else {
        (column, row)
    };
    (row * (row + 1)) / 2 + column
}

#[inline]
fn extend_lower_triangular_matrix<T: Default>(
    node_adjacencies: &mut Vec<T>,
    new_capacity: usize,
) -> usize {
    let max_node = new_capacity - 1;
    let max_pos = to_lower_triangular_matrix_position(max_node, max_node);
    ensure_len(node_adjacencies, max_pos + 1);
    new_capacity
}

/// Grow a Vec by appending the type's default value until the `size` is reached.
fn ensure_len<T: Default>(v: &mut Vec<T>, size: usize) {
    v.resize_with(size, T::default);
}

#[derive(Debug, Clone)]
struct IdStorage<T> {
    elements: Vec<Option<T>>,
    upper_bound: usize,
    removed_ids: IndexSet<usize>,
}

impl<T> IdStorage<T> {
    fn with_capacity(capacity: usize) -> Self {
        IdStorage {
            elements: Vec::with_capacity(capacity),
            upper_bound: 0,
            removed_ids: IndexSet::new(),
        }
    }

    fn add(&mut self, element: T) -> usize {
        let id = if let Some(id) = self.removed_ids.pop() {
            id
        } else {
            let id = self.upper_bound;
            self.upper_bound += 1;

            ensure_len(&mut self.elements, id + 1);

            id
        };

        self.elements[id] = Some(element);

        id
    }

    fn remove(&mut self, id: usize) -> T {
        let data = self.elements[id].take().unwrap();
        if self.upper_bound - id == 1 {
            self.upper_bound -= 1;
        } else {
            self.removed_ids.insert(id);
        }
        data
    }

    fn clear(&mut self) {
        self.upper_bound = 0;
        self.elements.clear();
        self.removed_ids.clear();
    }

    #[inline]
    fn len(&self) -> usize {
        self.upper_bound - self.removed_ids.len()
    }

    fn iter_ids(&self) -> IdIterator {
        IdIterator {
            upper_bound: self.upper_bound,
            removed_ids: &self.removed_ids,
            current: None,
        }
    }
}

impl<T> Index<usize> for IdStorage<T> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.elements[index].as_ref().unwrap()
    }
}

impl<T> IndexMut<usize> for IdStorage<T> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.elements[index].as_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
struct IdIterator<'a> {
    upper_bound: usize,
    removed_ids: &'a IndexSet<usize>,
    current: Option<usize>,
}

impl<'a> Iterator for IdIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        // initialize / advance
        let current = {
            if self.current.is_none() {
                self.current = Some(0);
                self.current.as_mut().unwrap()
            } else {
                let current = self.current.as_mut().unwrap();
                *current += 1;
                current
            }
        };

        // skip removed ids
        while self.removed_ids.contains(current) && *current < self.upper_bound {
            *current += 1;
        }

        if *current < self.upper_bound {
            Some(*current)
        } else {
            None
        }
    }
}

/// Create a new empty `MatrixGraph`.
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> Default
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<N, E> MatrixGraph<N, E, Directed> {
    /// Create a new `MatrixGraph` with directed edges.
    ///
    /// This is a convenience method. Use `MatrixGraph::with_capacity` or `MatrixGraph::default` for
    /// a constructor that is generic in all the type parameters of `MatrixGraph`.
    pub fn new() -> Self {
        MatrixGraph::default()
    }
}

impl<N, E> MatrixGraph<N, E, Undirected> {
    /// Create a new `MatrixGraph` with undirected edges.
    ///
    /// This is a convenience method. Use `MatrixGraph::with_capacity` or `MatrixGraph::default` for
    /// a constructor that is generic in all the type parameters of `MatrixGraph`.
    pub fn new_undirected() -> Self {
        MatrixGraph::default()
    }
}

/// Index the `MatrixGraph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> Index<NodeIndex<Ix>>
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    type Output = N;

    fn index(&self, ax: NodeIndex<Ix>) -> &N {
        self.node_weight(ax)
    }
}

/// Index the `MatrixGraph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> IndexMut<NodeIndex<Ix>>
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    fn index_mut(&mut self, ax: NodeIndex<Ix>) -> &mut N {
        self.node_weight_mut(ax)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> NodeCount
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    fn node_count(&self) -> usize {
        MatrixGraph::node_count(self)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> EdgeCount
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    #[inline]
    fn edge_count(&self) -> usize {
        self.edge_count()
    }
}

/// Index the `MatrixGraph` by `NodeIndex` pair to access edge weights.
///
/// Also available with indexing syntax: `&graph[e]`.
///
/// **Panics** if no edge exists between `a` and `b`.
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType>
    Index<(NodeIndex<Ix>, NodeIndex<Ix>)> for MatrixGraph<N, E, Ty, Null, Ix>
{
    type Output = E;

    fn index(&self, (ax, bx): (NodeIndex<Ix>, NodeIndex<Ix>)) -> &E {
        self.edge_weight(ax, bx)
    }
}

/// Index the `MatrixGraph` by `NodeIndex` pair to access edge weights.
///
/// Also available with indexing syntax: `&mut graph[e]`.
///
/// **Panics** if no edge exists between `a` and `b`.
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType>
    IndexMut<(NodeIndex<Ix>, NodeIndex<Ix>)> for MatrixGraph<N, E, Ty, Null, Ix>
{
    fn index_mut(&mut self, (ax, bx): (NodeIndex<Ix>, NodeIndex<Ix>)) -> &mut E {
        self.edge_weight_mut(ax, bx)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> GetAdjacencyMatrix
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    type AdjMatrix = ();

    fn adjacency_matrix(&self) -> Self::AdjMatrix {}

    fn is_adjacent(&self, _: &Self::AdjMatrix, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        MatrixGraph::has_edge(self, a, b)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> Visitable
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    type Map = FixedBitSet;

    fn visit_map(&self) -> FixedBitSet {
        FixedBitSet::with_capacity(self.node_bound())
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
        map.grow(self.node_bound());
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> GraphBase
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    type EdgeId = (NodeIndex<Ix>, NodeIndex<Ix>);
    type NodeId = NodeIndex<Ix>;
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> GraphProp
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    type EdgeType = Ty;
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> Data
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    type EdgeWeight = E;
    type NodeWeight = N;
}

impl<'a, N, E: 'a, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> IntoNodeIdentifiers
    for &'a MatrixGraph<N, E, Ty, Null, Ix>
{
    type NodeIdentifiers = NodeIdentifiers<'a, Ix>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIdentifiers::new(self.nodes.iter_ids())
    }
}

impl<'a, N, E: 'a, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> IntoNeighbors
    for &'a MatrixGraph<N, E, Ty, Null, Ix>
{
    type Neighbors = Neighbors<'a, Ty, Null, Ix>;

    fn neighbors(self, a: NodeIndex<Ix>) -> Self::Neighbors {
        MatrixGraph::neighbors(self, a)
    }
}

impl<'a, N, E: 'a, Null: Nullable<Wrapped = E>, Ix: IndexType> IntoNeighborsDirected
    for &'a MatrixGraph<N, E, Directed, Null, Ix>
{
    type NeighborsDirected = Neighbors<'a, Directed, Null, Ix>;

    fn neighbors_directed(self, a: NodeIndex<Ix>, d: Direction) -> Self::NeighborsDirected {
        MatrixGraph::neighbors_directed(self, a, d)
    }
}

impl<'a, N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> IntoNodeReferences
    for &'a MatrixGraph<N, E, Ty, Null, Ix>
{
    type NodeRef = (NodeIndex<Ix>, &'a N);
    type NodeReferences = NodeReferences<'a, N, Ix>;

    fn node_references(self) -> Self::NodeReferences {
        NodeReferences::new(&self.nodes)
    }
}

impl<'a, N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> IntoEdgeReferences
    for &'a MatrixGraph<N, E, Ty, Null, Ix>
{
    type EdgeRef = (NodeIndex<Ix>, NodeIndex<Ix>, &'a E);
    type EdgeReferences = EdgeReferences<'a, Ty, Null, Ix>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences::new(&self.node_adjacencies, self.node_capacity)
    }
}

impl<'a, N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> IntoEdges
    for &'a MatrixGraph<N, E, Ty, Null, Ix>
{
    type Edges = Edges<'a, Ty, Null, Ix>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        MatrixGraph::edges(self, a)
    }
}

impl<'a, N, E, Null: Nullable<Wrapped = E>, Ix: IndexType> IntoEdgesDirected
    for &'a MatrixGraph<N, E, Directed, Null, Ix>
{
    type EdgesDirected = Edges<'a, Directed, Null, Ix>;

    fn edges_directed(self, a: Self::NodeId, dir: Direction) -> Self::EdgesDirected {
        MatrixGraph::edges_directed(self, a, dir)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> NodeIndexable
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    fn node_bound(&self) -> usize {
        self.nodes.upper_bound
    }

    fn to_index(&self, ix: NodeIndex<Ix>) -> usize {
        ix.into_usize()
    }

    fn from_index(&self, ix: usize) -> Self::NodeId {
        NodeIndex::new(Ix::from_usize(ix))
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped = E>, Ix: IndexType> Build
    for MatrixGraph<N, E, Ty, Null, Ix>
{
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId {
        self.add_node(weight)
    }

    fn add_edge(
        &mut self,
        a: Self::NodeId,
        b: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Option<Self::EdgeId> {
        if !self.has_edge(a, b) {
            MatrixGraph::update_edge(self, a, b, weight);
            Some((a, b))
        } else {
            None
        }
    }

    fn update_edge(
        &mut self,
        a: Self::NodeId,
        b: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Self::EdgeId {
        MatrixGraph::update_edge(self, a, b, weight);
        (a, b)
    }
}
