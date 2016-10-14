
//! Compressed Sparse Row (CSR) is a sparse adjacency matrix graph.

use std::marker::PhantomData;
use std::cmp::max;
use std::ops::Range;
use std::iter::{Enumerate, Zip};
use std::slice::Windows;

use visit::{EdgeRef, GraphBase, IntoNeighbors, NodeIndexable, IntoEdges};
use visit::{NodeCompactIndexable, IntoNodeIdentifiers, Visitable};
use visit::{Data, IntoEdgeReferences, NodeCount};

use util::zip;

#[doc(no_inline)]
pub use graph::{IndexType, DefaultIx};

use {
    EdgeType,
    Directed,
    IntoWeightedEdge,
};

/// Csr node index type, a plain integer.
pub type NodeIndex<Ix = DefaultIx> = Ix;
/// Csr edge index type, a plain integer.
pub type EdgeIndex = usize;

const BINARY_SEARCH_CUTOFF: usize = 32;

/// Compressed Sparse Row (CSR) is a sparse adjacency matrix graph.
///
/// Using **O(|E| + |V|)** space.
///
/// Self loops are allowed, no parallel edges.
///
/// Fast iteration of the outgoing edges of a vertex.
///
/// Implementation notes: `N` is not actually used yet, but it is
/// “reserved” as the first type parameter for forward compatibility.
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
    where Ty: EdgeType,
          Ix: IndexType,
{
    /// Create a new `Csr` with `n` nodes.
    pub fn with_nodes(n: usize) -> Self
        where N: Default,
    {
        Csr {
            column: Vec::new(),
            edges: Vec::new(),
            row: vec![0; n + 1],
            node_weights: Vec::new(),
            edge_count: 0,
            ty: PhantomData,
        }
    }
}

/// Csr creation error: edges were not in sorted order.
#[derive(Clone, Debug)]
pub struct EdgesNotSorted {
    first_error: (usize, usize),
}

