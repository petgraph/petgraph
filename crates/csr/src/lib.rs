//! Compressed Sparse Row (CSR) is a sparse adjacency matrix graph.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

extern crate alloc;

use alloc::{vec, vec::Vec};
use core::{
    cmp::{max, Ordering},
    iter,
    marker::PhantomData,
    ops::{Index, IndexMut, Range},
    slice,
};

use fixedbitset::FixedBitSet;
use petgraph_core::{
    deprecated::IntoWeightedEdge,
    edge::{Directed, EdgeType},
    index::{DefaultIx, FromIndexType, IndexType, IntoIndexType, SafeCast},
    visit::{
        Data, EdgeCount, EdgeRef, GetAdjacencyMatrix, GraphBase, GraphProp, IntoEdgeReferences,
        IntoEdges, IntoNeighbors, IntoNodeIdentifiers, IntoNodeReferences, NodeCompactIndexable,
        NodeCount, NodeIndexable, Visitable,
    },
};

/// Csr node index type, a plain integer.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NodeIndex<Ix = DefaultIx>(Ix);

impl<Ix> NodeIndex<Ix>
where
    Ix: IndexType,
{
    const fn new(value: Ix) -> Self {
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

impl<Ix> From<Ix> for NodeIndex<Ix>
where
    Ix: IndexType,
{
    fn from(value: Ix) -> Self {
        Self(value)
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

/// Csr edge index type, a plain integer.
// TODO: move to newtype?
pub type EdgeIndex = usize;

const BINARY_SEARCH_CUTOFF: usize = 32;

/// Compressed Sparse Row ([`CSR`]) is a sparse adjacency matrix graph.
///
/// `CSR` is parameterized over:
///
/// - Associated data `N` for nodes and `E` for edges, called *weights*. The associated data can be
///   of arbitrary type.
/// - Edge type `Ty` that determines whether the graph edges are directed or undirected.
/// - Index type `Ix`, which determines the maximum size of the graph.
///
///
/// Using **O(|E| + |V|)** space.
///
/// Self loops are allowed, no parallel edges.
///
/// Fast iteration of the outgoing edges of a vertex.
///
/// [`CSR`]: https://en.wikipedia.org/wiki/Sparse_matrix#Compressed_sparse_row_(CSR,_CRS_or_Yale_format)
#[derive(Debug)]
pub struct Csr<N = (), E = (), Ty = Directed, Ix = DefaultIx> {
    /// Column of next edge
    column: Vec<NodeIndex<Ix>>,
    /// weight of each edge; lock step with column
    edges: Vec<E>,
    /// Index of start of row Always node_count + 1 long.
    /// Last element is always equal to column.len()
    row: Vec<usize>,
    node_weights: Vec<N>,
    edge_count: usize,
    ty: PhantomData<Ty>,
}

impl<N, E, Ty, Ix> Default for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N: Clone, E: Clone, Ty, Ix: Clone> Clone for Csr<N, E, Ty, Ix> {
    fn clone(&self) -> Self {
        Csr {
            column: self.column.clone(),
            edges: self.edges.clone(),
            row: self.row.clone(),
            node_weights: self.node_weights.clone(),
            edge_count: self.edge_count,
            ty: self.ty,
        }
    }
}

impl<N, E, Ty, Ix> Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    /// Create an empty `Csr`.
    pub fn new() -> Self {
        Csr {
            column: vec![],
            edges: vec![],
            row: vec![0; 1],
            node_weights: vec![],
            edge_count: 0,
            ty: PhantomData,
        }
    }

    /// Create a new `Csr` with `n` nodes. `N` must implement [`Default`] for the weight of each
    /// node.
    ///
    /// [`Default`]: https://doc.rust-lang.org/nightly/core/default/trait.Default.html
    ///
    /// # Example
    /// ```rust
    /// use petgraph_csr::Csr;
    ///
    /// let graph = Csr::<u8, ()>::with_nodes(5);
    /// assert_eq!(graph.node_count(), 5);
    /// assert_eq!(graph.edge_count(), 0);
    ///
    /// assert_eq!(graph[0], 0);
    /// assert_eq!(graph[4], 0);
    /// ```
    pub fn with_nodes(n: usize) -> Self
    where
        N: Default,
    {
        Csr {
            column: Vec::new(),
            edges: Vec::new(),
            row: vec![0; n + 1],
            node_weights: (0..n).map(|_| N::default()).collect(),
            edge_count: 0,
            ty: PhantomData,
        }
    }
}

/// Csr creation error: edges were not in sorted order.
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct EdgesNotSorted {
    pub(crate) first_error: (usize, usize),
}

impl<N, E, Ix> Csr<N, E, Directed, Ix>
where
    Ix: IndexType,
{
    /// Create a new `Csr` from a sorted sequence of edges
    ///
    /// Edges **must** be sorted and unique, where the sort order is the default
    /// order for the pair *(u, v)* in Rust (*u* has priority).
    ///
    /// Computes in **O(|E| + |V|)** time.
    /// # Example
    /// ```rust
    /// use petgraph_csr::Csr;
    ///
    /// let graph =
    ///     Csr::<(), ()>::from_sorted_edges(&[(0, 1), (0, 2), (1, 0), (1, 2), (1, 3), (2, 0), (3, 1)]);
    /// ```
    pub fn from_sorted_edges<Edge>(edges: &[Edge]) -> Result<Self, EdgesNotSorted>
    where
        Edge: Clone + IntoWeightedEdge<E, NodeId = NodeIndex<Ix>>,
        N: Default,
    {
        let max_node_id = match edges
            .iter()
            .map(|edge| {
                let (x, y, _) = edge.clone().into_weighted_edge();
                max(x.cast(), y.cast())
            })
            .max()
        {
            None => return Ok(Self::with_nodes(0)),
            Some(x) => x,
        };
        let mut self_ = Self::with_nodes(max_node_id + 1);
        let mut iter = edges.iter().cloned().peekable();
        {
            let mut rows = self_.row.iter_mut();

            let mut rstart = 0;
            let mut last_target;
            'outer: for (node, r) in (&mut rows).enumerate() {
                *r = rstart;
                last_target = None;
                'inner: loop {
                    if let Some(edge) = iter.peek() {
                        let (n, m, weight) = edge.clone().into_weighted_edge();
                        // check that the edges are in increasing sequence
                        if node > n.cast() {
                            return Err(EdgesNotSorted {
                                first_error: (n.cast(), m.cast()),
                            });
                        }
                        /*
                        debug_assert!(node <= n.index(),
                                      concat!("edges are not sorted, ",
                                              "failed assertion source {:?} <= {:?} ",
                                              "for edge {:?}"),
                                      node, n, (n, m));
                                      */
                        if n.cast() != node {
                            break 'inner;
                        }
                        // check that the edges are in increasing sequence
                        /*
                        debug_assert!(last_target.map_or(true, |x| m > x),
                                      "edges are not sorted, failed assertion {:?} < {:?}",
                                      last_target, m);
                                      */
                        if !last_target.map_or(true, |x| m > x) {
                            return Err(EdgesNotSorted {
                                first_error: (n.cast(), m.cast()),
                            });
                        }
                        last_target = Some(m);
                        self_.column.push(m);
                        self_.edges.push(weight);
                        rstart += 1;
                    } else {
                        break 'outer;
                    }
                    iter.next();
                }
            }
            for r in rows {
                *r = rstart;
            }
        }

        Ok(self_)
    }
}

impl<N, E, Ty, Ix> Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    pub fn node_count(&self) -> usize {
        self.row.len() - 1
    }

    pub fn edge_count(&self) -> usize {
        if self.is_directed() {
            self.column.len()
        } else {
            self.edge_count
        }
    }

    pub fn is_directed(&self) -> bool {
        Ty::is_directed()
    }

    /// Remove all edges
    pub fn clear_edges(&mut self) {
        self.column.clear();
        self.edges.clear();
        for r in &mut self.row {
            *r = 0;
        }
        if !self.is_directed() {
            self.edge_count = 0;
        }
    }

    /// Adds a new node with the given weight, returning the corresponding node index.
    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let i = self.row.len() - 1;
        self.row.insert(i, self.column.len());
        self.node_weights.insert(i, weight);
        NodeIndex::new(Ix::from_usize(i))
    }

    /// Return `true` if the edge was added
    ///
    /// If you add all edges in row-major order, the time complexity
    /// is **O(|V|·|E|)** for the whole operation.
    ///
    /// **Panics** if `a` or `b` are out of bounds.
    // We relax the bounds here from `NodeIndex<Ix>` to `Into<NodeIx>`, as `a` and `b` do not need
    // to pre-exist.
    pub fn add_edge(
        &mut self,
        a: impl Into<NodeIndex<Ix>>,
        b: impl Into<NodeIndex<Ix>>,
        weight: E,
    ) -> bool
    where
        E: Clone,
    {
        let a = a.into();
        let b = b.into();

        let ret = self.add_edge_(a, b, weight.clone());
        if ret && !self.is_directed() {
            self.edge_count += 1;
        }
        if ret && !self.is_directed() && a != b {
            let _ret2 = self.add_edge_(b, a, weight);
            debug_assert_eq!(ret, _ret2);
        }
        ret
    }

    // Return false if the edge already exists
    fn add_edge_(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> bool {
        assert!(a.cast() < self.node_count() && b.cast() < self.node_count());
        // a x b is at (a, b) in the matrix

        // find current range of edges from a
        let pos = match self.find_edge_pos(a, b) {
            Ok(_) => return false, /* already exists */
            Err(i) => i,
        };
        self.column.insert(pos, b);
        self.edges.insert(pos, weight);
        // update row vector
        for r in &mut self.row[a.cast() + 1..] {
            *r += 1;
        }
        true
    }

    fn find_edge_pos(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Result<usize, usize> {
        let (index, neighbors) = self.neighbors_of(a);
        if neighbors.len() < BINARY_SEARCH_CUTOFF {
            for (i, elt) in neighbors.iter().enumerate() {
                match elt.cmp(&b) {
                    Ordering::Equal => return Ok(i + index),
                    Ordering::Greater => return Err(i + index),
                    Ordering::Less => {}
                }
            }
            Err(neighbors.len() + index)
        } else {
            match neighbors.binary_search(&b) {
                Ok(i) => Ok(i + index),
                Err(i) => Err(i + index),
            }
        }
    }

    /// Computes in **O(log |V|)** time.
    ///
    /// **Panics** if the node `a` does not exist.
    pub fn contains_edge(&self, a: NodeIndex<Ix>, b: impl Into<NodeIndex<Ix>>) -> bool {
        self.find_edge_pos(a, b.into()).is_ok()
    }

    fn neighbors_range(&self, a: NodeIndex<Ix>) -> Range<usize> {
        let index = self.row[a.cast()];
        let end = self
            .row
            .get(a.cast() + 1)
            .cloned()
            .unwrap_or_else(|| self.column.len());
        index..end
    }

    fn neighbors_of(&self, a: NodeIndex<Ix>) -> (usize, &[NodeIndex<Ix>]) {
        let r = self.neighbors_range(a);
        (r.start, &self.column[r])
    }

    /// Computes in **O(1)** time.
    ///
    /// **Panics** if the node `a` does not exist.
    pub fn out_degree(&self, a: NodeIndex<Ix>) -> usize {
        let r = self.neighbors_range(a.into());
        r.end - r.start
    }

    /// Computes in **O(1)** time.
    ///
    /// **Panics** if the node `a` does not exist.
    pub fn neighbors_slice(&self, a: NodeIndex<Ix>) -> &[NodeIndex<Ix>] {
        self.neighbors_of(a.into()).1
    }

    /// Computes in **O(1)** time.
    ///
    /// **Panics** if the node `a` does not exist.
    pub fn edges_slice(&self, a: NodeIndex<Ix>) -> &[E] {
        &self.edges[self.neighbors_range(a.into())]
    }

    /// Return an iterator of all edges of `a`.
    ///
    /// - `Directed`: Outgoing edges from `a`.
    /// - `Undirected`: All edges connected to `a`.
    ///
    /// **Panics** if the node `a` does not exist.<br>
    /// Iterator element type is `EdgeReference<E, Ty, Ix>`.
    pub fn edges(&self, a: NodeIndex<Ix>) -> Edges<E, Ty, Ix> {
        let r = self.neighbors_range(a);
        Edges {
            index: r.start,
            source: a,
            iter: iter::zip(&self.column[r.clone()], &self.edges[r]),
            ty: self.ty,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Edges<'a, E: 'a, Ty = Directed, Ix: 'a = DefaultIx> {
    index: usize,
    source: NodeIndex<Ix>,
    iter: iter::Zip<slice::Iter<'a, NodeIndex<Ix>>, slice::Iter<'a, E>>,
    ty: PhantomData<Ty>,
}

#[derive(Debug)]
pub struct EdgeReference<'a, E: 'a, Ty, Ix: 'a = DefaultIx> {
    index: EdgeIndex,
    source: NodeIndex<Ix>,
    target: NodeIndex<Ix>,
    weight: &'a E,
    ty: PhantomData<Ty>,
}

impl<'a, E, Ty, Ix: Copy> Clone for EdgeReference<'a, E, Ty, Ix> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, E, Ty, Ix: Copy> Copy for EdgeReference<'a, E, Ty, Ix> {}

impl<'a, Ty, E, Ix> EdgeReference<'a, E, Ty, Ix>
where
    Ty: EdgeType,
{
    /// Access the edge’s weight.
    ///
    /// **NOTE** that this method offers a longer lifetime
    /// than the trait (unfortunately they don't match yet).
    pub fn weight(&self) -> &'a E {
        self.weight
    }
}

impl<'a, E, Ty, Ix> EdgeRef for EdgeReference<'a, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type EdgeId = EdgeIndex;
    type NodeId = NodeIndex<Ix>;
    type Weight = E;

    fn source(&self) -> Self::NodeId {
        self.source
    }

    fn target(&self) -> Self::NodeId {
        self.target
    }

    fn weight(&self) -> &E {
        self.weight
    }

    fn id(&self) -> Self::EdgeId {
        self.index
    }
}

