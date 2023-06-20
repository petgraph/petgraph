//! Simple adjacency list.
#![cfg_attr(not(feature = "std"), no_std)]

mod utils;

extern crate alloc;

use alloc::{boxed::Box, vec::Vec};
use core::{
    fmt,
    fmt::{Display, Formatter},
    iter, mem,
    ops::Range,
    slice,
};

use fixedbitset::FixedBitSet;
use funty::Fundamental;
use petgraph_core::{
    data::{Build, DataMap, DataMapMut},
    edge::Directed,
    index::{DefaultIx, FromIndexType, IndexType, IntoIndexType, SafeCast},
    iterator_wrap, visit,
    visit::{EdgeCount, EdgeRef, GetAdjacencyMatrix, IntoEdgeReferences, IntoNeighbors, NodeCount},
};

use crate::utils::DebugFn;

/// Adjacency list node index type, a plain integer.
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

impl Display for NodeIndex {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0.as_usize(), f)
    }
}

/// Adjacency list edge index type, a pair of integers.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeIndex<Ix = DefaultIx>
where
    Ix: IndexType,
{
    /// Source of the edge.
    from: NodeIndex<Ix>,
    /// Index of the sucessor in the successor list.
    successor_index: usize,
}

iterator_wrap! {
impl (Iterator) for
/// An Iterator over the indices of the outgoing edges from a node.
///
/// It does not borrow the graph during iteration.
#[derive(Debug, Clone)]
struct OutgoingEdgeIndices <Ix> where { Ix: IndexType }
item: EdgeIndex<Ix>,
iter: iter::Map<iter::Zip<Range<usize>, iter::Repeat<NodeIndex<Ix>>>, fn((usize, NodeIndex<Ix>)) -> EdgeIndex<Ix>>,
}

/// Weighted sucessor
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct WSuc<E, Ix: IndexType> {
    /// Index of the sucessor.
    suc: NodeIndex<Ix>,
    /// Weight of the edge to `suc`.
    weight: E,
}

/// One row of the adjacency list.
type Row<E, Ix> = Vec<WSuc<E, Ix>>;
type RowIter<'a, E, Ix> = slice::Iter<'a, WSuc<E, Ix>>;

iterator_wrap! {
impl (Iterator DoubleEndedIterator ExactSizeIterator) for
/// An iterator over the indices of the neighbors of a node.
#[derive(Debug, Clone)]
struct Neighbors<'a, E, Ix> where { Ix: IndexType }
item: NodeIndex<Ix>,
iter: iter::Map<RowIter<'a, E, Ix>, fn(&WSuc<E, Ix>) -> NodeIndex<Ix>>,
}

/// A reference to an edge of the graph.
#[derive(Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct EdgeReference<'a, E, Ix: IndexType> {
    /// index of the edge
    id: EdgeIndex<Ix>,
    /// a reference to the corresponding item in the adjacency list
    edge: &'a WSuc<E, Ix>,
}

impl<'a, E, Ix: IndexType> Copy for EdgeReference<'a, E, Ix> {}
impl<'a, E, Ix: IndexType> Clone for EdgeReference<'a, E, Ix> {
    fn clone(&self) -> Self {
        EdgeReference {
            id: self.id,
            edge: self.edge,
        }
    }
}

impl<'a, E, Ix: IndexType> visit::EdgeRef for EdgeReference<'a, E, Ix> {
    type EdgeId = EdgeIndex<Ix>;
    type NodeId = NodeIndex<Ix>;
    type Weight = E;

    fn source(&self) -> Self::NodeId {
        self.id.from
    }

    fn target(&self) -> Self::NodeId {
        self.edge.suc
    }

    fn id(&self) -> Self::EdgeId {
        self.id
    }

    fn weight(&self) -> &Self::Weight {
        &self.edge.weight
    }
}

#[derive(Debug, Clone)]
pub struct EdgeIndices<'a, E, Ix: IndexType> {
    rows: iter::Enumerate<slice::Iter<'a, Row<E, Ix>>>,
    row_index: usize,
    row_len: usize,
    cur: usize,
}

impl<'a, E, Ix: IndexType> Iterator for EdgeIndices<'a, E, Ix> {
    type Item = EdgeIndex<Ix>;

    fn next(&mut self) -> Option<EdgeIndex<Ix>> {
        loop {
            if self.cur < self.row_len {
                let res = self.cur;
                self.cur += 1;
                return Some(EdgeIndex {
                    from: NodeIndex(Ix::from_usize(self.row_index)),
                    successor_index: res,
                });
            } else {
                match self.rows.next() {
                    Some((index, row)) => {
                        self.row_index = index;
                        self.cur = 0;
                        self.row_len = row.len();
                    }
                    None => return None,
                }
            }
        }
    }
}

