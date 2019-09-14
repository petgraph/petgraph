use std::marker::PhantomData;
use std::ops::{Index, IndexMut};

use std::cmp;
use std::mem;

use indexmap::IndexSet;

use fixedbitset::FixedBitSet;

use crate::{
    Directed,
    EdgeType,
    Outgoing,
    Undirected,
    Direction,
    IntoWeightedEdge,
};

use crate::graph::NodeIndex as GraphNodeIndex;

use crate::visit::{
    Data,
    GetAdjacencyMatrix,
    GraphBase,
    GraphProp,
    IntoEdgeReferences,
    IntoEdges,
    IntoNeighbors,
    IntoNeighborsDirected,
    IntoNodeIdentifiers,
    IntoNodeReferences,
    NodeCount,
    NodeIndexable,
    NodeCompactIndexable,
    Visitable,
};

use crate::data::Build;

pub use crate::graph::IndexType;

// The following types are used to control the max size of the adjacency matrix. Since the maximum
// size of the matrix vector's is the square of the maximum number of nodes, the number of nodes
// should be reasonably picked.
type Ix = u16;

pub type NodeIndex = GraphNodeIndex<Ix>;

mod private {
    pub trait Sealed {}

    impl<N> Sealed for super::NotZero<N> {}
    impl<N> Sealed for Option<N> {}
}

/// Wrapper trait for an `Option`, allowing user-defined structs to be input as containers when
/// defining a null element.
///
/// Note: this trait currently is sealed and cannot be implemented for types outside this crate.
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

/// Struct used to optimize the memory usage of edge weights `E` in a
/// [`MatrixGraph`](struct.MatrixGraph.html), by using a sentinel value (such as 0 for numeric
/// types) instead of the default `Option<E>`.
///
/// Note that if you're already using the standard non-zero types (such as `NonZeroU32`), you don't
/// have to use this wrapper and can leave the default Null type argument.
pub struct NotZero<N>(N);

impl<N: Zero> Default for NotZero<N> {
    fn default() -> Self {
        NotZero(N::zero())
    }
}

impl<N: Zero> Nullable for NotZero<N> {
    type Wrapped = N;

    fn new(value: T) -> Self {
        assert!(!value.is_zero());
        NotZero(value)
    }

    // implemented here for optimization purposes
    fn is_null(&self) -> bool {
        self.0.is_zero()
    }

    fn as_ref(&self) -> Option<&Self::Wrapped> {
        if !self.is_null() { Some(&self.0) }
        else { None }
    }

    fn as_mut(&mut self) -> Option<&mut Self::Wrapped> {
        if !self.is_null() { Some(&mut self.0) }
        else { None }
    }
}

impl<N: Zero> Into<Option<N>> for NotZero<N> {
    fn into(self) -> Option<N> {
        if !self.is_null() { Some(self.0) }
        else { None }
    }
}

/// Base trait for types that can be wrapped in a [`NotZero`](trait.NotZero.html).
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
    ($t:ty,$z:expr) => {
        impl Zero for $t {
            fn zero() -> Self {
                $z as $t
            }

            fn is_zero(&self) -> bool {
                self == &Self::zero()
            }
        }
    }
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

#[inline]
pub fn node_index(ax: usize) -> NodeIndex {
    debug_assert!(ax.index() <= Ix::max_value() as usize);
    NodeIndex::new(ax)
}

#[derive(Clone)]
pub struct MatrixGraph<N, E, Ty = Directed, Null: Nullable<Wrapped=E> = Option<E>> {
    node_adjacencies: Vec<Null>,
    node_capacity: usize,
    nodes: IdStorage<N>,
    nb_edges: usize,
    ty: PhantomData<Ty>,
}