impl<'a, E, Ty, Ix> Iterator for Edges<'a, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Item = EdgeReference<'a, E, Ty, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(move |(&j, w)| {
            let index = self.index;
            self.index += 1;
            EdgeReference {
                index,
                source: self.source,
                target: j,
                weight: w,
                ty: PhantomData,
            }
        })
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<N, E, Ty, Ix> Data for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type EdgeWeight = E;
    type NodeWeight = N;
}

impl<'a, N, E, Ty, Ix> IntoEdgeReferences for &'a Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type EdgeRef = EdgeReference<'a, E, Ty, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ty, Ix>;

    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences {
            index: 0,
            source_index: NodeIndex::new(Ix::from_usize(0)),
            edge_ranges: self.row.windows(2).enumerate(),
            column: &self.column,
            edges: &self.edges,
            iter: iter::zip(&[], &[]),
            ty: self.ty,
        }
    }
}

#[derive(Debug, Clone)]
pub struct EdgeReferences<'a, E: 'a, Ty, Ix: 'a> {
    source_index: NodeIndex<Ix>,
    index: usize,
    edge_ranges: iter::Enumerate<slice::Windows<'a, usize>>,
    column: &'a [NodeIndex<Ix>],
    edges: &'a [E],
    iter: iter::Zip<slice::Iter<'a, NodeIndex<Ix>>, slice::Iter<'a, E>>,
    ty: PhantomData<Ty>,
}

