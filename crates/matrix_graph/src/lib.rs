//! `MatrixGraph<N, E, Ty, NullN, NullE, Ix>` is a graph datastructure backed by an adjacency
//! matrix.

extern crate alloc;

use alloc::{fmt, vec, vec::Vec};
use core::{cmp, hash::BuildHasher, marker::PhantomData, mem};
use std::fmt::Display;

use foldhash::fast::RandomState;
use indexmap::IndexSet;
use petgraph_core::id::Id;

use crate::private::Sealed;

mod directed;
mod undirected;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Directed;

impl Display for Directed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Directed")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Undirected;

impl Display for Undirected {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Undirected")
    }
}

/// Node index type for the `MatrixGraph`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeId(usize);

impl Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node({})", self.0)
    }
}

impl Id for NodeId {}

impl NodeId {
    const MAX: Self = NodeId(usize::MAX);
}

impl From<usize> for NodeId {
    fn from(value: usize) -> Self {
        NodeId(value)
    }
}

impl From<u32> for NodeId {
    fn from(value: u32) -> Self {
        NodeId(value as usize)
    }
}

/// Edge index type for `MatrixGraph`.
///
/// Contains the two node indices. If the graph is directed, node1 is the source and node2 is
/// the target. If the graph is undirected, the order of the nodes is irrelevant and traits such
/// as `PartialEq` and `Hash` are implemented accordingly.
#[derive(Debug, Clone, Copy)]
pub struct EdgeId<Dir> {
    node1: NodeId,
    node2: NodeId,
    direction: PhantomData<Dir>,
}

pub use directed::DirEdgeId;
pub use undirected::UndirEdgeId;

impl<Dir> Display for EdgeId<Dir> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edge({}, {})", self.node1.0, self.node2.0)
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
/// [`MatrixGraph`](struct.MatrixGraph.html), replacing the default `Option<E>` sentinel.
///
/// Pre-requisite: edge data should implement [`Zero`](trait.Zero.html).
///
/// Note that if you're already using the standard non-zero types (such as `NonZeroU32`), you don't
/// have to use this wrapper and can leave the default `Null` type argument.
pub struct NotZero<T>(T);

impl<T: Zeroable> Default for NotZero<T> {
    fn default() -> Self {
        NotZero(T::zero())
    }
}

impl<T: Zeroable> NicheWrapper for NotZero<T> {
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

impl<T: Zeroable> From<NotZero<T>> for Option<T> {
    fn from(not_zero: NotZero<T>) -> Self {
        if !not_zero.is_null() {
            Some(not_zero.0)
        } else {
            None
        }
    }
}

/// Base trait for types that have a zero-like value and can thus be wrapped in a
/// [`NotZero`](struct.NotZero.html).
///
/// Implementors must provide a singleton zero-like object that will be used to mark empty edges in
/// a [`MatrixGraph`](struct.MatrixGraph.html).
///
/// Note that this trait is already implemented for the base numeric types.
pub trait Zeroable {
    /// Return the singleton object which can be used as a sentinel value.
    fn zero() -> Self;