pub type DiMatrix<N, E, Null: Nullable<Wrapped=E> = Option<E>> = MatrixGraph<N, E, Directed, Null>;
pub type UnMatrix<N, E, Null: Nullable<Wrapped=E> = Option<E>> = MatrixGraph<N, E, Undirected, Null>;

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> MatrixGraph<N, E, Ty, Null> {
    /// Create a new `MatrixGraph` with estimated capacity for nodes.
    pub fn with_nodes(nodes: usize) -> Self {
        let mut m = MatrixGraph {
            node_adjacencies: vec![],
            node_capacity: 0,
            nodes: IdStorage::with_capacity(nodes),
            nb_edges: 0,
            ty: PhantomData,
        };

        m.extend_capacity_to(nodes);

        m
    }

    #[inline]
    fn extend_capacity_to(&mut self, min_node_index: usize) {
        debug_assert!(min_node_index <= Ix::max_value() as usize);
        self.node_capacity = extend_linearized_matrix::<Ty, _>(&mut self.node_adjacencies, self.node_capacity, min_node_index);
    }

    #[inline]
    fn to_edge_position(&self, a: NodeIndex, b: NodeIndex) -> usize {
        to_linearized_matrix_position::<Ty>(a.index(), b.index(), self.node_capacity)
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

    /// Whether the graph has directed edges or not.
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
    /// **Panics** if the Graph is at the maximum number of nodes for its index
    /// type.
    pub fn add_node(&mut self, weight: N) -> NodeIndex {
        node_index(self.nodes.add(weight))
    }

    /// Remove `a` from the graph.
    ///
    /// Computes in **O(V)** time, due to the removal of edges with other nodes.
    ///
    /// **Panics** if the node `a` does not exist.
    pub fn remove_node(&mut self, a: NodeIndex) -> N {
        macro_rules! node_ids {
            () => {
                self.node_identifiers()
                    .filter(|n| n.index() < self.node_capacity)
            }
        };

        let out_edges_to_remove: Vec<_> = node_ids!()
            .map(|n| self.to_edge_position(a, n))
            .filter(|&p| !self.node_adjacencies[p].is_null())
            .collect();
        for p in out_edges_to_remove {
            self.node_adjacencies[p] = Default::default();
        }

        if Ty::is_directed() {
            let in_edges_to_remove: Vec<_> = node_ids!()
                .map(|n| self.to_edge_position(n, a))
                .filter(|&p| !self.node_adjacencies[p].is_null())
                .collect();
            for p in in_edges_to_remove {
                self.node_adjacencies[p] = Default::default();
            }
        }

        self.nodes.remove(a.index())
    }

    /// Update the edge from `a` to `b` to the graph, with its associated data `weight`.
    ///
    /// Return the previous data, if any.
    ///
    /// Computes in **O(1)** time, best case.
    /// Computes in **O(|V|^2)** time, worst case (matrix needs to be re-allocated).
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn update_edge(&mut self, a: NodeIndex, b: NodeIndex, weight: E) -> Option<E> {
        let min_node_index = cmp::max(a.index(), b.index());
        if min_node_index >= self.node_capacity {
            self.extend_capacity_to(min_node_index);
        }

        let p = self.to_edge_position(a, b);
        let old_weight = mem::replace(&mut self.node_adjacencies[p], Null::new(weight));
        if old_weight.is_null() {
            self.nb_edges += 1;
        }
        old_weight.into()
    }

    /// Add an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(1)** time, best case.
    /// Computes in **O(|V|^2)** time, worst case (matrix needs to be re-allocated).
    ///
    /// **Panics** if any of the nodes don't exist.
    /// **Panics** if an edge already exists from `a` to `b`.
    ///
    /// **Note:** `MatrixGraph` does not allow adding parallel (“duplicate”) edges. If you want to avoid
    /// this, use [`.update_edge(a, b, weight)`](#method.update_edge) instead.
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex, weight: E) {
        let old_edge_id = self.update_edge(a, b, weight);
        assert!(old_edge_id.is_none());
    }

    /// Remove the edge from `a` to `b` to the graph.
    ///
    /// **Panics** if any of the nodes don't exist.
    /// **Panics** if an edge already exists from `a` to `b`.
    ///
    /// **Note:** `MatrixGraph` does not allow adding parallel (“duplicate”) edges. If you want to avoid
    /// this, use [`.update_edge(a, b, weight)`](#method.update_edge) instead.
    pub fn remove_edge(&mut self, a: NodeIndex, b: NodeIndex) -> E {
        let p = self.to_edge_position(a, b);
        let old_weight = mem::replace(&mut self.node_adjacencies[p], Default::default()).into().unwrap();
        let old_weight: Option<_> = old_weight.into();
        self.nb_edges -= 1;
        old_weight.unwrap()
    }

    /// Return true if there is an edge between `a` and `b`.
    ///
    /// **Panics** if any of the nodes don't exist.
    pub fn has_edge(&self, a: NodeIndex, b: NodeIndex) -> bool {
        let p = self.to_edge_position(a, b);
        !self.node_adjacencies[p].is_null()
    }

    /// Access the weight for node `a`.
    ///
    /// Also available with indexing syntax: `&graph[a]`.
    ///
    /// **Panics** if the node doesn't exist.
    pub fn node_weight(&self, a: NodeIndex) -> &N {
        &self.nodes[a.index()]
    }

    /// Access the weight for node `a`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[a]`.
    ///
    /// **Panics** if the node doesn't exist.
    pub fn node_weight_mut(&mut self, a: NodeIndex) -> &mut N {
        &mut self.nodes[a.index()]
    }

    /// Access the weight for edge `e`.
    ///
    /// Also available with indexing syntax: `&graph[e]`.
    ///
    /// **Panics** if the edge doesn't exist.
    pub fn edge_weight(&self, a: NodeIndex, b: NodeIndex) -> &E {
        let p = self.to_edge_position(a, b);
        self.node_adjacencies[p].as_ref().unwrap()
    }

    /// Access the weight for edge `e`, mutably.
    ///
    /// Also available with indexing syntax: `&mut graph[e]`.
    ///
    /// **Panics** if the edge doesn't exist.
    pub fn edge_weight_mut(&mut self, a: NodeIndex, b: NodeIndex) -> &mut E {
        let p = self.to_edge_position(a, b);
        self.node_adjacencies[p].as_mut().unwrap()
    }

    pub fn neighbors(&self, a: NodeIndex) -> Neighbors<Ty, Null> {
        Neighbors(Edges::on_columns(a.index(), &self.node_adjacencies, self.node_capacity))
    }

    pub fn edges(&self, a: NodeIndex) -> Edges<Ty, Null> {
        Edges::on_columns(a.index(), &self.node_adjacencies, self.node_capacity)
    }

    pub fn node_identifiers(&self) -> NodeIdentifiers {
        NodeIdentifiers(self.nodes.iter_ids())
    }

    pub fn node_references(&self) -> NodeReferences<N> {
        NodeReferences::new(&self.nodes)
    }

    pub fn edge_references(&self) -> EdgeReferences<Ty, Null> {
        EdgeReferences::new(&self.node_adjacencies, self.node_capacity)
    }

    pub fn from_edges<I>(iterable: I) -> Self
        where I: IntoIterator,
              I::Item: IntoWeightedEdge<E>,
              <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex>,
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
        where I: IntoIterator,
              I::Item: IntoWeightedEdge<E>,
              <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex>,
              N: Default,
    {
        for elt in iterable {
            let (source, target, weight) = elt.into_weighted_edge();
            let (source, target) = (source.into(), target.into());
            let nx = cmp::max(source, target);
            while nx.index() >= self.node_count() {
                self.add_node(N::default());
            }
            self.add_edge(source, target, weight);
        }
    }
}