impl<N, E, Ix> Csr<N, E, Directed, Ix>
    where Ix: IndexType,
{

    /// Create a new `Csr` from a sorted sequence of edges
    ///
    /// Edges **must** be sorted and unique, where the sort order is the default
    /// order for the pair *(u, v)* in Rust (*u* has priority).
    ///
    /// Computes in **O(|E| + |V|)** time.
    pub fn from_sorted_edges<Edge>(edges: &[Edge]) -> Result<Self, EdgesNotSorted>
        where Edge: Clone + IntoWeightedEdge<E, NodeId=NodeIndex<Ix>>,
              N: Default,
    {
        let max_node_id = match edges.iter().map(|edge|
            match edge.clone().into_weighted_edge() {
                (x, y, _) => max(x.index(), y.index())
            }).max() {
            None => return Ok(Self::with_nodes(0)),
            Some(x) => x,
        };
        let mut self_ = Self::with_nodes(max_node_id + 1);
        let mut iter = edges.iter().cloned().peekable();
        {
            let mut rows = self_.row.iter_mut();

            let mut node = 0;
            let mut rstart = 0;
            let mut last_target;
            'outer: for r in &mut rows {
                *r = rstart;
                last_target = None;
                'inner: loop {
                    if let Some(edge) = iter.peek() {
                        let (n, m, weight) = edge.clone().into_weighted_edge();
                        // check that the edges are in increasing sequence
                        if node > n.index() {
                            return Err(EdgesNotSorted {
                                first_error: (n.index(), m.index()),
                            });
                        }
                        /*
                        debug_assert!(node <= n.index(),
                                      concat!("edges are not sorted, ",
                                              "failed assertion source {:?} <= {:?} ",
                                              "for edge {:?}"),
                                      node, n, (n, m));
                                      */
                        if n.index() != node {
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
                                first_error: (n.index(), m.index()),
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
                node += 1;
            }
            for r in rows {
                *r = rstart;
            }
        }

        Ok(self_)
    }
}

impl<N, E, Ty, Ix> Csr<N, E, Ty, Ix>
    where Ty: EdgeType,
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

    /// Return `true` if the edge was added
    ///
    /// If you add all edges in row-major order, the time complexity
    /// is **O(|V|·|E|)** for the whole operation.
    ///
    /// **Panics** if `a` or `b` are out of bounds.
    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> bool
        where E: Clone,
    {
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
        assert!(a.index() < self.node_count() && b.index() < self.node_count());
        // a x b is at (a, b) in the matrix

        // find current range of edges from a
        let pos = match self.find_edge_pos(a, b) {
            Ok(_) => return false, /* already exists */
            Err(i) => i,
        };
        self.column.insert(pos, b);
        self.edges.insert(pos, weight);
        // update row vector
        for r in &mut self.row[a.index() + 1..] {
            *r += 1;
        }
        true
    }

    fn find_edge_pos(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Result<usize, usize> {
        let (index, neighbors) = self.neighbors_of(a);
        if neighbors.len() < BINARY_SEARCH_CUTOFF {
            for (i, elt) in neighbors.iter().enumerate() {
                if b == *elt {
                    return Ok(i + index);
                } else if *elt > b {
                    return Err(i + index);
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
    pub fn contains_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        self.find_edge_pos(a, b).is_ok()
    }

    /// Computes in **O(1)** time.
    pub fn out_degree(&self, node: NodeIndex<Ix>) -> usize {
        self.neighbors_slice(node).len()
    }

    fn neighbors_of(&self, a: NodeIndex<Ix>) -> (usize, &[Ix]) {
        let index = self.row[a.index()];
        let end = self.row.get(a.index() + 1).cloned().unwrap_or(self.column.len());
        (index, &self.column[index..end])
    }

    /// Computes in **O(1)** time.
    pub fn neighbors_slice(&self, node: NodeIndex<Ix>) -> &[NodeIndex<Ix>] {
        self.neighbors_of(node).1
    }

    /// Computes in **O(1)** time.
    pub fn edges_slice(&self, a: NodeIndex<Ix>) -> &[E] {
        let index = self.row[a.index()];
        let end = self.row.get(a.index() + 1).cloned().unwrap_or(self.column.len());
        &self.edges[index..end]
    }

    /// Return an iterator of all edges of `a`.
    ///
    /// - `Directed`: Outgoing edges from `a`.
    /// - `Undirected`: All edges connected to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `EdgeReference<E, Ty, Ix>`.
    pub fn edges(&self, a: NodeIndex<Ix>) -> Edges<E, Ty, Ix> {
        let index = self.row[a.index()];
        let end = self.row.get(a.index() + 1).cloned().unwrap_or(self.column.len());
        Edges {
            index: index,
            source: a,
            iter: zip(&self.column[index..end], &self.edges[index..end]),
            ty: self.ty,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Edges<'a, E: 'a, Ty = Directed, Ix: 'a = DefaultIx> {
    index: usize,
    source: NodeIndex<Ix>,
    iter: Zip<SliceIter<'a, NodeIndex<Ix>>, SliceIter<'a, E>>,
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

impl<'a, E, Ty, Ix: Copy> Copy for EdgeReference<'a, E, Ty, Ix> { }

impl<'a, Ty, E, Ix> EdgeReference<'a, E, Ty, Ix>
    where Ty: EdgeType,
{
    /// Access the edge’s weight.
    ///
    /// **NOTE** that this method offers a longer lifetime
    /// than the trait (unfortunately they don't match yet).
    pub fn weight(&self) -> &'a E { self.weight }
}

impl<'a, E, Ty, Ix> EdgeRef for EdgeReference<'a, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex;
    type Weight = E;

    fn source(&self) -> Self::NodeId { self.source }
    fn target(&self) -> Self::NodeId { self.target }
    fn weight(&self) -> &E { self.weight }
    fn id(&self) -> Self::EdgeId { self.index }
}

impl<'a, E, Ty, Ix> Iterator for Edges<'a, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Item = EdgeReference<'a, E, Ty, Ix>;
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().map(move |(&j, w)| {
            let index = self.index;
            self.index += 1;
            EdgeReference {
                index: index,
                source: self.source,
                target: j,
                weight: w,
                ty: PhantomData,
            }
        })
    }
}

impl<N, E, Ty, Ix> Data for Csr<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<'a, N, E, Ty, Ix> IntoEdgeReferences for &'a Csr<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type EdgeRef = EdgeReference<'a, E, Ty, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ty, Ix>;
    fn edge_references(self) -> Self::EdgeReferences {
        EdgeReferences {
            index: 0,
            source_index: Ix::new(0),
            edge_ranges: self.row.windows(2).enumerate(),
            column: &self.column,
            edges: &self.edges,
            iter: zip(&[], &[]),
            ty: self.ty,
        }
    }
}

pub struct EdgeReferences<'a, E: 'a, Ty, Ix: 'a> {
    source_index: NodeIndex<Ix>,
    index: usize,
    edge_ranges: Enumerate<Windows<'a, usize>>,
    column: &'a [NodeIndex<Ix>],
    edges: &'a [E],
    iter: Zip<SliceIter<'a, NodeIndex<Ix>>, SliceIter<'a, E>>,
    ty: PhantomData<Ty>,
}

impl<'a, E, Ty, Ix> Iterator for EdgeReferences<'a, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Item = EdgeReference<'a, E, Ty, Ix>;
    fn next(&mut self) -> Option<Self::Item> {
        loop {
            if let Some((&j, w)) = self.iter.next() {
                let index = self.index;
                self.index += 1;
                return Some(EdgeReference {
                    index: index,
                    source: self.source_index,
                    target: j,
                    weight: w,
                    ty: PhantomData,
                })
            }
            if let Some((i, w)) = self.edge_ranges.next() {
                let a = w[0];
                let b = w[1];
                self.iter = zip(&self.column[a..b], &self.edges[a..b]);
                self.source_index = Ix::new(i);
            } else {
                return None;
            }
        }
    }
}

impl<'a, N, E, Ty, Ix> IntoEdges for &'a Csr<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Edges = Edges<'a, E, Ty, Ix>;
    fn edges(self, a: Self::NodeId) -> Self::Edges {
        self.edges(a)
    }
}

