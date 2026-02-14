//! `MatrixGraph<N, E, Ty, NullN, NullE, Ix>` is a graph datastructure backed by an adjacency
//! matrix.

extern crate alloc;

use alloc::{fmt, vec, vec::Vec};
use core::{
    cmp,
    hash::BuildHasher,
    marker::PhantomData,
    mem,
    ops::{Index, IndexMut},
};
use std::fmt::Display;

use foldhash::fast::RandomState;
use indexmap::IndexSet;
use petgraph_core::{
    edge::{EdgeMut, EdgeRef},
    graph::{Cardinality, DensityHint, DirectedGraph, Graph},
    id::Id,
    node::{NodeMut, NodeRef},
};

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

// The following types are used to control the max size of the adjacency matrix. Since the maximum
// size of the matrix vector's is the square of the maximum number of nodes, the number of nodes
// should be reasonably picked.
type DefaultIx = u16;

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

/// Edge index type for `MatrixGraph`.
///
/// Contains the two node indices. If the graph is directed, the node1 is the source and node2 is
/// the target. If the graph is undirected, the order of the nodes is not relevant.
#[derive(Debug, Clone, Copy, Hash)]
pub struct EdgeId<Dir> {
    node1: NodeId,
    node2: NodeId,
    direction: PhantomData<Dir>,
}

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

/// `NotZero` is used to optimize the memory usage of edge data `E` in a
/// [`MatrixGraph`](struct.MatrixGraph.html), replacing the default `Option<E>` sentinel.
///
/// Pre-requisite: edge data should implement [`Zero`](trait.Zero.html).
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

/// The error type for fallible `MatrixGraph` operations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MatrixError {
    /// The `MatrixGraph` is at the maximum number of nodes for its index.
    NodeIxLimit,

    /// The node with the specified index is missing from the graph.
    NodeMissed(usize),
}

impl core::error::Error for MatrixError {}

impl fmt::Display for MatrixError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MatrixError::NodeIxLimit => write!(
                f,
                "The MatrixGraph is at the maximum number of nodes for its index"
            ),

            MatrixError::NodeMissed(i) => {
                write!(f, "The node with index {i} is missing from the graph.")
            }
        }
    }
}

pub trait MatrixGraphExtras<N>: Sealed {
    fn to_edge_position(&self, a: NodeId, b: NodeId) -> Option<usize>;
    fn to_edge_position_unchecked(&self, a: NodeId, b: NodeId) -> usize;
    fn extend_capacity_for_node(&mut self, new_node_capacity: usize, exact: bool);
    fn remove_node(&mut self, a: NodeId) -> N;
}

/// `MatrixGraph<N, E, Ty, Null>` is a graph datastructure using an adjacency matrix
/// representation.
///
/// `MatrixGraph` is parameterized over:
///
/// - Associated data `N` for nodes and `E` for edges, called *data*. The associated data can be of
///   arbitrary type.
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
/// matrix is stored. Since the backing array stores edge data, it is recommended to box large
/// edge data.
#[derive(Clone)]
pub struct MatrixGraph<
    N,
    E,
    S = RandomState,
    Null: Nullable<Wrapped = E> = Option<E>,
    Dir = Directed,
> {
    node_adjacencies: Vec<Null>,
    node_capacity: usize,
    nodes: IdStorage<N, S>,
    edge_count: usize,
    directionality: PhantomData<Dir>,
}

/// A `MatrixGraph` with directed edges.
pub type DiMatrix<N, E, S = RandomState, Null = Option<E>> = MatrixGraph<N, E, S, Null, Directed>;

/// A `MatrixGraph` with undirected edges.
pub type UnMatrix<N, E, S = RandomState, Null = Option<E>> = MatrixGraph<N, E, S, Null, Undirected>;

impl<N, E, S: BuildHasher, Null: Nullable<Wrapped = E>, Dir> MatrixGraph<N, E, S, Null, Dir> {
    /// Remove all nodes and edges.
    pub fn clear(&mut self) {
        for edge in self.node_adjacencies.iter_mut() {
            *edge = Default::default();
        }
        self.nodes.clear();
        self.edge_count = 0;
    }