impl<N, E, Null: Nullable<Wrapped=E>> MatrixGraph<N, E, Directed, Null> {
    pub fn neighbors_directed(&self, a: NodeIndex, d: Direction) -> Neighbors<Directed, Null> {
        if d == Outgoing {
            self.neighbors(a)
        } else {
            Neighbors(Edges::on_rows(a.index(), &self.node_adjacencies, self.node_capacity))
        }
    }

    pub fn edges_directed(&self, a: NodeIndex, d: Direction) -> Edges<Directed, Null> {
        if d == Outgoing {
            self.edges(a)
        } else {
            Edges::on_rows(a.index(), &self.node_adjacencies, self.node_capacity)
        }
    }
}

pub struct NodeIdentifiers<'a>(IdIterator<'a>);

impl<'a> Iterator for NodeIdentifiers<'a> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(node_index)
    }
}

pub struct NodeReferences<'a, N: 'a> {
    nodes: &'a IdStorage<N>,
    iter: IdIterator<'a>,
}

impl<'a, N: 'a> NodeReferences<'a, N> {
    fn new(nodes: &'a IdStorage<N>) -> Self {
        NodeReferences {
            nodes: nodes,
            iter: nodes.iter_ids(),
        }
    }
}

impl<'a, N: 'a> Iterator for NodeReferences<'a, N> {
    type Item = (NodeIndex, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next()
            .map(|i| (node_index(i), &self.nodes[i]))
    }
}

pub struct EdgeReferences<'a, Ty: EdgeType, Null: 'a + Nullable> {
    row: usize,
    column: usize,
    node_adjacencies: &'a [Null],
    node_capacity: usize,
    ty: PhantomData<Ty>,
}

impl<'a, Ty: EdgeType, Null: 'a + Nullable> EdgeReferences<'a, Ty, Null> {
    fn new(node_adjacencies: &'a [Null], node_capacity: usize) -> Self {
        EdgeReferences {
            row: 0, column: 0,
            node_adjacencies: node_adjacencies,
            node_capacity: node_capacity,
            ty: PhantomData,
        }
    }
}

impl<'a, Ty: EdgeType, Null: Nullable> Iterator for EdgeReferences<'a, Ty, Null> {
    type Item = (NodeIndex, NodeIndex, &'a Null::Wrapped);

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
            let max_column_len = if !Ty::is_directed() { row + 1 } else { self.node_capacity };
            if self.column >= max_column_len {
                self.column = 0;
                self.row += 1;
            }

            let p = to_linearized_matrix_position::<Ty>(row, column, self.node_capacity);
            if let Some(e) = self.node_adjacencies[p].as_ref() {
                return Some((node_index(row), node_index(column), e));
            }
        }
    }
}