impl<N, E, Ty, Ix> GraphBase for Csr<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex; // index into edges vector
}

use fixedbitset::FixedBitSet;

impl<N, E, Ty, Ix> Visitable for Csr<N, E, Ty, Ix>
    where Ty: EdgeType,
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

use std::slice::Iter as SliceIter;

#[derive(Clone, Debug)]
pub struct Neighbors<'a, Ix: 'a = DefaultIx> {
    iter: SliceIter<'a, NodeIndex<Ix>>,
}

impl<'a, Ix> Iterator for Neighbors<'a, Ix>
    where Ix: IndexType,
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
    where Ty: EdgeType,
          Ix: IndexType,
{
    type Neighbors = Neighbors<'a, Ix>;

    /// Return an iterator of all neighbors of `a`.
    ///
    /// - `Directed`: Targets of outgoing edges from `a`.
    /// - `Undirected`: Opposing endpoints of all edges connected to `a`.
    ///
    /// Produces an empty iterator if the node doesn't exist.<br>
    /// Iterator element type is `NodeIndex<Ix>`.
    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors {
        Neighbors {
            iter: self.neighbors_slice(n).iter(),
        }
    }
}

impl<N, E, Ty, Ix> NodeIndexable for Csr<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
{
    fn node_bound(&self) -> usize { self.node_count() }
    fn to_index(&self, a: Self::NodeId) -> usize { a.index() }
    fn from_index(&self, ix: usize) -> Self::NodeId { Ix::new(ix) }
}
impl<N, E, Ty> NodeCompactIndexable for Csr<N, E, Ty>
    where Ty: EdgeType { }

pub struct NodeIdentifiers<Ix = DefaultIx> {
    r: Range<usize>,
    ty: PhantomData<Ix>,
}

impl<Ix> Iterator for NodeIdentifiers<Ix>
    where Ix: IndexType,
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        self.r.next().map(Ix::new)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.r.size_hint()
    }
}

impl<'a, N, E, Ty> IntoNodeIdentifiers for &'a Csr<N, E, Ty>
    where Ty: EdgeType,
{
    type NodeIdentifiers = NodeIdentifiers;
    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIdentifiers {
            r: 0..self.node_count(),
            ty: PhantomData,
        }
    }
}