    /// Return the number of nodes (also called vertices) in the graph.
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
        self.edge_count
    }

    /// Add a node (also called vertex) with associated data to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Return the index of the new node.
    ///
    /// **Panics** if the MatrixGraph is at the maximum number of nodes for its index type.
    #[track_caller]
    pub fn add_node(&mut self, data: N) -> NodeId {
        NodeId(self.nodes.add(data))
    }

    /// Try to add a node (also called vertex) with associated data to the graph.
    ///
    /// Computes in **O(1)** time.
    ///
    /// Return the index of the new node.
    ///
    /// Possible errors:
    /// - [`MatrixError::NodeIxLimit`] if the `MatrixGraph` is at the maximum number of nodes for
    ///   its index type.
    pub fn try_add_node(&mut self, data: N) -> Result<NodeId, MatrixError> {
        let node_idx = NodeId(self.nodes.len());
        // TODO: Check whether the first condition here makes sense
        if !(NodeId::MAX.0 == !0 || NodeId::MAX != node_idx) {
            return Err(MatrixError::NodeIxLimit);
        }
        Ok(NodeId(self.nodes.add(data)))
    }

    /// Access the data for node `a`.
    ///
    /// Also available with indexing syntax: `&graph[a]`.
    ///
    /// **Panics** if the node doesn't exist.
    #[track_caller]
    pub fn node_weight(&self, a: NodeId) -> &N {
        &self.nodes[a.0]
    }

    /// Try to access the weight for node `a`.
    ///
    /// Return `None` if the node doesn't exist.
    pub fn get_node_weight(&self, a: NodeId) -> Option<&N> {
        self.nodes.elements.get(a.0)?.as_ref()
    }

    /// Access the weight for node `a`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[a]`.
    ///
    /// **Panics** if the node doesn't exist.
    #[track_caller]
    pub fn node_weight_mut(&mut self, a: NodeId) -> &mut N {
        &mut self.nodes[a.0]
    }

    /// Try to access the weight for node `a`, mutably.
    ///
    /// Return `None` if the node doesn't exist.
    pub fn get_node_weight_mut(&mut self, a: NodeId) -> Option<&mut N> {
        self.nodes.elements.get_mut(a.0)?.as_mut()
    }

    fn assert_node_bounds(&self, a: NodeId, b: NodeId) -> Result<(), MatrixError> {
        if a.0 >= self.node_capacity {
            Err(MatrixError::NodeMissed(a.0))
        } else if b.0 >= self.node_capacity {
            Err(MatrixError::NodeMissed(b.0))
        } else {
            Ok(())
        }
    }
}