pub struct Neighbors<'a, Ty: EdgeType, Null: 'a + Nullable>(Edges<'a, Ty, Null>);

impl<'a, Ty: EdgeType, Null: Nullable> Iterator for Neighbors<'a, Ty, Null> {
    type Item = NodeIndex;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(_, b, _)| b)
    }
}

enum NeighborIterDirection {
    Rows,
    Columns,
}

pub struct Edges<'a, Ty: EdgeType, Null: 'a + Nullable> {
    iter_direction: NeighborIterDirection,
    node_adjacencies: &'a [Null],
    node_capacity: usize,
    row: usize,
    column: usize,
    ty: PhantomData<Ty>,
}

impl<'a, Ty: EdgeType, Null: 'a + Nullable> Edges<'a, Ty, Null> {
    fn on_columns(row: usize, node_adjacencies: &'a [Null], node_capacity: usize) -> Self {
        Edges {
            iter_direction: NeighborIterDirection::Columns,
            node_adjacencies: node_adjacencies,
            node_capacity: node_capacity,
            row: row,
            column: 0,
            ty: PhantomData,
        }
    }

    fn on_rows(column: usize, node_adjacencies: &'a [Null], node_capacity: usize) -> Self {
        Edges {
            iter_direction: NeighborIterDirection::Rows,
            node_adjacencies: node_adjacencies,
            node_capacity: node_capacity,
            row: 0,
            column: column,
            ty: PhantomData,
        }
    }
}