impl<'a, E, Ty, Ix> Iterator for EdgeReferences<'a, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Item = EdgeReference<'a, E, Ty, Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((&j, w)) = self.iter.next() {
                let index = self.index;
                self.index += 1;
                return Some(EdgeReference {
                    index,
                    source: self.source_index,
                    target: j,
                    weight: w,
                    ty: PhantomData,
                });
            }
            if let Some((i, w)) = self.edge_ranges.next() {
                let a = w[0];
                let b = w[1];
                self.iter = iter::zip(&self.column[a..b], &self.edges[a..b]);
                self.source_index = NodeIndex::new(Ix::from_usize(i));
            } else {
                return None;
            }
        }
    }
}

impl<'a, N, E, Ty, Ix> IntoEdges for &'a Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Edges = Edges<'a, E, Ty, Ix>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        self.edges(a)
    }
}

impl<N, E, Ty, Ix> GraphBase for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type EdgeId = EdgeIndex;
    type NodeId = NodeIndex<Ix>; // index into edges vector
}

impl<N, E, Ty, Ix> Visitable for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
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

#[derive(Clone, Debug)]
pub struct Neighbors<'a, Ix: 'a = DefaultIx> {
    iter: slice::Iter<'a, NodeIndex<Ix>>,
}

impl<'a, Ix> Iterator for Neighbors<'a, Ix>
where
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().cloned()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, N, E, Ty, Ix> IntoNeighbors for &'a Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Neighbors = Neighbors<'a, Ix>;

    /// Return an iterator of all neighbors of `a`.
    ///
    /// - `Directed`: Targets of outgoing edges from `a`.
    /// - `Undirected`: Opposing endpoints of all edges connected to `a`.
    ///
    /// **Panics** if the node `a` does not exist.<br>
    /// Iterator element type is `NodeIndex<Ix>`.
    fn neighbors(self, a: Self::NodeId) -> Self::Neighbors {
        Neighbors {
            iter: self.neighbors_slice(a).iter(),
        }
    }
}