impl<N, E, Ty> NodeCount for Csr<N, E, Ty>
    where Ty: EdgeType,
{
    fn node_count(&self) -> usize {
        (*self).node_count()
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

#[cfg(test)]
mod tests {
    use super::Csr;
    use Undirected;
    use visit::Dfs;
    use visit::VisitMap;
    use algo::tarjan_scc;
    use algo::bellman_ford;

    #[test]
    fn csr1() {
        let mut m: Csr = Csr::with_nodes(3);
        m.add_edge(0, 0, ());
        m.add_edge(1, 2, ());
        m.add_edge(2, 2, ());
        m.add_edge(0, 2, ());
        m.add_edge(1, 0, ());
        m.add_edge(1, 1, ());
        println!("{:?}", m);
        assert_eq!(&m.column, &[0, 2, 0, 1, 2, 2]);
        assert_eq!(&m.row, &[0, 2, 5, 6]);

        let added = m.add_edge(1, 2, ());
        assert!(!added);
        assert_eq!(&m.column, &[0, 2, 0, 1, 2, 2]);
        assert_eq!(&m.row, &[0, 2, 5, 6]);

        assert_eq!(m.neighbors_slice(1), &[0, 1, 2]);
        assert_eq!(m.node_count(), 3);
        assert_eq!(m.edge_count(), 6);
    }

    #[test]
    fn csr_undirected() {
    /*
        [ 1 . 1
          . . 1
          1 1 1 ]
     */

        let mut m: Csr<(), (), Undirected> = Csr::with_nodes(3);
        m.add_edge(0, 0, ());
        m.add_edge(0, 2, ());
        m.add_edge(1, 2, ());
        m.add_edge(2, 2, ());
        println!("{:?}", m);
        assert_eq!(&m.column, &[0, 2, 2, 0, 1, 2]);
        assert_eq!(&m.row, &[0, 2, 3, 6]);
        assert_eq!(m.node_count(), 3);
        assert_eq!(m.edge_count(), 4);
    }

    #[should_panic]
    #[test]
    fn csr_from_error_1() {
        // not sorted in source
        let m: Csr = Csr::from_sorted_edges(&[
            (0, 1),
            (1, 0),
            (0, 2),
        ]).unwrap();
        println!("{:?}", m);
    }

    #[should_panic]
    #[test]
    fn csr_from_error_2() {
        // not sorted in target
        let m: Csr = Csr::from_sorted_edges(&[
            (0, 1),
            (1, 0),
            (1, 2),
            (1, 1),
        ]).unwrap();
        println!("{:?}", m);
    }

    #[test]
    fn csr_from() {
        let m: Csr = Csr::from_sorted_edges(&[
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (2, 2),
            (2, 4),
        ]).unwrap();
        println!("{:?}", m);
        assert_eq!(m.neighbors_slice(0), &[1, 2]);
        assert_eq!(m.neighbors_slice(1), &[0, 1]);
        assert_eq!(m.neighbors_slice(2), &[2, 4]);
        assert_eq!(m.node_count(), 5);
        assert_eq!(m.edge_count(), 6);
    }

    #[test]
    fn csr_dfs() {
        let mut m: Csr = Csr::from_sorted_edges(&[
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (1, 3),
            (2, 2),

            // disconnected subgraph
            (4, 4),
            (4, 5),
        ]).unwrap();
        println!("{:?}", m);
        let mut dfs = Dfs::new(&m, 0);
        while let Some(_) = dfs.next(&m) {
        }
        for i in 0..m.node_count() - 2 {
            assert!(dfs.discovered.is_visited(&i), "visited {}", i)
        }
        assert!(!dfs.discovered[4]);
        assert!(!dfs.discovered[5]);

        m.add_edge(1, 4, ());
        println!("{:?}", m);

        dfs.reset(&m);
        dfs.move_to(0);
        while let Some(_) = dfs.next(&m) {
        }

        for i in 0..m.node_count() {
            assert!(dfs.discovered[i], "visited {}", i)
        }
    }

    #[test]
    fn csr_tarjan() {
        let m: Csr = Csr::from_sorted_edges(&[
            (0, 1),
            (0, 2),
            (1, 0),
            (1, 1),
            (1, 3),
            (2, 2),
            (2, 4),
            (4, 4),
            (4, 5),
            (5, 2),
        ]).unwrap();
        println!("{:?}", m);
        println!("{:?}", tarjan_scc(&m));
    }

    #[test]
    fn test_bellman_ford() {
        let m: Csr<(), _> = Csr::from_sorted_edges(&[
            (0, 1, 0.5),
            (0, 2, 2.),
            (1, 0, 1.),
            (1, 1, 1.),
            (1, 2, 1.),
            (1, 3, 1.),
            (2, 3, 3.),

            (4, 5, 1.),
            (5, 7, 2.),
            (6, 7, 1.),
            (7, 8, 3.),
        ]).unwrap();
        println!("{:?}", m);
        println!("{:?}", bellman_ford(&m, 0));
    }

    #[test]
    fn test_edge_references() {
        use visit::EdgeRef;
        use visit::IntoEdgeReferences;
        let m: Csr<(), _> = Csr::from_sorted_edges(&[
            (0, 1, 0.5),
            (0, 2, 2.),
            (1, 0, 1.),
            (1, 1, 1.),
            (1, 2, 1.),
            (1, 3, 1.),
            (2, 3, 3.),

            (4, 5, 1.),
            (5, 7, 2.),
            (6, 7, 1.),
            (7, 8, 3.),
        ]).unwrap();
        let mut copy = Vec::new();
        for e in m.edge_references() {
            copy.push((e.source(), e.target(), *e.weight()));
            println!("{:?}", e);
        }
        let m2: Csr<(), _> = Csr::from_sorted_edges(&copy).unwrap();
        assert_eq!(&m.row, &m2.row);
        assert_eq!(&m.column, &m2.column);
        assert_eq!(&m.edges, &m2.edges);
    }
}