impl<'a, Ty: EdgeType, Null: Nullable> Iterator for Edges<'a, Ty, Null> {
    type Item = (NodeIndex, NodeIndex, &'a Null::Wrapped);

    fn next(&mut self) -> Option<Self::Item> {
        use self::NeighborIterDirection::*;

        loop {
            let (row, column) = (self.row, self.column);
            if row >= self.node_capacity || column >= self.node_capacity {
                return None;
            }

            match self.iter_direction {
                Rows    => self.row += 1,
                Columns => self.column += 1,
            }

            let p = to_linearized_matrix_position::<Ty>(row, column, self.node_capacity);
            if let Some(e) = self.node_adjacencies[p].as_ref() {
                let (a, b) = match self.iter_direction {
                    Rows    => (column, row),
                    Columns => (row, column),
                };

                return Some((node_index(a), node_index(b), e));
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
fn extend_linearized_matrix<Ty: EdgeType, T: Default>(node_adjacencies: &mut Vec<T>, old_node_capacity: usize, min_node_capacity: usize) -> usize {
    if Ty::is_directed() {
        extend_flat_square_matrix(node_adjacencies, old_node_capacity, min_node_capacity)
    } else {
        extend_lower_triangular_matrix(node_adjacencies, min_node_capacity)
    }
}

#[inline]
fn to_flat_square_matrix_position(row: usize, column: usize, width: usize) -> usize {
    row * width + column
}

#[inline]
fn extend_flat_square_matrix<T: Default>(node_adjacencies: &mut Vec<T>, old_node_capacity: usize, min_node_capacity: usize) -> usize {
    let min_node_capacity = (min_node_capacity + 1).next_power_of_two();

    // Optimization: when resizing the matrix this way we skip the first few grows to make
    // small matrices a bit faster to work with.
    const MIN_CAPACITY: usize = 4;
    let new_node_capacity = cmp::max(min_node_capacity, MIN_CAPACITY);

    let mut new_node_adjacencies = vec![];
    ensure_len(&mut new_node_adjacencies, new_node_capacity.pow(2));

    for c in 0..old_node_capacity {
        let pos = c * old_node_capacity;
        let new_pos = c * new_node_capacity;

        let mut old = &mut node_adjacencies[pos..pos + old_node_capacity];
        let mut new = &mut new_node_adjacencies[new_pos..new_pos + old_node_capacity];

        mem::swap(&mut old, &mut new);
    }

    mem::swap(node_adjacencies, &mut new_node_adjacencies);

    new_node_capacity
}

#[inline]
fn to_lower_triangular_matrix_position(row: usize, column: usize) -> usize {
    let (row, column) = if row > column { (row, column) } else { (column, row) };
    (row * (row + 1)) / 2 + column
}

#[inline]
fn extend_lower_triangular_matrix<T: Default>(node_adjacencies: &mut Vec<T>, new_node_capacity: usize) -> usize {
    let max_pos = to_lower_triangular_matrix_position(new_node_capacity, new_node_capacity);
    ensure_len(node_adjacencies, max_pos + 1);
    new_node_capacity + 1
}

/// Grow a Vec by appending the type's default value the `size` is reached.
fn ensure_len<T: Default>(v: &mut Vec<T>, size: usize) {
    if let Some(n) = size.checked_sub(v.len()) {
        v.reserve(n);
        for _ in 0..n {
            v.push(T::default());
        }
    }
}

#[derive(Clone)]
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

pub struct IdIterator<'a> {
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
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> Default for MatrixGraph<N, E, Ty, Null> {
    fn default() -> Self {
        Self::with_nodes(0)
    }
}

impl<N, E> MatrixGraph<N, E, Directed> {
    /// Create a new `MatrixGraph` with directed edges.
    ///
    /// This is a convenience method. Use `MatrixGraph::with_nodes` or `MatrixGraph::default` for
    /// a constructor that is generic in all the type parameters of `MatrixGraph`.
    pub fn new() -> Self {
        MatrixGraph::default()
    }
}

impl<N, E> MatrixGraph<N, E, Undirected> {
    /// Create a new `MatrixGraph` with undirected edges.
    ///
    /// This is a convenience method. Use `MatrixGraph::with_nodes` or `MatrixGraph::default` for
    /// a constructor that is generic in all the type parameters of `MatrixGraph`.
    pub fn new_undirected() -> Self {
        MatrixGraph::default()
    }
}

/// Index the `MatrixGraph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> Index<NodeIndex> for MatrixGraph<N, E, Ty, Null> {
    type Output = N;

    fn index(&self, ax: NodeIndex) -> &N {
        self.node_weight(ax)
    }
}

/// Index the `MatrixGraph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> IndexMut<NodeIndex> for MatrixGraph<N, E, Ty, Null> {
    fn index_mut(&mut self, ax: NodeIndex) -> &mut N {
        self.node_weight_mut(ax)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> NodeCount for MatrixGraph<N, E, Ty, Null> {
    fn node_count(&self) -> usize {
        MatrixGraph::node_count(self)
    }
}

/// Index the `MatrixGraph` by `NodeIndex` pair to access edge weights.
///
/// Also available with indexing syntax: `&graph[e]`.
///
/// **Panics** if no edge exists between `a` and `b`.
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> Index<(NodeIndex, NodeIndex)> for MatrixGraph<N, E, Ty, Null> {
    type Output = E;

    fn index(&self, (ax, bx): (NodeIndex, NodeIndex)) -> &E {
        self.edge_weight(ax, bx)
    }
}

/// Index the `MatrixGraph` by `NodeIndex` pair to access edge weights.
///
/// Also available with indexing syntax: `&mut graph[e]`.
///
/// **Panics** if no edge exists between `a` and `b`.
impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> IndexMut<(NodeIndex, NodeIndex)> for MatrixGraph<N, E, Ty, Null> {
    fn index_mut(&mut self, (ax, bx): (NodeIndex, NodeIndex)) -> &mut E {
        self.edge_weight_mut(ax, bx)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> GetAdjacencyMatrix for MatrixGraph<N, E, Ty, Null> {
    type AdjMatrix = ();

    fn adjacency_matrix(&self) -> Self::AdjMatrix {
    }

    fn is_adjacent(&self, _: &Self::AdjMatrix, a: NodeIndex, b: NodeIndex) -> bool {
        MatrixGraph::has_edge(self, a, b)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> Visitable for MatrixGraph<N, E, Ty, Null> {
    type Map = FixedBitSet;

    fn visit_map(&self) -> FixedBitSet {
        FixedBitSet::with_capacity(self.node_count())
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
        map.grow(self.node_count());
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> GraphBase for MatrixGraph<N, E, Ty, Null> {
    type NodeId = NodeIndex;
    type EdgeId = (NodeIndex, NodeIndex);
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> GraphProp for MatrixGraph<N, E, Ty, Null> {
    type EdgeType = Ty;
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> Data for MatrixGraph<N, E, Ty, Null> {
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<'a, N, E: 'a, Ty: EdgeType, Null: Nullable<Wrapped=E>> IntoNodeIdentifiers for &'a MatrixGraph<N, E, Ty, Null> {
    type NodeIdentifiers = NodeIdentifiers<'a>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        MatrixGraph::node_identifiers(self)
    }
}

impl<'a, N, E: 'a, Ty: EdgeType, Null: Nullable<Wrapped=E>> IntoNeighbors for &'a MatrixGraph<N, E, Ty, Null> {
    type Neighbors = Neighbors<'a, Ty, Null>;

    fn neighbors(self, a: NodeIndex) -> Self::Neighbors {
        MatrixGraph::neighbors(self, a)
    }
}

impl<'a, N, E: 'a, Null: Nullable<Wrapped=E>> IntoNeighborsDirected for &'a MatrixGraph<N, E, Directed, Null> {
    type NeighborsDirected = Neighbors<'a, Directed, Null>;

    fn neighbors_directed(self, a: NodeIndex, d: Direction) -> Self::NeighborsDirected {
        MatrixGraph::neighbors_directed(self, a, d)
    }
}

impl<'a, N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> IntoNodeReferences for &'a MatrixGraph<N, E, Ty, Null> {
    type NodeRef = (NodeIndex, &'a N);
    type NodeReferences = NodeReferences<'a, N>;
    fn node_references(self) -> Self::NodeReferences {
        MatrixGraph::node_references(self)
    }
}

impl<'a, N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> IntoEdgeReferences for &'a MatrixGraph<N, E, Ty, Null> {
    type EdgeRef = (NodeIndex, NodeIndex, &'a E);
    type EdgeReferences = EdgeReferences<'a, Ty, Null>;
    fn edge_references(self) -> Self::EdgeReferences {
        MatrixGraph::edge_references(self)
    }
}

impl<'a, N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> IntoEdges for &'a MatrixGraph<N, E, Ty, Null> {
    type Edges = Edges<'a, Ty, Null>;
    fn edges(self, a: Self::NodeId) -> Self::Edges {
        MatrixGraph::edges(self, a)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> NodeIndexable for MatrixGraph<N, E, Ty, Null> {
    fn node_bound(&self) -> usize {
        self.node_count()
    }
    fn to_index(&self, ix: NodeIndex) -> usize {
        ix.index()
    }
    fn from_index(&self, ix: usize) -> Self::NodeId {
        node_index(ix)
    }
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> NodeCompactIndexable for MatrixGraph<N, E, Ty, Null> {
}

impl<N, E, Ty: EdgeType, Null: Nullable<Wrapped=E>> Build for MatrixGraph<N, E, Ty, Null> {
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId {
        self.add_node(weight)
    }

    fn add_edge(&mut self, a: Self::NodeId, b: Self::NodeId, weight: Self::EdgeWeight) -> Option<Self::EdgeId> {
        if !self.has_edge(a, b) {
            MatrixGraph::update_edge(self, a, b, weight);
            Some((a, b))
        } else {
            None
        }
    }

    fn update_edge(&mut self, a: Self::NodeId, b: Self::NodeId, weight: Self::EdgeWeight) -> Self::EdgeId {
        MatrixGraph::update_edge(self, a, b, weight);
        (a, b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Outgoing, Incoming};

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
        let g = MatrixGraph::<i32, i32>::with_nodes(10);
        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_node_indexing() {
        let mut g: MatrixGraph<char, ()> = MatrixGraph::new();
        let a = g.add_node('a');
        let b = g.add_node('b');
        assert_eq!(g.node_count(), 2);
        assert_eq!(g.edge_count(), 0);
        assert_eq!(g[a], 'a');
        assert_eq!(g[b], 'b');
    }

    #[test]
    fn test_remove_node() {
        let mut g: MatrixGraph<char, ()> = MatrixGraph::new();
        let a = g.add_node('a');

        g.remove_node(a);

        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    fn test_add_edge() {
        let mut g = MatrixGraph::new();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 2);
    }

    #[test]
    fn test_add_edge_with_weights() {
        let mut g = MatrixGraph::new();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        g.add_edge(a, b, true);
        g.add_edge(b, c, false);
        assert_eq!(*g.edge_weight(a, b), true);
        assert_eq!(*g.edge_weight(b, c), false);
    }

    #[test]
    fn test_add_edge_with_weights_undirected() {
        let mut g = MatrixGraph::new_undirected();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        let d = g.add_node('d');
        g.add_edge(a, b, "ab");
        g.add_edge(a, a, "aa");
        g.add_edge(b, c, "bc");
        g.add_edge(d, d, "dd");
        assert_eq!(*g.edge_weight(a, b), "ab");
        assert_eq!(*g.edge_weight(b, c), "bc");
    }

    /// Shorthand for `.collect::<Vec<_>>()`
    trait IntoVec<T> {
        fn into_vec(self) -> Vec<T>;
    }

    impl<It, T> IntoVec<T> for It
        where It: Iterator<Item=T>,
    {
        fn into_vec(self) -> Vec<T> {
            self.collect()
        }
    }

    #[test]
    fn test_clear() {
        let mut g = MatrixGraph::new();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        assert_eq!(g.node_count(), 3);

        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        g.add_edge(c, a, ());
        assert_eq!(g.edge_count(), 3);

        g.clear();

        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);

        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 0);

        assert_eq!(g.neighbors_directed(a, Incoming).into_vec(), vec![]);
        assert_eq!(g.neighbors_directed(b, Incoming).into_vec(), vec![]);
        assert_eq!(g.neighbors_directed(c, Incoming).into_vec(), vec![]);

        assert_eq!(g.neighbors_directed(a, Outgoing).into_vec(), vec![]);
        assert_eq!(g.neighbors_directed(b, Outgoing).into_vec(), vec![]);
        assert_eq!(g.neighbors_directed(c, Outgoing).into_vec(), vec![]);
    }

    #[test]
    fn test_clear_undirected() {
        let mut g = MatrixGraph::new_undirected();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        assert_eq!(g.node_count(), 3);

        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        g.add_edge(c, a, ());
        assert_eq!(g.edge_count(), 3);

        g.clear();

        assert_eq!(g.node_count(), 0);
        assert_eq!(g.edge_count(), 0);

        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        assert_eq!(g.node_count(), 3);
        assert_eq!(g.edge_count(), 0);

        assert_eq!(g.neighbors(a).into_vec(), vec![]);
        assert_eq!(g.neighbors(b).into_vec(), vec![]);
        assert_eq!(g.neighbors(c).into_vec(), vec![]);
    }

    /// Helper trait for always sorting before testing.
    trait IntoSortedVec<T> {
        fn into_sorted_vec(self) -> Vec<T>;
    }

    impl<It, T> IntoSortedVec<T> for It
        where It: Iterator<Item=T>,
            T: Ord,
    {
        fn into_sorted_vec(self) -> Vec<T> {
            let mut v: Vec<T> = self.collect();
            v.sort();
            v
        }
    }

    /// Helper macro for always sorting before testing.
    macro_rules! sorted_vec {
        ($($x:expr),*) => {
            {
                let mut v = vec![$($x,)*];
                v.sort();
                v
            }
        }
    }

    #[test]
    fn test_neighbors() {
        let mut g = MatrixGraph::new();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        g.add_edge(a, b, ());
        g.add_edge(a, c, ());

        let a_neighbors = g.neighbors(a).into_sorted_vec();
        assert_eq!(a_neighbors, sorted_vec![b, c]);

        let b_neighbors = g.neighbors(b).into_sorted_vec();
        assert_eq!(b_neighbors, vec![]);

        let c_neighbors = g.neighbors(c).into_sorted_vec();
        assert_eq!(c_neighbors, vec![]);
    }

    #[test]
    fn test_neighbors_undirected() {
        let mut g = MatrixGraph::new_undirected();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        g.add_edge(a, b, ());
        g.add_edge(a, c, ());

        let a_neighbors = g.neighbors(a).into_sorted_vec();
        assert_eq!(a_neighbors, sorted_vec![b, c]);

        let b_neighbors = g.neighbors(b).into_sorted_vec();
        assert_eq!(b_neighbors, sorted_vec![a]);

        let c_neighbors = g.neighbors(c).into_sorted_vec();
        assert_eq!(c_neighbors, sorted_vec![a]);
    }

    #[test]
    fn test_remove_node_and_edges() {
        let mut g = MatrixGraph::new();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        g.add_edge(c, a, ());

        // removing b should break the `a -> b` and `b -> c` edges
        g.remove_node(b);

        assert_eq!(g.node_count(), 2);

        let a_neighbors = g.neighbors(a).into_sorted_vec();
        assert_eq!(a_neighbors, vec![]);

        let c_neighbors = g.neighbors(c).into_sorted_vec();
        assert_eq!(c_neighbors, vec![a]);
    }

    #[test]
    fn test_remove_node_and_edges_undirected() {
        let mut g = UnMatrix::new_undirected();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        g.add_edge(c, a, ());

        // removing a should break the `a - b` and `a - c` edges
        g.remove_node(a);

        assert_eq!(g.node_count(), 2);

        let b_neighbors = g.neighbors(b).into_sorted_vec();
        assert_eq!(b_neighbors, vec![c]);

        let c_neighbors = g.neighbors(c).into_sorted_vec();
        assert_eq!(c_neighbors, vec![b]);
    }

    #[test]
    fn test_node_identifiers() {
        let mut g = MatrixGraph::new();
        let a = g.add_node('a');
        let b = g.add_node('b');
        let c = g.add_node('c');
        let d = g.add_node('c');
        g.add_edge(a, b, ());
        g.add_edge(a, c, ());

        let node_ids = g.node_identifiers().into_sorted_vec();
        assert_eq!(node_ids, sorted_vec![a, b, c, d]);
    }

    #[test]
    fn test_edges_directed() {
        let g: MatrixGraph<char, bool> = MatrixGraph::from_edges(&[
            (0, 5), (0, 2), (0, 3), (0, 1),
            (1, 3),
            (2, 3), (2, 4),
            (4, 0),
            (6, 6),
        ]);

        assert_eq!(g.edges_directed(node_index(0), Outgoing).count(), 4);
        assert_eq!(g.edges_directed(node_index(1), Outgoing).count(), 1);
        assert_eq!(g.edges_directed(node_index(2), Outgoing).count(), 2);
        assert_eq!(g.edges_directed(node_index(3), Outgoing).count(), 0);
        assert_eq!(g.edges_directed(node_index(4), Outgoing).count(), 1);
        assert_eq!(g.edges_directed(node_index(5), Outgoing).count(), 0);
        assert_eq!(g.edges_directed(node_index(6), Outgoing).count(), 1);

        assert_eq!(g.edges_directed(node_index(0), Incoming).count(), 1);
        assert_eq!(g.edges_directed(node_index(1), Incoming).count(), 1);
        assert_eq!(g.edges_directed(node_index(2), Incoming).count(), 1);
        assert_eq!(g.edges_directed(node_index(3), Incoming).count(), 3);
        assert_eq!(g.edges_directed(node_index(4), Incoming).count(), 1);
        assert_eq!(g.edges_directed(node_index(5), Incoming).count(), 1);
        assert_eq!(g.edges_directed(node_index(6), Incoming).count(), 1);
    }

    #[test]
    fn test_edges_undirected() {
        let g: UnMatrix<char, bool> = UnMatrix::from_edges(&[
            (0, 5), (0, 2), (0, 3), (0, 1),
            (1, 3),
            (2, 3), (2, 4),
            (4, 0),
            (6, 6),
        ]);

        assert_eq!(g.edges(node_index(0)).count(), 5);
        assert_eq!(g.edges(node_index(1)).count(), 2);
        assert_eq!(g.edges(node_index(2)).count(), 3);
        assert_eq!(g.edges(node_index(3)).count(), 3);
        assert_eq!(g.edges(node_index(4)).count(), 2);
        assert_eq!(g.edges(node_index(5)).count(), 1);
        assert_eq!(g.edges(node_index(6)).count(), 1);
    }

    #[test]
    fn test_edge_references() {
        let g: MatrixGraph<char, bool> = MatrixGraph::from_edges(&[
            (0, 5), (0, 2), (0, 3), (0, 1),
            (1, 3),
            (2, 3), (2, 4),
            (4, 0),
            (6, 6),
        ]);

        assert_eq!(g.edge_references().count(), 9);
    }

    #[test]
    fn test_edge_references_undirected() {
        let g: UnMatrix<char, bool> = UnMatrix::from_edges(&[
            (0, 5), (0, 2), (0, 3), (0, 1),
            (1, 3),
            (2, 3), (2, 4),
            (4, 0),
            (6, 6),
        ]);

        assert_eq!(g.edge_references().count(), 9);
    }

    #[test]
    fn test_id_storage() {
        use super::IdStorage;

        let mut storage: IdStorage<char> = IdStorage::with_capacity(0);
        let a = storage.add('a');
        let b = storage.add('b');
        let c = storage.add('c');

        assert!(a < b && b < c);

        // list IDs
        assert_eq!(storage.iter_ids().into_vec(), vec![a, b, c]);

        storage.remove(b);

        // re-use of IDs
        let bb = storage.add('B');
        assert_eq!(b, bb);

        // list IDs
        assert_eq!(storage.iter_ids().into_vec(), vec![a, b, c]);
    }

    #[test]
    fn test_not_zero() {
        let mut g: MatrixGraph<(), i32, Directed, NotZero<i32>> = MatrixGraph::default();

        let a = g.add_node(());
        let b = g.add_node(());

        assert!(!g.has_edge(a, b));
        assert_eq!(g.edge_count(), 0);

        g.add_edge(a, b, 12);

        assert!(g.has_edge(a, b));
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.edge_weight(a, b), &12);

        g.remove_edge(a, b);

        assert!(!g.has_edge(a, b));
        assert_eq!(g.edge_count(), 0);
    }

    #[test]
    #[should_panic]
    fn test_not_zero_asserted() {
        let mut g: MatrixGraph<(), i32, Directed, NotZero<i32>> = MatrixGraph::default();

        let a = g.add_node(());
        let b = g.add_node(());

        g.add_edge(a, b, 0); // this should trigger an assertion
    }

    #[test]
    fn test_not_zero_float() {
        let mut g: MatrixGraph<(), f32, Directed, NotZero<f32>> = MatrixGraph::default();

        let a = g.add_node(());
        let b = g.add_node(());

        assert!(!g.has_edge(a, b));
        assert_eq!(g.edge_count(), 0);

        g.add_edge(a, b, 12.);

        assert!(g.has_edge(a, b));
        assert_eq!(g.edge_count(), 1);
        assert_eq!(g.edge_weight(a, b), &12.);

        g.remove_edge(a, b);

        assert!(!g.has_edge(a, b));
        assert_eq!(g.edge_count(), 0);
    }
}