impl<N, E, Ty, Ix> NodeIndexable for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn node_bound(&self) -> usize {
        self.node_count()
    }

    fn to_index(&self, a: Self::NodeId) -> usize {
        a.cast()
    }

    fn from_index(&self, ix: usize) -> Self::NodeId {
        NodeIndex::from_usize(ix)
    }
}

impl<N, E, Ty, Ix> NodeCompactIndexable for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
}

impl<N, E, Ty, Ix> Index<NodeIndex<Ix>> for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type Output = N;

    fn index(&self, ix: NodeIndex<Ix>) -> &N {
        &self.node_weights[ix.cast()]
    }
}

impl<N, E, Ty, Ix> IndexMut<NodeIndex<Ix>> for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn index_mut(&mut self, ix: NodeIndex<Ix>) -> &mut N {
        &mut self.node_weights[ix.cast()]
    }
}

#[derive(Debug, Clone)]
pub struct NodeIdentifiers<Ix = DefaultIx> {
    r: Range<usize>,
    ty: PhantomData<Ix>,
}

impl<Ix> Iterator for NodeIdentifiers<Ix>
where
    Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.r.next().map(Ix::from_usize).map(NodeIndex::new)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.r.size_hint()
    }
}

impl<'a, N, E, Ty, Ix> IntoNodeIdentifiers for &'a Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type NodeIdentifiers = NodeIdentifiers<Ix>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIdentifiers {
            r: 0..self.node_count(),
            ty: PhantomData,
        }
    }
}

