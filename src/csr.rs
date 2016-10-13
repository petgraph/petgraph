
//! Compressed Sparse Row (CSR) is a sparse adjacency matrix graph.

use std::marker::PhantomData;
use std::cmp::max;
use std::ops::Range;
use std::iter::Zip;

use visit::{EdgeRef, GraphBase, IntoNeighbors, NodeIndexable, IntoEdges};
use visit::{NodeCompactIndexable, IntoNodeIdentifiers, Visitable};
use visit::GraphEdgeRef;
use data::Data;

use util::zip;

#[doc(no_inline)]
pub use graph::IndexType;

use {
    EdgeType,
    Directed,
    IntoWeightedEdge,
};

pub type NodeIndex = usize;
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
pub struct Csr<N = (), E = (), Ty = Directed> {
    /// Column of next edge
    column: Vec<NodeIndex>,
    /// weight of each edge; lock step with column
    edges: Vec<E>,
    /// Index of start of row
    row: Vec<NodeIndex>,
    node_weights: Vec<N>,
    edge_count: usize,
    ty: PhantomData<Ty>,
}

impl<N: Clone, E: Clone, Ty> Clone for Csr<N, E, Ty> {
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

impl<N, E, Ty> Csr<N, E, Ty>
    where Ty: EdgeType
{
    /// Create a new `Csr` with `n` nodes.
    pub fn with_nodes(n: usize) -> Self
        where N: Default,
    {
        Csr {
            column: Vec::new(),
            edges: Vec::new(),
            row: vec![0; n],
            node_weights: Vec::new(),
            edge_count: 0,
            ty: PhantomData,
        }
    }

    /// Create a new `Csr` from a sorted sequence of edges
    ///
    /// Edges **must** be sorted and unique, where the sort order is the default
    /// order for the pair *(u, v)* in Rust (*u* has priority).
    ///
    /// Computes in **O(|E| + |V|)** time.
    pub fn from_sorted_edges<Edge>(edges: &[Edge]) -> Self
        where Edge: Clone + IntoWeightedEdge<E, NodeId=NodeIndex>,
              N: Default,
    {
        let nodes = match edges.iter().map(|edge|
            match edge.clone().into_weighted_edge() {
                (x, y, _) => max(x, y)
            }).max() {
            None => return Self::with_nodes(0),
            Some(x) => x,
        };
        let mut self_ = Self::with_nodes(nodes + 1);
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
                        debug_assert!(node <= n,
                                      concat!("edges are not sorted, ",
                                              "failed assertion source {:?} <= {:?} ",
                                              "for edge {:?}"),
                                      node, n, (n, m));
                        if n != node {
                            break 'inner;
                        }
                        // check that the edges are in increasing sequence
                        debug_assert!(last_target.map_or(true, |x| m > x),
                                      "edges are not sorted, failed assertion {:?} < {:?}",
                                      last_target, m);
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

        self_
    }

    pub fn node_count(&self) -> usize {
        self.row.len()
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
    pub fn add_edge(&mut self, a: NodeIndex, b: NodeIndex, weight: E) -> bool
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
    fn add_edge_(&mut self, a: NodeIndex, b: NodeIndex, weight: E) -> bool {
        assert!(a < self.node_count() && b < self.node_count());
        // a x b is at (a, b) in the matrix

        // find current range of edges from a
        let pos = match self.find_edge_pos(a, b) {
            Ok(_) => return false, /* already exists */
            Err(i) => i,
        };
        self.column.insert(pos, b);
        self.edges.insert(pos, weight);
        // update row vector
        for r in &mut self.row[a + 1..] {
            *r += 1;
        }
        true
    }

    fn find_edge_pos(&self, a: NodeIndex, b: NodeIndex) -> Result<usize, usize> {
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
    pub fn contains_edge(&self, a: NodeIndex, b: NodeIndex) -> bool {
        self.find_edge_pos(a, b).is_ok()
    }

    /// Computes in **O(1)** time.
    pub fn out_degree(&self, node: NodeIndex) -> usize {
        self.neighbors_slice(node).len()
    }

    fn neighbors_of(&self, node: NodeIndex) -> (usize, &[NodeIndex]) {
        let index = self.row[node];
        let end = self.row.get(node + 1).cloned().unwrap_or(self.column.len());
        (index, &self.column[index..end])
    }

    /// Computes in **O(1)** time.
    pub fn neighbors_slice(&self, node: NodeIndex) -> &[NodeIndex] {
        self.neighbors_of(node).1
    }

    /// Computes in **O(1)** time.
    pub fn edges_slice(&self, node: NodeIndex) -> &[E] {
        let index = self.row[node];
        let end = self.row.get(node + 1).cloned().unwrap_or(self.column.len());
        &self.edges[index..end]
    }

    pub fn edges(&self, node: NodeIndex) -> Edges<E, Ty> {
        let index = self.row[node];
        let end = self.row.get(node + 1).cloned().unwrap_or(self.column.len());
        Edges {
            index: index,
            source: node,
            iter: zip(&self.column[index..end], &self.edges[index..end]),
            ty: PhantomData,
        }
    }
}

#[derive(Clone, Debug)]
pub struct Edges<'a, E: 'a, Ty = Directed> {
    index: EdgeIndex,
    source: NodeIndex,
    iter: Zip<SliceIter<'a, NodeIndex>, SliceIter<'a, E>>,
    ty: PhantomData<Ty>,
}

#[derive(Debug)]
pub struct EdgeReference<'a, E: 'a, Ty> {
    index: EdgeIndex,
    source: NodeIndex,
    target: NodeIndex,
    weight: &'a E,
    ty: PhantomData<Ty>,
}

impl<'a, E, Ty> Clone for EdgeReference<'a, E, Ty> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, E, Ty> Copy for EdgeReference<'a, E, Ty> { }

impl<'a, Ty, E> EdgeReference<'a, E, Ty>
    where Ty: EdgeType,
{
    /// Access the edge’s weight.
    ///
    /// **NOTE** that this method offers a longer lifetime
    /// than the trait (unfortunately they don't match yet).
    pub fn weight(&self) -> &'a E { self.weight }
}

impl<'a, E, Ty> EdgeRef for EdgeReference<'a, E, Ty>
    where Ty: EdgeType,
{
    type NodeId = NodeIndex;
    type EdgeId = EdgeIndex;
    type Weight = E;

    fn source(&self) -> Self::NodeId { self.source }
    fn target(&self) -> Self::NodeId { self.target }
    fn weight(&self) -> &E { self.weight }
    fn id(&self) -> Self::EdgeId { self.index }
}

impl<'a, E, Ty> Iterator for Edges<'a, E, Ty>
    where Ty: EdgeType,
{
    type Item = EdgeReference<'a, E, Ty>;
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

impl<N, E, Ty> Data for Csr<N, E, Ty>
    where Ty: EdgeType,
{
    type NodeWeight = N;
    type EdgeWeight = E;
}

impl<'a, N, E, Ty> GraphEdgeRef for &'a Csr<N, E, Ty>
    where Ty: EdgeType,
{
    type EdgeRef = EdgeReference<'a, E, Ty>;
}

impl<'a, N, E, Ty> IntoEdges for &'a Csr<N, E, Ty>
    where Ty: EdgeType,
{
    type Edges = Edges<'a, E, Ty>;
    fn edges(self, a: Self::NodeId) -> Self::Edges {
        self.edges(a)
    }
}

impl<N, E, Ty> GraphBase for Csr<N, E, Ty>
    where Ty: EdgeType,
{
    type NodeId = NodeIndex;
    type EdgeId = NodeIndex; // index into edges?
}

use fixedbitset::FixedBitSet;

impl<N, E, Ty> Visitable for Csr<N, E, Ty>
    where Ty: EdgeType
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
pub struct Neighbors<'a> {
    iter: SliceIter<'a, NodeIndex>,
}

impl<'a> Iterator for Neighbors<'a> {
    type Item = NodeIndex;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.iter.next().cloned()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}

impl<'a, N, E, Ty> IntoNeighbors for &'a Csr<N, E, Ty>
    where Ty: EdgeType,
{
    type Neighbors = Neighbors<'a>;
    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors {
        Neighbors {
            iter: self.neighbors_slice(n).iter(),
        }
    }
}

impl<N, E, Ty> NodeIndexable for Csr<N, E, Ty>
    where Ty: EdgeType,
{
    fn node_bound(&self) -> usize { self.node_count() }
    fn to_index(a: Self::NodeId) -> usize { a }
    fn from_index(ix: usize) -> Self::NodeId { ix }
}
impl<N, E, Ty> NodeCompactIndexable for Csr<N, E, Ty>
    where Ty: EdgeType { }

pub struct NodeIdentifiers<Ix = NodeIndex> {
    r: Range<usize>,
    ty: PhantomData<Ix>,
}

impl Iterator for NodeIdentifiers {
    type Item = NodeIndex;

    #[inline]
    fn next(&mut self) -> Option<Self::Item> {
        self.r.next()
    }

    #[inline]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.r.size_hint()
    }
}

impl<'a, N, E, Ty> IntoNodeIdentifiers for &'a Csr<N, E, Ty>
    where Ty: EdgeType,
{
    type NodeIdentifiers = NodeIdentifiers;
    fn node_count(&self) -> usize {
        (*self).node_count()
    }
    fn node_identifiers(self) -> Self::NodeIdentifiers {
        NodeIdentifiers {
            r: 0..self.node_count(),
            ty: PhantomData,
        }
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
        assert_eq!(&m.row, &[0, 2, 5]);

        let added = m.add_edge(1, 2, ());
        assert!(!added);
        assert_eq!(&m.column, &[0, 2, 0, 1, 2, 2]);
        assert_eq!(&m.row, &[0, 2, 5]);

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
        assert_eq!(&m.row, &[0, 2, 3]);
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
        ]);
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
        ]);
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
        ]);
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
        ]);
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
        ]);
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
        ]);
        println!("{:?}", m);
        println!("{:?}", bellman_ford(&m, 0));
    }
}