impl<N, E, S: BuildHasher, Null: Nullable<Wrapped = E>, Dir> MatrixGraph<N, E, S, Null, Dir>
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

    /// Create a new `MatrixGraph` with estimated capacity for nodes and a provided hasher.
    pub fn with_capacity_and_hasher(node_capacity: usize, hasher: S) -> Self {
        let mut m = Self {
            node_adjacencies: vec![],
            node_capacity: 0,
            nodes: IdStorage::with_capacity_and_hasher(node_capacity, hasher),
            edge_count: 0,
            directionality: PhantomData,
        };

        assert!(
            node_capacity <= usize::MAX,
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

    /// Remove `a` from the graph.
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
        let old_weight = mem::replace(&mut self.node_adjacencies[p], Null::new(weight));
        if old_weight.is_null() {
            self.edge_count += 1;
        }
        old_weight.into()
    }

    /// Try to update the edge from `a` to `b`, with its associated data `weight`.
    ///
    /// Return the previous data, if any.
    ///
    /// Computes in **O(1)** time, best case.
    /// Computes in **O(|V|^2)** time, worst case (matrix needs to be re-allocated).
    ///
    /// Possible errors:
    /// - [`MatrixError::NodeMissed`] if any of the nodes don't exist.
    pub fn try_update_edge(
        &mut self,
        a: NodeId,
        b: NodeId,
        weight: E,
    ) -> Result<Option<E>, MatrixError> {
        self.assert_node_bounds(a, b)?;
        Ok(self.update_edge(a, b, weight))
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

    /// Add or update edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the previous data, if any.
    ///
    /// Computes in **O(1)** time, best case.
    /// Computes in **O(|V|^2)** time, worst case (matrix needs to be re-allocated).
    ///
    /// Possible errors:
    /// - [`MatrixError::NodeMissed`] if any of the nodes don't exist.
    pub fn add_or_update_edge(
        &mut self,
        a: NodeId,
        b: NodeId,
        weight: E,
    ) -> Result<Option<E>, MatrixError> {
        self.extend_capacity_for_edge(a, b);
        self.try_update_edge(a, b, weight)
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
        let old_weight = mem::take(&mut self.node_adjacencies[p]).into().unwrap();
        let old_weight: Option<_> = old_weight.into();
        self.edge_count -= 1;
        old_weight.unwrap()
    }

    /// Try to remove the edge from `a` to `b`.
    ///
    /// Return old value if present.
    pub fn try_remove_edge(&mut self, a: NodeId, b: NodeId) -> Option<E> {
        let p = self.to_edge_position(a, b)?;
        if let Some(entry) = self.node_adjacencies.get_mut(p) {
            let old_weight = mem::take(entry).into()?;
            self.edge_count -= 1;
            return Some(old_weight);
        }
        None
    }

    /// Return `true` if there is an edge between `a` and `b`.
    ///
    /// If any of the nodes don't exist - returns `false`.
    /// **Panics** if any of the nodes don't exist.
    #[track_caller]
    pub fn has_edge(&self, a: NodeId, b: NodeId) -> bool {
        if let Some(p) = self.to_edge_position(a, b) {
            return self
                .node_adjacencies
                .get(p)
                .map(|e| !e.is_null())
                .unwrap_or(false);
        }
        false
    }

    /// Access the weight for edge `e`.
    ///
    /// Also available with indexing syntax: `&graph[e]`.
    ///
    /// **Panics** if no edge exists between `a` and `b`.
    #[track_caller]
    pub fn edge_weight(&self, a: NodeId, b: NodeId) -> &E {
        let p = self
            .to_edge_position(a, b)
            .expect("No edge found between the nodes.");
        self.node_adjacencies[p]
            .as_ref()
            .expect("No edge found between the nodes.")
    }

    /// Access the weight for edge from `a` to `b`.
    ///
    /// Return `None` if the edge doesn't exist.
    pub fn get_edge_weight(&self, a: NodeId, b: NodeId) -> Option<&E> {
        let p = self.to_edge_position(a, b)?;
        self.node_adjacencies.get(p)?.as_ref()
    }

    /// Access the weight for edge `e`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[e]`.
    ///
    /// **Panics** if no edge exists between `a` and `b`.
    #[track_caller]
    pub fn edge_weight_mut(&mut self, a: NodeId, b: NodeId) -> &mut E {
        let p = self
            .to_edge_position(a, b)
            .expect("No edge found between the nodes.");
        self.node_adjacencies[p]
            .as_mut()
            .expect("No edge found between the nodes.")
    }

    /// Access the weight for edge from `a` to `b`, mutably.
    ///
    /// Return `None` if the edge doesn't exist.
    pub fn get_edge_weight_mut(&mut self, a: NodeId, b: NodeId) -> Option<&mut E> {
        let p = self.to_edge_position(a, b)?;
        self.node_adjacencies.get_mut(p)?.as_mut()
    }
}

/// Grow a Vec by appending the type's default value until the `size` is reached.
fn ensure_len<T: Default>(v: &mut Vec<T>, size: usize) {
    v.resize_with(size, T::default);
}

struct EdgeIterator<'a, It: Iterator<Item = &'a Null>, Null: Nullable + 'a> {
    edges: It,
    current_edge_tuple: (usize, usize),
    node_capacity: usize,
}

impl<'a, It: Iterator<Item = &'a Null>, Null: Nullable> Iterator for EdgeIterator<'a, It, Null> {
    type Item = (NodeId, NodeId, &'a Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(edge) = self.edges.next() {
            let current_edge_tuple = self.current_edge_tuple;
            self.current_edge_tuple.1 += 1;
            if self.current_edge_tuple.1 == self.node_capacity {
                self.current_edge_tuple.0 += 1;
                self.current_edge_tuple.1 = 0;
            }
            if !edge.is_null() {
                return Some((
                    NodeId(current_edge_tuple.0),
                    NodeId(current_edge_tuple.1),
                    edge.as_ref().unwrap(),
                ));
            }
        }
        None
    }
}

struct EdgeIteratorMut<'a, It: Iterator<Item = &'a mut Null>, Null: Nullable + 'a> {
    edges: It,
    current_edge_tuple: (usize, usize),
    node_capacity: usize,
}

impl<'a, It: Iterator<Item = &'a mut Null>, Null: Nullable> Iterator
    for EdgeIteratorMut<'a, It, Null>
{
    type Item = (NodeId, NodeId, &'a mut Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(edge) = self.edges.next() {
            let current_edge_tuple = self.current_edge_tuple;
            self.current_edge_tuple.1 += 1;
            if self.current_edge_tuple.1 == self.node_capacity {
                self.current_edge_tuple.0 += 1;
                self.current_edge_tuple.1 = 0;
            }
            if !edge.is_null() {
                return Some((
                    NodeId(current_edge_tuple.0),
                    NodeId(current_edge_tuple.1),
                    edge.as_mut().unwrap(),
                ));
            }
        }
        None
    }
}