iterator_wrap! {
    impl (Iterator DoubleEndedIterator ExactSizeIterator) for
    /// An iterator over all node indices in the graph.
    #[derive(Debug, Clone)]
    struct NodeIndices <Ix> where {}
    item: NodeIndex<Ix>,
    iter: iter::Map<Range<usize>, fn(usize) -> NodeIndex<Ix>>,
}

/// An adjacency list with labeled edges.
///
/// Can be interpreted as a directed graph
/// with unweighted nodes.
///
/// This is the most simple adjacency list you can imagine. [`Graph`](../graph/struct.Graph.html),
/// in contrast, maintains both the list of successors and predecessors for each node,
/// which is a different trade-off.
///
/// Allows parallel edges and self-loops.
///
/// This data structure is append-only (except for [`clear`](#method.clear)), so indices
/// returned at some point for a given graph will stay valid with this same
/// graph until it is dropped or [`clear`](#method.clear) is called.
///
/// Space consumption: **O(|E|)**.
#[derive(Clone, Default)]
pub struct AdjacencyList<E, Ix = DefaultIx>
where
    Ix: IndexType,
{
    suc: Vec<Row<E, Ix>>,
}

impl<E, Ix: IndexType> AdjacencyList<E, Ix> {
    /// Creates a new, empty adjacency list.
    pub fn new() -> AdjacencyList<E, Ix> {
        AdjacencyList { suc: Vec::new() }
    }

    /// Creates a new, empty adjacency list tailored for `nodes` nodes.
    pub fn with_capacity(nodes: usize) -> AdjacencyList<E, Ix> {
        AdjacencyList {
            suc: Vec::with_capacity(nodes),
        }
    }

    /// Removes all nodes and edges from the list.
    pub fn clear(&mut self) {
        self.suc.clear()
    }

    /// Returns the number of edges in the list
    ///
    /// Computes in **O(|V|)** time.
    pub fn edge_count(&self) -> usize {
        self.suc.iter().map(|x| x.len()).sum()
    }

    /// Adds a new node to the list. This allocates a new `Vec` and then should
    /// run in amortized **O(1)** time.
    pub fn add_node(&mut self) -> NodeIndex<Ix> {
        let i = self.suc.len();
        self.suc.push(Vec::new());
        NodeIndex(Ix::from_usize(i))
    }

    /// Adds a new node to the list. This allocates a new `Vec` and then should
    /// run in amortized **O(1)** time.
    pub fn add_node_with_capacity(&mut self, successors: usize) -> NodeIndex<Ix> {
        let i = self.suc.len();
        self.suc.push(Vec::with_capacity(successors));
        NodeIndex(Ix::from_usize(i))
    }

    /// Adds a new node to the list by giving its list of successors and the corresponding
    /// weigths.
    pub fn add_node_from_edges<I: Iterator<Item = (NodeIndex<Ix>, E)>>(
        &mut self,
        edges: I,
    ) -> NodeIndex<Ix> {
        let i = self.suc.len();
        self.suc
            .push(edges.map(|(suc, weight)| WSuc { suc, weight }).collect());
        NodeIndex(Ix::from_usize(i))
    }