impl<N, E, Ty, Ix> NodeCount for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn node_count(&self) -> usize {
        (*self).node_count()
    }
}

impl<N, E, Ty, Ix> EdgeCount for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    #[inline]
    fn edge_count(&self) -> usize {
        self.edge_count()
    }
}

impl<N, E, Ty, Ix> GraphProp for Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type EdgeType = Ty;
}

impl<'a, N, E, Ty, Ix> IntoNodeReferences for &'a Csr<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    type NodeRef = (NodeIndex<Ix>, &'a N);
    type NodeReferences = NodeReferences<'a, N, Ix>;

    fn node_references(self) -> Self::NodeReferences {
        NodeReferences {
            iter: self.node_weights.iter().enumerate(),
            ty: PhantomData,
        }
    }
}

/// Iterator over all nodes of a graph.
#[derive(Debug, Clone)]
pub struct NodeReferences<'a, N: 'a, Ix: IndexType = DefaultIx> {
    iter: iter::Enumerate<slice::Iter<'a, N>>,
    ty: PhantomData<Ix>,
}

impl<'a, N, Ix> Iterator for NodeReferences<'a, N, Ix>
where
    Ix: IndexType,
{
    type Item = (NodeIndex<Ix>, &'a N);

    fn next(&mut self) -> Option<Self::Item> {
        self.iter
            .next()
            .map(|(i, weight)| (NodeIndex::new(Ix::from_usize(i)), weight))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, N, Ix> DoubleEndedIterator for NodeReferences<'a, N, Ix>
where
    Ix: IndexType,
{
    fn next_back(&mut self) -> Option<Self::Item> {
        self.iter
            .next_back()
            .map(|(i, weight)| (NodeIndex::from_usize(i), weight))
    }
}

impl<'a, N, Ix> ExactSizeIterator for NodeReferences<'a, N, Ix> where Ix: IndexType {}

/// The adjacency matrix for **Csr** is a bitmap that's computed by
/// `.adjacency_matrix()`.
impl<'a, N, E, Ty, Ix> GetAdjacencyMatrix for &'a Csr<N, E, Ty, Ix>
where
    Ix: IndexType,
    Ty: EdgeType,
{
    type AdjMatrix = FixedBitSet;

    fn adjacency_matrix(&self) -> FixedBitSet {
        let n = self.node_count();
        let mut matrix = FixedBitSet::with_capacity(n * n);
        for edge in self.edge_references() {
            let i = edge.source().cast() * n + edge.target().cast();
            matrix.put(i);

            let j = edge.source().cast() + n * edge.target().cast();
            matrix.put(j);
        }
        matrix
    }

    fn is_adjacent(&self, matrix: &FixedBitSet, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        let n = self.edge_count();
        let index = n * a.cast() + b.cast();
        matrix.contains(index)
    }
}

/*
 *
Example

[ a 0 b
  c d e
  0 0 f ]

Values: [a, b, c, d, e, f]
Column: [0, 2, 0, 1, 2, 2]
Row   : [0, 2, 5]   <- value index of row start

 * */