#[derive(Debug, Clone)]
struct IdStorage<T, S = RandomState> {
    elements: Vec<Option<T>>,
    upper_bound: usize,
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

impl<T, S> Index<usize> for IdStorage<T, S> {
    type Output = T;

    fn index(&self, index: usize) -> &T {
        self.elements[index].as_ref().unwrap()
    }
}

impl<T, S> IndexMut<usize> for IdStorage<T, S> {
    fn index_mut(&mut self, index: usize) -> &mut T {
        self.elements[index].as_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
struct IdStorageIterator<'a, T, S> {
    storage: &'a IdStorage<T, S>,
    current: Option<usize>,
}

impl<T, S> IdStorageIterator<'_, T, S> {
    fn new(storage: &IdStorage<T, S>) -> IdStorageIterator<'_, T, S> {
        IdStorageIterator {
            storage,
            current: None,
        }
    }
}

impl<'a, T, S: BuildHasher> Iterator for IdStorageIterator<'a, T, S> {
    type Item = (usize, &'a T);

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
        while self.storage.removed_ids.contains(current) && *current < self.storage.upper_bound {
            *current += 1;
        }

        if *current < self.storage.upper_bound {
            Some((
                *current,
                self.storage
                    .elements
                    .get(*current)
                    .expect("Current index should be within upper bound")
                    .as_ref()
                    .expect("Current index should not be removed"),
            ))
        } else {
            None
        }
    }
}