    /// Add an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(1)** time.
    ///
    /// **Panics** if the source node does not exist.<br>
    ///
    /// **Note:** `List` allows adding parallel (“duplicate”) edges. If you want
    /// to avoid this, use [`.update_edge(a, b, weight)`](#method.update_edge) instead.
    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
        if b.into_usize() >= self.suc.len() {
            panic!(
                "{} is not a valid node index for a {} nodes adjacency list",
                b.into_usize(),
                self.suc.len()
            );
        }
        let row = &mut self.suc[a.into_usize()];
        let rank = row.len();
        row.push(WSuc { suc: b, weight });
        EdgeIndex {
            from: a,
            successor_index: rank,
        }
    }

    fn get_edge(&self, e: EdgeIndex<Ix>) -> Option<&WSuc<E, Ix>> {
        self.suc
            .get(e.from.into_usize())
            .and_then(|row| row.get(e.successor_index))
    }

    fn get_edge_mut(&mut self, e: EdgeIndex<Ix>) -> Option<&mut WSuc<E, Ix>> {
        self.suc
            .get_mut(e.from.into_usize())
            .and_then(|row| row.get_mut(e.successor_index))
    }

    /// Accesses the source and target of edge `e`
    ///
    /// Computes in **O(1)**
    pub fn edge_endpoints(&self, e: EdgeIndex<Ix>) -> Option<(NodeIndex<Ix>, NodeIndex<Ix>)> {
        self.get_edge(e).map(|x| (e.from, x.suc))
    }

    pub fn edge_indices_from(&self, a: NodeIndex<Ix>) -> OutgoingEdgeIndices<Ix> {
        let proj: fn((usize, NodeIndex<Ix>)) -> EdgeIndex<Ix> =
            |(successor_index, from)| EdgeIndex {
                from,
                successor_index,
            };
        let iter = (0..(self.suc[a.into_usize()].len()))
            .zip(iter::repeat(a))
            .map(proj);
        OutgoingEdgeIndices { iter }
    }

    /// Lookups whether there is an edge from `a` to `b`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of successors of `a`.
    pub fn contains_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        match self.suc.get(a.into_usize()) {
            None => false,
            Some(row) => row.iter().any(|x| x.suc == b),
        }
    }

    /// Lookups whether there is an edge from `a` to `b`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of successors of `a`.
    pub fn find_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<EdgeIndex<Ix>> {
        self.suc.get(a.into_usize()).and_then(|row| {
            row.iter()
                .enumerate()
                .find(|(_, x)| x.suc == b)
                .map(|(i, _)| EdgeIndex {
                    from: a,
                    successor_index: i,
                })
        })
    }

    /// Returns an iterator over all node indices of the graph.
    ///
    /// Consuming the whole iterator take **O(|V|)**.
    pub fn node_indices(&self) -> NodeIndices<Ix> {
        NodeIndices {
            iter: (0..self.suc.len()).map(NodeIndex::from_usize),
        }
    }

    /// Returns an iterator over all edge indices of the graph.
    ///
    /// Consuming the whole iterator take **O(|V| + |E|)**.
    pub fn edge_indices(&self) -> EdgeIndices<E, Ix> {
        EdgeIndices {
            rows: self.suc.iter().enumerate(),
            row_index: 0,
            row_len: 0,
            cur: 0,
        }
    }
}

/// A very simple adjacency list with no node or label weights.
pub type UnweightedAdjacencyList<Ix> = AdjacencyList<(), Ix>;

impl<E, Ix: IndexType> Build for AdjacencyList<E, Ix> {
    /// Adds a new node to the list. This allocates a new `Vec` and then should
    /// run in amortized **O(1)** time.
    fn add_node(&mut self, _weight: ()) -> NodeIndex<Ix> {
        self.add_node()
    }

    /// Add an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(1)** time.
    ///
    /// **Panics** if the source node does not exist.<br>
    ///
    /// **Note:** `List` allows adding parallel (“duplicate”) edges. If you want
    /// to avoid this, use [`.update_edge(a, b, weight)`](#method.update_edge) instead.
    fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> Option<EdgeIndex<Ix>> {
        Some(self.add_edge(a, b, weight))
    }

    /// Updates or adds an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of successors of `a`.
    ///
    /// **Panics** if the source node does not exist.<br>
    fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
        let row = &mut self.suc[a.into_usize()];
        for (i, info) in row.iter_mut().enumerate() {
            if info.suc == b {
                info.weight = weight;
                return EdgeIndex {
                    from: a,
                    successor_index: i,
                };
            }
        }
        let rank = row.len();
        row.push(WSuc { suc: b, weight });
        EdgeIndex {
            from: a,
            successor_index: rank,
        }
    }
}

impl<'a, E, Ix> fmt::Debug for EdgeReferences<'a, E, Ix>
where
    E: fmt::Debug,
    Ix: IndexType,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut edge_list = f.debug_list();
        let iter: Self = self.clone();
        for e in iter {
            if mem::size_of::<E>() != 0 {
                edge_list.entry(&(
                    DebugFn(|f| {
                        f.write_fmt(format_args!(
                            "{:?}",
                            (e.source().into_usize(), e.target().into_usize())
                        ))
                    }),
                    e.weight(),
                ));
            } else {
                edge_list.entry(&DebugFn(|f| {
                    f.write_fmt(format_args!(
                        "{:?}",
                        (e.source().into_usize(), e.target().into_usize())
                    ))
                }));
            }
        }
        edge_list.finish()
    }
}