    /// Return true if `self` is equal to the sentinel value.
    fn is_zero(&self) -> bool;
}

macro_rules! zeroable_impl {
    ($t:ty, $z:expr) => {
        impl Zeroable for $t {
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

macro_rules! zeroable_impls {
    ($($t:ty),*) => {
        $(
            zeroable_impl!($t, 0);
        )*
    }
}

zeroable_impls!(u8, u16, u32, u64, usize);
zeroable_impls!(i8, i16, i32, i64, isize);
zeroable_impls!(f32, f64);

/// The error type for fallible `MatrixGraph` operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixError {
    /// The node with the specified index is missing from the graph.
    NodeMissed(usize),
}

pub trait MatrixGraphExtras<N>: Sealed {
    fn to_edge_position(&self, a: NodeId, b: NodeId) -> Option<usize>;
    fn to_edge_position_unchecked(&self, a: NodeId, b: NodeId) -> usize;
    fn extend_capacity_for_node(&mut self, new_node_capacity: usize, exact: bool);
    fn remove_node(&mut self, a: NodeId) -> N;
}

/// `MatrixGraph<N, E, S, Null, Ty>` is a graph using an adjacency matrix representation.
///
/// It uses a flattened 2D array to store edge data, and a separate storage for node data and
/// management of node indices. The graph supports both directed and undirected edges.
///
/// Edge data can be of arbitrary type that has a notion of "null" or "empty" value, which is used
/// to mark the absence of an edge in the graph and allows for better memory optimization.
///
/// `MatrixGraph` is parameterized over:
/// - Associated data `N` for nodes and `E` for edges. The associated data can be of arbitrary type.
/// - Hasher type `S` that determines how node indices are hashed (defaults to `RandomState`). This
///   is used to keep track of removed node indices and reuse them when adding new nodes, thus
///   optimizing memory usage.
/// - Nullable type `Null`, which denotes the edges' presence (defaults to `Option<E>`). You may
///   specify [`NotZero<E>`](struct.NotZero.html) if you want to use a sentinel value (such as 0) to
///   mark the absence of an edge.
/// - Edge type `Ty` determines whether the graph edges are directed or undirected.
///
/// The graph uses **O(|V^2|)** space, with fast edge insertion & amortized node insertion, as well
/// as efficient graph search and graph algorithms on dense graphs.
///
/// For undirected graphs, only the lower triangular part of the adjacency matrix is stored. Since
/// the backing array stores edge data, it is recommended to box large edge data.
///
/// The graph uses [`NodeId`] and [`EdgeId`] as node and edge indices. Node indices are convertible
/// to `usize`, however not guaranteed to be contiguous. When removing nodes, the graph will reuse
/// the indices of removed nodes for new nodes filling in the gaps. For most use cases however, the
/// graph is assumed to have dense node indices.
#[derive(Clone)]
pub struct MatrixGraph<
    N,
    E,
    S = RandomState,
    Null: NicheWrapper<Wrapped = E> = Option<E>,
    Dir = Directed,
> {
    /// Edge Data including presence information.
    flattened_edge_data: Vec<Null>,
    /// Node data and management of node indices.
    node_data: IdStorage<N, S>,
    /// The current edge capacity with respect to the number of nodes. This is used to determine
    /// when the backing matrix needs to be resized.
    node_capacity: usize,
    /// The number of edges currently in the graph.
    edge_count: usize,
    directionality: PhantomData<Dir>,
}

/// A [`MatrixGraph`] with directed edges.
pub type DiMatrix<N, E, S = RandomState, Null = Option<E>> = MatrixGraph<N, E, S, Null, Directed>;

/// A [`MatrixGraph`] with undirected edges.
pub type UnMatrix<N, E, S = RandomState, Null = Option<E>> = MatrixGraph<N, E, S, Null, Undirected>;

impl<N, E, S: BuildHasher, Null: NicheWrapper<Wrapped = E>, Dir> MatrixGraph<N, E, S, Null, Dir> {
    /// Remove all nodes and edges.
    pub fn clear(&mut self) {
        for edge in self.flattened_edge_data.iter_mut() {
            *edge = Default::default();
        }
        self.node_data.clear();
        self.edge_count = 0;
    }

    /// Add a node (also called vertex) with associated data to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Returns the index of the new node.
    ///
    /// **Panics** if the MatrixGraph contains usize::MAX nodes already.
    #[track_caller]
    pub fn add_node(&mut self, data: N) -> NodeId {
        NodeId(self.node_data.add(data))
    }
}

impl<N, E, S: BuildHasher, Null: NicheWrapper<Wrapped = E>, Dir> MatrixGraph<N, E, S, Null, Dir>
where
    MatrixGraph<N, E, S, Null, Dir>: MatrixGraphExtras<N>,
{
    /// Create a new `MatrixGraph` with estimated capacity for nodes.
    pub fn with_capacity(node_capacity: usize) -> Self
    where
        S: Default,
    {
        Self::with_capacity_and_hasher(node_capacity, Default::default())
    }

    /// Create a new `MatrixGraph` which has the specified hasher and capacity sufficient
    /// to hold the specified number of nodes (and any edges between them) without resizing.
    pub fn with_capacity_and_hasher(node_capacity: usize, hasher: S) -> Self {
        let mut m = Self {
            flattened_edge_data: vec![],
            node_capacity: 0,
            node_data: IdStorage::with_capacity_and_hasher(node_capacity, hasher),
            edge_count: 0,
            directionality: PhantomData,
        };

        assert!(
            node_capacity <= NodeId::MAX.0,
            "Node capacity cannot exceed maximum NodeId value"
        );
        if node_capacity > 0 {
            m.extend_capacity_for_node(node_capacity, true);
        }

        m
    }

    #[inline]
    fn extend_capacity_for_edge(&mut self, a: NodeId, b: NodeId) {
        let min_node = cmp::max(a, b);
        if min_node.0 >= self.node_capacity {
            self.extend_capacity_for_node(min_node.0 + 1, false);
        }
    }

    /// Remove node `a` from the graph.
    ///
    /// Computes in **O(V)** time, due to the removal of edges with other nodes.
    ///
    /// **Panics** if the node `a` does not exist.
    #[track_caller]
    pub fn remove_node(&mut self, a: NodeId) -> N {
        <Self as MatrixGraphExtras<N>>::remove_node(self, a)
    }

    /// Update the edge from `a` to `b` to the graph, with its associated data `weight`.
    ///
    /// Return the previous data, if any.
    ///
    /// Computes in **O(1)** time, best case.
    /// Computes in **O(|V|^2)** time, worst case (matrix needs to be re-allocated).
    ///
    /// **Panics** if any of the nodes don't exist.
    #[track_caller]
    pub fn update_edge(&mut self, a: NodeId, b: NodeId, weight: E) -> Option<E> {
        self.extend_capacity_for_edge(a, b);
        let p = self.to_edge_position_unchecked(a, b);
        let old_weight = mem::replace(&mut self.flattened_edge_data[p], Null::new(weight));
        if old_weight.is_null() {
            self.edge_count += 1;
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
    #[track_caller]
    pub fn add_edge(&mut self, a: NodeId, b: NodeId, weight: E) {
        let old_edge_id = self.update_edge(a, b, weight);
        assert!(old_edge_id.is_none());
    }

    /// Remove the edge from `a` to `b` to the graph.
    ///
    /// **Panics** if any of the nodes don't exist.
    /// **Panics** if no edge exists between `a` and `b`.
    #[track_caller]
    pub fn remove_edge(&mut self, a: NodeId, b: NodeId) -> E {
        let p = self
            .to_edge_position(a, b)
            .expect("No edge found between the nodes.");
        let old_weight = mem::take(&mut self.flattened_edge_data[p]).into().unwrap();
        let old_weight: Option<_> = old_weight.into();
        self.edge_count -= 1;
        old_weight.unwrap()
    }
}

/// Grow a Vec by appending the type's default value until the `size` is reached.
fn ensure_len<T: Default>(v: &mut Vec<T>, size: usize) {
    v.resize_with(size, T::default);
}

#[derive(Debug, Clone)]
struct IdStorage<T, S = RandomState> {
    /// The elements of the storage. An element is `None` if and only if the corresponding node has
    /// been removed.
    elements: Vec<Option<T>>,
    /// The current highest index that has ever been used in the storage. Used to determine
    /// the index of a new node.
    upper_bound: usize,
    /// The set of removed node indices.
    removed_ids: IndexSet<usize, S>,
}

impl<T, S: BuildHasher> IdStorage<T, S> {
    fn with_capacity_and_hasher(capacity: usize, hasher: S) -> Self {
        IdStorage {
            elements: Vec::with_capacity(capacity),
            upper_bound: 0,
            removed_ids: IndexSet::with_hasher(hasher),
        }
    }

    fn get(&self, id: usize) -> Option<&T> {
        self.elements.get(id)?.as_ref()
    }

    fn get_mut(&mut self, id: usize) -> Option<&mut T> {
        self.elements.get_mut(id)?.as_mut()
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

    /// Remove all elements from the storage.
    fn clear(&mut self) {
        self.upper_bound = 0;
        self.elements.clear();
        self.removed_ids.clear();
    }

    /// Returns the number of existing elements in the storage.
    #[inline]
    fn len(&self) -> usize {
        self.upper_bound - self.removed_ids.len()
    }

    /// Returns an iterator over the (NodeId, data) pairs in the storage / graph for all existing
    /// nodes.
    fn iter(&self) -> impl Iterator<Item = (NodeId, &T)> {
        self.elements
            .iter()
            .enumerate()
            .filter_map(|(id, element)| {
                if let Some(element) = element {
                    Some((NodeId(id), element))
                } else {
                    None
                }
            })
    }

    /// Returns an iterator over the (NodeId, data) pairs in the storage / graph for all existing
    /// nodes, with mutable access to the data.
    fn iter_mut(&mut self) -> impl Iterator<Item = (NodeId, &mut T)> {
        self.elements
            .iter_mut()
            .enumerate()
            .filter_map(|(id, element)| {
                if let Some(element) = element {
                    Some((NodeId(id), element))
                } else {
                    None
                }
            })
    }
}

impl<N, E, S: BuildHasher + Default, Null: NicheWrapper<Wrapped = E>, Dir> Default
    for MatrixGraph<N, E, S, Null, Dir>
where
    MatrixGraph<N, E, S, Null, Dir>: MatrixGraphExtras<N>,
{
    fn default() -> Self {
        Self::with_capacity(0)
    }
}

impl<N, E> MatrixGraph<N, E, RandomState, Option<E>, Directed> {
    /// Create a new `MatrixGraph` with directed edges.
    ///
    /// This is a convenience method. Use `MatrixGraph::with_capacity` or `MatrixGraph::default` for
    /// a constructor that is generic in all the type parameters of `MatrixGraph`.
    pub fn new() -> Self {
        MatrixGraph::default()
    }
}

impl<N, E> MatrixGraph<N, E, RandomState, Option<E>, Undirected> {
    /// Create a new `MatrixGraph` with undirected edges.
    ///
    /// This is a convenience method. Use `MatrixGraph::with_capacity` or `MatrixGraph::default` for
    /// a constructor that is generic in all the type parameters of `MatrixGraph`.
    pub fn new_undirected() -> Self {
        MatrixGraph::default()
    }
}

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
            Either::Left(l) => l.next(),
            Either::Right(r) => r.next(),
        }
    }
}

#[cfg(test)]
mod tests {
    use petgraph_core::graph::DirectedGraph;

    use super::*;

    #[test]
    fn test_new() {
        let g = MatrixGraph::<i32, i32>::new();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_id_storage() {
        let mut storage: IdStorage<char> =
            IdStorage::with_capacity_and_hasher(0, Default::default());
        let a = storage.add('a');
        let b = storage.add('b');
        let c = storage.add('c');

        assert!(a < b && b < c);

        assert_eq!(
            storage.iter().map(|(id, _)| id.0).collect::<Vec<_>>(),
            vec![a, b, c]
        );

        storage.remove(b);

        let bb = storage.add('B');
        assert_eq!(b, bb);

        // list IDs
        assert_eq!(
            storage.iter().map(|(id, _)| id.0).collect::<Vec<_>>(),
            vec![a, b, c]
        );
    }
}