/// Create a new empty `MatrixGraph`.
impl<N, E, S: BuildHasher + Default, Null: Nullable<Wrapped = E>, Dir> Default
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let g = MatrixGraph::<i32, i32>::new();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_default() {
        let g = MatrixGraph::<i32, i32>::default();
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_with_capacity() {
        let g = MatrixGraph::<i32, i32>::with_capacity(10);
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    // #[test]
    // fn test_node_indexing() {
    //     let mut g: MatrixGraph<char, ()> = MatrixGraph::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     assert_eq!(g.node_count(), 2);
    //     assert_eq!(g.edge_count(), 0);
    //     assert_eq!(g[a], 'a');
    //     assert_eq!(g[b], 'b');
    // }

    // #[test]
    // fn test_remove_node() {
    //     let mut g: MatrixGraph<char, ()> = MatrixGraph::new();
    //     let a = g.add_node('a');

    //     g.remove_node(a);

    //     assert_eq!(g.node_count(), 0);
    //     assert_eq!(g.edge_count(), 0);
    // }

    // #[test]
    // fn test_add_edge() {
    //     let mut g = MatrixGraph::<_, _>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     g.add_edge(a, b, ());
    //     g.add_edge(b, c, ());
    //     assert_eq!(g.node_count(), 3);
    //     assert_eq!(g.edge_count(), 2);
    // }

    // #[test]
    // /// Adds an edge that triggers a second extension of the matrix.
    // /// From #425
    // fn test_add_edge_with_extension() {
    //     let mut g = DiMatrix::<u8, ()>::new();
    //     let _n0 = g.add_node(0);
    //     let n1 = g.add_node(1);
    //     let n2 = g.add_node(2);
    //     let n3 = g.add_node(3);
    //     let n4 = g.add_node(4);
    //     let _n5 = g.add_node(5);
    //     g.add_edge(n2, n1, ());
    //     g.add_edge(n2, n3, ());
    //     g.add_edge(n2, n4, ());
    //     assert_eq!(g.node_count(), 6);
    //     assert_eq!(g.edge_count(), 3);
    //     assert!(g.has_edge(n2, n1));
    //     assert!(g.has_edge(n2, n3));
    //     assert!(g.has_edge(n2, n4));
    // }

    // #[test]
    // fn test_has_edge() {
    //     let mut g = MatrixGraph::<_, _>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     g.add_edge(a, b, ());
    //     g.add_edge(b, c, ());
    //     assert!(g.has_edge(a, b));
    //     assert!(g.has_edge(b, c));
    //     assert!(!g.has_edge(a, c));
    //     assert!(!g.has_edge(10.into(), 100.into())); // Non-existent nodes.
    // }

    // #[test]
    // fn test_matrix_resize() {
    //     let mut g = DiMatrix::<u8, ()>::with_capacity(3);
    //     let n0 = g.add_node(0);
    //     let n1 = g.add_node(1);
    //     let n2 = g.add_node(2);
    //     let n3 = g.add_node(3);
    //     g.add_edge(n1, n0, ());
    //     g.add_edge(n1, n1, ());
    //     // Triggers a resize from capacity 3 to 4
    //     g.add_edge(n2, n3, ());
    //     assert_eq!(g.node_count(), 4);
    //     assert_eq!(g.edge_count(), 3);
    //     assert!(g.has_edge(n1, n0));
    //     assert!(g.has_edge(n1, n1));
    //     assert!(g.has_edge(n2, n3));
    // }

    // #[test]
    // fn test_add_edge_with_weights() {
    //     let mut g = MatrixGraph::<_, _>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     g.add_edge(a, b, true);
    //     g.add_edge(b, c, false);
    //     assert!(*g.edge_weight(a, b));
    //     assert!(!*g.edge_weight(b, c));
    // }

    // #[test]
    // fn test_add_edge_with_weights_undirected() {
    //     let mut g = MatrixGraph::<_, _, fxhash::FxBuildHasher, Undirected>::new_undirected();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     let d = g.add_node('d');
    //     g.add_edge(a, b, "ab");
    //     g.add_edge(a, a, "aa");
    //     g.add_edge(b, c, "bc");
    //     g.add_edge(d, d, "dd");
    //     assert_eq!(*g.edge_weight(a, b), "ab");
    //     assert_eq!(*g.edge_weight(b, c), "bc");
    // }

    // /// Shorthand for `.collect::<Vec<_>>()`
    // trait IntoVec<T> {
    //     fn into_vec(self) -> Vec<T>;
    // }

    // impl<It, T> IntoVec<T> for It
    // where
    //     It: Iterator<Item = T>,
    // {
    //     fn into_vec(self) -> Vec<T> {
    //         self.collect()
    //     }
    // }

    // #[test]
    // fn test_clear() {
    //     let mut g = MatrixGraph::<_, _>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     assert_eq!(g.node_count(), 3);

    //     g.add_edge(a, b, ());
    //     g.add_edge(b, c, ());
    //     g.add_edge(c, a, ());
    //     assert_eq!(g.edge_count(), 3);

    //     g.clear();

    //     assert_eq!(g.node_count(), 0);
    //     assert_eq!(g.edge_count(), 0);

    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     assert_eq!(g.node_count(), 3);
    //     assert_eq!(g.edge_count(), 0);

    //     assert_eq!(g.neighbors_directed(a, Incoming).into_vec(), vec![]);
    //     assert_eq!(g.neighbors_directed(b, Incoming).into_vec(), vec![]);
    //     assert_eq!(g.neighbors_directed(c, Incoming).into_vec(), vec![]);

    //     assert_eq!(g.neighbors_directed(a, Outgoing).into_vec(), vec![]);
    //     assert_eq!(g.neighbors_directed(b, Outgoing).into_vec(), vec![]);
    //     assert_eq!(g.neighbors_directed(c, Outgoing).into_vec(), vec![]);
    // }

    // #[test]
    // fn test_clear_undirected() {
    //     let mut g = MatrixGraph::<_, _, fxhash::FxBuildHasher, Undirected>::new_undirected();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     assert_eq!(g.node_count(), 3);

    //     g.add_edge(a, b, ());
    //     g.add_edge(b, c, ());
    //     g.add_edge(c, a, ());
    //     assert_eq!(g.edge_count(), 3);

    //     g.clear();

    //     assert_eq!(g.node_count(), 0);
    //     assert_eq!(g.edge_count(), 0);

    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     assert_eq!(g.node_count(), 3);
    //     assert_eq!(g.edge_count(), 0);

    //     assert_eq!(g.neighbors(a).into_vec(), vec![]);
    //     assert_eq!(g.neighbors(b).into_vec(), vec![]);
    //     assert_eq!(g.neighbors(c).into_vec(), vec![]);
    // }

    // /// Helper trait for always sorting before testing.
    // trait IntoSortedVec<T> {
    //     fn into_sorted_vec(self) -> Vec<T>;
    // }

    // impl<It, T> IntoSortedVec<T> for It
    // where
    //     It: Iterator<Item = T>,
    //     T: Ord,
    // {
    //     fn into_sorted_vec(self) -> Vec<T> {
    //         let mut v: Vec<T> = self.collect();
    //         v.sort();
    //         v
    //     }
    // }

    // /// Helper macro for always sorting before testing.
    // macro_rules! sorted_vec {
    //     ($($x:expr),*) => {
    //         {
    //             let mut v = vec![$($x,)*];
    //             v.sort();
    //             v
    //         }
    //     }
    // }

    // #[test]
    // fn test_neighbors() {
    //     let mut g = MatrixGraph::<_, _>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     g.add_edge(a, b, ());
    //     g.add_edge(a, c, ());

    //     let a_neighbors = g.neighbors(a).into_sorted_vec();
    //     assert_eq!(a_neighbors, sorted_vec![b, c]);

    //     let b_neighbors = g.neighbors(b).into_sorted_vec();
    //     assert_eq!(b_neighbors, vec![]);

    //     let c_neighbors = g.neighbors(c).into_sorted_vec();
    //     assert_eq!(c_neighbors, vec![]);
    // }

    // #[test]
    // fn test_neighbors_undirected() {
    //     let mut g = MatrixGraph::<_, _, fxhash::FxBuildHasher, Undirected>::new_undirected();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     g.add_edge(a, b, ());
    //     g.add_edge(a, c, ());

    //     let a_neighbors = g.neighbors(a).into_sorted_vec();
    //     assert_eq!(a_neighbors, sorted_vec![b, c]);

    //     let b_neighbors = g.neighbors(b).into_sorted_vec();
    //     assert_eq!(b_neighbors, sorted_vec![a]);

    //     let c_neighbors = g.neighbors(c).into_sorted_vec();
    //     assert_eq!(c_neighbors, sorted_vec![a]);
    // }

    // #[test]
    // fn test_remove_node_and_edges() {
    //     let mut g = MatrixGraph::<_, _>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     g.add_edge(a, b, ());
    //     g.add_edge(b, c, ());
    //     g.add_edge(c, a, ());

    //     // removing b should break the `a -> b` and `b -> c` edges
    //     g.remove_node(b);

    //     assert_eq!(g.node_count(), 2);

    //     let a_neighbors = g.neighbors(a).into_sorted_vec();
    //     assert_eq!(a_neighbors, vec![]);

    //     let c_neighbors = g.neighbors(c).into_sorted_vec();
    //     assert_eq!(c_neighbors, vec![a]);
    // }

    // #[test]
    // fn test_remove_node_and_edges_undirected() {
    //     let mut g = UnMatrix::<_, _>::new_undirected();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     g.add_edge(a, b, ());
    //     g.add_edge(b, c, ());
    //     g.add_edge(c, a, ());

    //     // removing a should break the `a - b` and `a - c` edges
    //     g.remove_node(a);

    //     assert_eq!(g.node_count(), 2);

    //     let b_neighbors = g.neighbors(b).into_sorted_vec();
    //     assert_eq!(b_neighbors, vec![c]);

    //     let c_neighbors = g.neighbors(c).into_sorted_vec();
    //     assert_eq!(c_neighbors, vec![b]);
    // }

    // #[test]
    // fn test_node_identifiers() {
    //     let mut g = MatrixGraph::<_, _>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     let d = g.add_node('c');
    //     g.add_edge(a, b, ());
    //     g.add_edge(a, c, ());

    //     let node_ids = g.node_identifiers().into_sorted_vec();
    //     assert_eq!(node_ids, sorted_vec![a, b, c, d]);
    // }

    // #[test]
    // fn test_edges_directed() {
    //     let g: MatrixGraph<char, bool> = MatrixGraph::from_edges([
    //         (0, 5),
    //         (0, 2),
    //         (0, 3),
    //         (0, 1),
    //         (1, 3),
    //         (2, 3),
    //         (2, 4),
    //         (4, 0),
    //         (6, 6),
    //     ]);

    //     assert_eq!(g.edges_directed(node_index(0), Outgoing).count(), 4);
    //     assert_eq!(g.edges_directed(node_index(1), Outgoing).count(), 1);
    //     assert_eq!(g.edges_directed(node_index(2), Outgoing).count(), 2);
    //     assert_eq!(g.edges_directed(node_index(3), Outgoing).count(), 0);
    //     assert_eq!(g.edges_directed(node_index(4), Outgoing).count(), 1);
    //     assert_eq!(g.edges_directed(node_index(5), Outgoing).count(), 0);
    //     assert_eq!(g.edges_directed(node_index(6), Outgoing).count(), 1);

    //     assert_eq!(g.edges_directed(node_index(0), Incoming).count(), 1);
    //     assert_eq!(g.edges_directed(node_index(1), Incoming).count(), 1);
    //     assert_eq!(g.edges_directed(node_index(2), Incoming).count(), 1);
    //     assert_eq!(g.edges_directed(node_index(3), Incoming).count(), 3);
    //     assert_eq!(g.edges_directed(node_index(4), Incoming).count(), 1);
    //     assert_eq!(g.edges_directed(node_index(5), Incoming).count(), 1);
    //     assert_eq!(g.edges_directed(node_index(6), Incoming).count(), 1);
    // }

    // #[test]
    // fn test_edges_undirected() {
    //     let g: UnMatrix<char, bool> = UnMatrix::from_edges([
    //         (0, 5),
    //         (0, 2),
    //         (0, 3),
    //         (0, 1),
    //         (1, 3),
    //         (2, 3),
    //         (2, 4),
    //         (4, 0),
    //         (6, 6),
    //     ]);

    //     assert_eq!(g.edges(node_index(0)).count(), 5);
    //     assert_eq!(g.edges(node_index(1)).count(), 2);
    //     assert_eq!(g.edges(node_index(2)).count(), 3);
    //     assert_eq!(g.edges(node_index(3)).count(), 3);
    //     assert_eq!(g.edges(node_index(4)).count(), 2);
    //     assert_eq!(g.edges(node_index(5)).count(), 1);
    //     assert_eq!(g.edges(node_index(6)).count(), 1);
    // }

    // #[test]
    // fn test_edges_of_absent_node_is_empty_iterator() {
    //     let g: MatrixGraph<char, bool> = MatrixGraph::new();
    //     assert_eq!(g.edges(node_index(0)).count(), 0);
    // }

    // #[test]
    // fn test_neighbors_of_absent_node_is_empty_iterator() {
    //     let g: MatrixGraph<char, bool> = MatrixGraph::new();
    //     assert_eq!(g.neighbors(node_index(0)).count(), 0);
    // }

    // #[test]
    // fn test_edge_references() {
    //     let g: MatrixGraph<char, bool> = MatrixGraph::from_edges([
    //         (0, 5),
    //         (0, 2),
    //         (0, 3),
    //         (0, 1),
    //         (1, 3),
    //         (2, 3),
    //         (2, 4),
    //         (4, 0),
    //         (6, 6),
    //     ]);

    //     assert_eq!(g.edge_references().count(), 9);
    // }

    // #[test]
    // fn test_edge_references_undirected() {
    //     let g: UnMatrix<char, bool> = UnMatrix::from_edges([
    //         (0, 5),
    //         (0, 2),
    //         (0, 3),
    //         (0, 1),
    //         (1, 3),
    //         (2, 3),
    //         (2, 4),
    //         (4, 0),
    //         (6, 6),
    //     ]);

    //     assert_eq!(g.edge_references().count(), 9);
    // }

    // #[test]
    // fn test_id_storage() {
    //     let mut storage: IdStorage<char> =
    //         IdStorage::with_capacity_and_hasher(0, Default::default());
    //     let a = storage.add('a');
    //     let b = storage.add('b');
    //     let c = storage.add('c');

    //     assert!(a < b && b < c);

    //     // list IDs
    //     assert_eq!(storage.iter_ids().into_vec(), vec![a, b, c]);

    //     storage.remove(b);

    //     // re-use of IDs
    //     let bb = storage.add('B');
    //     assert_eq!(b, bb);

    //     // list IDs
    //     assert_eq!(storage.iter_ids().into_vec(), vec![a, b, c]);
    // }

    // #[test]
    // fn test_not_zero() {
    //     let mut g: MatrixGraph<(), i32, fxhash::FxBuildHasher, Directed, NotZero<i32>> =
    //         MatrixGraph::default();

    //     let a = g.add_node(());
    //     let b = g.add_node(());

    //     assert!(!g.has_edge(a, b));
    //     assert_eq!(g.edge_count(), 0);

    //     g.add_edge(a, b, 12);

    //     assert!(g.has_edge(a, b));
    //     assert_eq!(g.edge_count(), 1);
    //     assert_eq!(g.edge_weight(a, b), &12);

    //     g.remove_edge(a, b);

    //     assert!(!g.has_edge(a, b));
    //     assert_eq!(g.edge_count(), 0);
    // }

    // #[test]
    // #[should_panic]
    // fn test_not_zero_asserted() {
    //     let mut g: MatrixGraph<(), i32, fxhash::FxBuildHasher, Directed, NotZero<i32>> =
    //         MatrixGraph::default();

    //     let a = g.add_node(());
    //     let b = g.add_node(());

    //     g.add_edge(a, b, 0); // this should trigger an assertion
    // }

    // #[test]
    // fn test_not_zero_float() {
    //     let mut g: MatrixGraph<(), f32, fxhash::FxBuildHasher, Directed, NotZero<f32>> =
    //         MatrixGraph::default();

    //     let a = g.add_node(());
    //     let b = g.add_node(());

    //     assert!(!g.has_edge(a, b));
    //     assert_eq!(g.edge_count(), 0);

    //     g.add_edge(a, b, 12.);

    //     assert!(g.has_edge(a, b));
    //     assert_eq!(g.edge_count(), 1);
    //     assert_eq!(g.edge_weight(a, b), &12.);

    //     g.remove_edge(a, b);

    //     assert!(!g.has_edge(a, b));
    //     assert_eq!(g.edge_count(), 0);
    // }
    // #[test]
    // // From https://github.com/petgraph/petgraph/issues/523
    // fn test_tarjan_scc_with_removed_node() {
    //     let mut g: MatrixGraph<(), ()> = MatrixGraph::new();

    //     g.add_node(());
    //     let b = g.add_node(());
    //     g.add_node(());

    //     g.remove_node(b);

    //     assert_eq!(
    //         crate::algo::tarjan_scc(&g),
    //         [[node_index(0)], [node_index(2)]]
    //     );
    // }

    // #[test]
    // // From https://github.com/petgraph/petgraph/issues/523
    // fn test_kosaraju_scc_with_removed_node() {
    //     let mut g: MatrixGraph<(), ()> = MatrixGraph::new();

    //     g.add_node(());
    //     let b = g.add_node(());
    //     g.add_node(());

    //     g.remove_node(b);

    //     assert_eq!(
    //         crate::algo::kosaraju_scc(&g),
    //         [[node_index(2)], [node_index(0)]]
    //     );
    // }

    // #[test]
    // fn test_try_update_edge() {
    //     let mut g = MatrixGraph::<char, u32>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     g.add_edge(a, b, 1);
    //     g.add_edge(b, c, 2);
    //     assert_eq!(g.try_update_edge(a, b, 10), Ok(Some(1)));
    //     assert_eq!(g.try_update_edge(a, b, 100), Ok(Some(10)));
    //     assert_eq!(g.try_update_edge(a, c, 33), Ok(None));
    //     assert_eq!(g.try_update_edge(a, c, 66), Ok(Some(33)));
    //     assert_eq!(
    //         g.try_update_edge(10.into(), 20.into(), 5),
    //         Err(MatrixError::NodeMissed(10))
    //     );
    // }

    // #[test]
    // fn test_add_or_update_edge() {
    //     let mut g = MatrixGraph::<char, u32>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     assert_eq!(g.add_or_update_edge(a, b, 1), Ok(None));
    //     assert_eq!(g.add_or_update_edge(b, c, 2), Ok(None));
    //     assert_eq!(g.add_or_update_edge(a, b, 10), Ok(Some(1)));
    //     assert_eq!(g.add_or_update_edge(a, c, 33), Ok(None));
    //     assert_eq!(g.add_or_update_edge(10.into(), 20.into(), 5), Ok(None));
    //     assert!(g.has_edge(10.into(), 20.into()));
    //     assert!(g.node_capacity >= 20);
    // }

    // #[test]
    // fn test_remove_edge() {
    //     let mut g = MatrixGraph::<char, u32>::new();
    //     let a = g.add_node('a');
    //     let b = g.add_node('b');
    //     let c = g.add_node('c');
    //     g.add_edge(a, b, 1);
    //     g.add_edge(b, c, 2);
    //     assert_eq!(g.try_remove_edge(a, b), Some(1));
    //     assert_eq!(g.try_remove_edge(a, b), None);
    //     assert_eq!(g.try_remove_edge(a, c), None);
    // }

    // #[test]
    // fn test_try_add_node() {
    //     let mut graph =
    //         MatrixGraph::<(), u32, fxhash::FxBuildHasher, Directed, Option<u32>,
    // u8>::with_capacity(             255,
    //         );
    //     for i in 0..255 {
    //         assert_eq!(graph.try_add_node(()), Ok(i.into()));
    //     }
    //     assert_eq!(graph.try_add_node(()), Err(MatrixError::NodeIxLimit));
    // }
}