impl<E, Ix> fmt::Debug for AdjacencyList<E, Ix>
where
    E: fmt::Debug,
    Ix: IndexType,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut fmt_struct = f.debug_struct("adj::List");
        fmt_struct.field("node_count", &self.node_count());
        fmt_struct.field("edge_count", &self.edge_count());
        if self.edge_count() > 0 {
            fmt_struct.field("edges", &self.edge_references());
        }
        fmt_struct.finish()
    }
}

impl<E, Ix> visit::GraphBase for AdjacencyList<E, Ix>
where
    Ix: IndexType,
{
    type EdgeId = EdgeIndex<Ix>;
    type NodeId = NodeIndex<Ix>;
}

impl<E, Ix> visit::Visitable for AdjacencyList<E, Ix>
where
    Ix: IndexType,
{
    type Map = FixedBitSet;

    fn visit_map(&self) -> FixedBitSet {
        FixedBitSet::with_capacity(self.node_count())
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.clear();
        map.grow(self.node_count());
    }
}

impl<'a, E, Ix: IndexType> visit::IntoNodeIdentifiers for &'a AdjacencyList<E, Ix> {
    type NodeIdentifiers = NodeIndices<Ix>;

    fn node_identifiers(self) -> NodeIndices<Ix> {
        self.node_indices()
    }
}

impl<Ix: IndexType> visit::NodeRef for NodeIndex<Ix> {
    type NodeId = NodeIndex<Ix>;
    type Weight = ();

    fn id(&self) -> Self::NodeId {
        *self
    }

    fn weight(&self) -> &Self::Weight {
        &()
    }
}

impl<'a, Ix: IndexType, E> visit::IntoNodeReferences for &'a AdjacencyList<E, Ix> {
    type NodeRef = NodeIndex<Ix>;
    type NodeReferences = NodeIndices<Ix>;

    fn node_references(self) -> Self::NodeReferences {
        self.node_indices()
    }
}

impl<E, Ix: IndexType> visit::Data for AdjacencyList<E, Ix> {
    type EdgeWeight = E;
    type NodeWeight = ();
}

impl<'a, E, Ix: IndexType> IntoNeighbors for &'a AdjacencyList<E, Ix> {
    type Neighbors = Neighbors<'a, E, Ix>;

    /// Returns an iterator of all nodes with an edge starting from `a`.
    /// Panics if `a` is out of bounds.
    /// Use [`AdjacencyList::edge_indices_from`] instead if you do not want to borrow the
    /// adjacency list while iterating.
    fn neighbors(self, a: NodeIndex<Ix>) -> Self::Neighbors {
        let proj: fn(&WSuc<E, Ix>) -> NodeIndex<Ix> = |x| x.suc;
        let iter = self.suc[a.into_usize()].iter().map(proj);
        Neighbors { iter }
    }
}

type SomeIter<'a, E, Ix> = iter::Map<
    iter::Zip<iter::Enumerate<RowIter<'a, E, Ix>>, iter::Repeat<NodeIndex<Ix>>>,
    fn(((usize, &'a WSuc<E, Ix>), NodeIndex<Ix>)) -> EdgeReference<'a, E, Ix>,
>;

iterator_wrap! {
impl (Iterator) for
/// An iterator over the [`EdgeReference`] of all the edges of the graph.
struct EdgeReferences<'a, E, Ix> where { Ix: IndexType }
item: EdgeReference<'a, E, Ix>,
iter: iter::FlatMap<
    iter::Enumerate<
        slice::Iter<'a, Row<E, Ix>>
    >,
    SomeIter<'a, E, Ix>,
    fn(
        (usize, &'a Vec<WSuc<E, Ix>>)
    ) -> SomeIter<'a, E, Ix>,
>,
}

impl<'a, E, Ix: IndexType> Clone for EdgeReferences<'a, E, Ix> {
    fn clone(&self) -> Self {
        EdgeReferences {
            iter: self.iter.clone(),
        }
    }
}

fn proj1<E, Ix: IndexType>(
    ((successor_index, edge), from): ((usize, &WSuc<E, Ix>), NodeIndex<Ix>),
) -> EdgeReference<E, Ix> {
    let id = EdgeIndex {
        from,
        successor_index,
    };
    EdgeReference { id, edge }
}
fn proj2<E, Ix: IndexType>((row_index, row): (usize, &Vec<WSuc<E, Ix>>)) -> SomeIter<E, Ix> {
    row.iter()
        .enumerate()
        .zip(iter::repeat(NodeIndex::from_usize(row_index)))
        .map(proj1 as _)
}

impl<'a, Ix: IndexType, E> visit::IntoEdgeReferences for &'a AdjacencyList<E, Ix> {
    type EdgeRef = EdgeReference<'a, E, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ix>;

    fn edge_references(self) -> Self::EdgeReferences {
        let iter = self.suc.iter().enumerate().flat_map(proj2 as _);
        EdgeReferences { iter }
    }
}

iterator_wrap! {
impl (Iterator) for
/// Iterator over the [`EdgeReference`] of the outgoing edges from a node.
#[derive(Debug, Clone)]
struct OutgoingEdgeReferences<'a, E, Ix> where { Ix: IndexType }
item: EdgeReference<'a, E, Ix>,
iter: SomeIter<'a, E, Ix>,
}

impl<'a, Ix: IndexType, E> visit::IntoEdges for &'a AdjacencyList<E, Ix> {
    type Edges = OutgoingEdgeReferences<'a, E, Ix>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        let iter = self.suc[a.into_usize()]
            .iter()
            .enumerate()
            .zip(iter::repeat(a))
            .map(proj1 as _);
        OutgoingEdgeReferences { iter }
    }
}

impl<E, Ix: IndexType> visit::GraphProp for AdjacencyList<E, Ix> {
    type EdgeType = Directed;

    fn is_directed(&self) -> bool {
        true
    }
}

impl<E, Ix: IndexType> NodeCount for AdjacencyList<E, Ix> {
    /// Returns the number of nodes in the list
    ///
    /// Computes in **O(1)** time.
    fn node_count(&self) -> usize {
        self.suc.len()
    }
}

impl<E, Ix: IndexType> EdgeCount for AdjacencyList<E, Ix> {
    /// Returns the number of edges in the list
    ///
    /// Computes in **O(|V|)** time.
    fn edge_count(&self) -> usize {
        AdjacencyList::edge_count(self)
    }
}

impl<E, Ix: IndexType> visit::NodeIndexable for AdjacencyList<E, Ix> {
    fn node_bound(&self) -> usize {
        self.node_count()
    }

    #[inline]
    fn to_index(&self, a: Self::NodeId) -> usize {
        a.into_usize()
    }

    #[inline]
    fn from_index(&self, i: usize) -> Self::NodeId {
        NodeIndex::from_usize(i)
    }
}

impl<E, Ix: IndexType> visit::NodeCompactIndexable for AdjacencyList<E, Ix> {}

impl<E, Ix: IndexType> DataMap for AdjacencyList<E, Ix> {
    fn node_weight(&self, n: Self::NodeId) -> Option<&()> {
        if n.into_usize() < self.suc.len() {
            Some(&())
        } else {
            None
        }
    }

    /// Accesses the weight of edge `e`
    ///
    /// Computes in **O(1)**
    fn edge_weight(&self, e: EdgeIndex<Ix>) -> Option<&E> {
        self.get_edge(e).map(|x| &x.weight)
    }
}

impl<E, Ix: IndexType> DataMapMut for AdjacencyList<E, Ix> {
    fn node_weight_mut(&mut self, n: Self::NodeId) -> Option<&mut ()> {
        if n.into_usize() < self.suc.len() {
            // A hack to produce a &'static mut ()
            // It does not actually allocate according to godbolt
            let b = Box::new(());
            Some(Box::leak(b))
        } else {
            None
        }
    }

    /// Accesses the weight of edge `e`
    ///
    /// Computes in **O(1)**
    fn edge_weight_mut(&mut self, e: EdgeIndex<Ix>) -> Option<&mut E> {
        self.get_edge_mut(e).map(|x| &mut x.weight)
    }
}

/// The adjacency matrix for **List** is a bitmap that's computed by
/// `.adjacency_matrix()`.
impl<E, Ix> GetAdjacencyMatrix for AdjacencyList<E, Ix>
where
    Ix: IndexType,
{
    type AdjMatrix = FixedBitSet;

    fn adjacency_matrix(&self) -> FixedBitSet {
        let n = self.node_count();
        let mut matrix = FixedBitSet::with_capacity(n * n);
        for edge in self.edge_references() {
            let i = edge.source().into_usize() * n + edge.target().into_usize();
            matrix.put(i);

            let j = edge.source().into_usize() + n * edge.target().into_usize();
            matrix.put(j);
        }
        matrix
    }

    fn is_adjacent(&self, matrix: &FixedBitSet, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        let n = self.edge_count();
        let index = n * a.into_usize() + b.into_usize();
        matrix.contains(index)
    }
}
