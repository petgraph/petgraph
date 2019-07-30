use fixedbitset::FixedBitSet;
use iter_format::NoPretty;
use std::fmt;
use std::ops::Range;
use visit::{self, EdgeRef, IntoEdgeReferences, IntoNeighbors};

#[doc(no_inline)]
pub use graph::{DefaultIx, IndexType};

/// Adjacency list node index type, a plain integer.
pub type NodeIndex<Ix = DefaultIx> = Ix;

/// Adjacency list edge index type, a pair of integers.
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct EdgeIndex<Ix = DefaultIx>
where
    Ix: IndexType,
{
    from: NodeIndex<Ix>,
    successor_index: usize,
}

iterator_wrap! {
impl (Iterator) for
struct OutgoingEdgeIndices <Ix> where { Ix: IndexType }
item: EdgeIndex<Ix>,
iter: std::iter::Map<std::iter::Zip<Range<usize>, std::iter::Repeat<NodeIndex<Ix>>>, fn((usize, NodeIndex<Ix>)) -> EdgeIndex<Ix>>,
}

/// Weighted sucessor
#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
struct WSuc<E, Ix: IndexType> {
    suc: Ix,
    weight: E,
}

type Row<E, Ix> = Vec<WSuc<E, Ix>>;
type RowIter<'a, E, Ix> = std::slice::Iter<'a, WSuc<E, Ix>>;

iterator_wrap! {
impl (Iterator DoubleEndedIterator ExactSizeIterator) for
struct Neighbors<'a, E, Ix> where { Ix: IndexType }
item: NodeIndex<Ix>,
iter: std::iter::Map<RowIter<'a, E, Ix>, fn(&WSuc<E, Ix>) -> NodeIndex<Ix>>,
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct EdgeReference<'a, E, Ix: IndexType> {
    id: EdgeIndex<Ix>,
    edge: &'a WSuc<E, Ix>,
}

// FIXME: why E: Copy ?
impl<'a, E: Copy, Ix: IndexType> visit::EdgeRef for EdgeReference<'a, E, Ix> {
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
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

pub struct EdgeIndices<'a, E, Ix: IndexType> {
    rows: std::iter::Enumerate<std::slice::Iter<'a, Row<E, Ix>>>,
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
                    from: Ix::new(self.row_index),
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
    struct NodeIndices <Ix> where {}
    item: Ix,
    iter: std::iter::Map<Range<usize>, fn(usize) -> Ix>,
}

/// An adjacency list with labeled edges. Can be interpreted as a directed graph
/// with unweighted nodes.
///
/// This is the most simple adjacency list you can imagine. `Graph`, in contrast,
/// maintains the both the list of successors and predecessors for each node,
/// which is a different trade-off.
pub struct List<E, Ix = DefaultIx>
where
    Ix: IndexType,
{
    suc: Vec<Row<E, Ix>>,
}

impl<E, Ix: IndexType> List<E, Ix> {
    /// Creates a new, empty adjacency list
    pub fn new() -> List<E, Ix> {
        List { suc: Vec::new() }
    }

    /// Creates a new, empty adjacency list tailored for `nodes` nodes.
    pub fn with_capacity(nodes: usize) -> List<E, Ix> {
        List {
            suc: Vec::with_capacity(nodes),
        }
    }

    /// Returns the number of nodes in the list
    ///
    /// Computes in **O(1)** time.
    pub fn node_count(&self) -> usize {
        self.suc.len()
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
        Ix::new(i)
    }

    /// Adds a new node to the list. This allocates a new `Vec` and then should
    /// run in amortized **O(1)** time.
    pub fn add_node_with_capacity(&mut self, successors: usize) -> NodeIndex<Ix> {
        let i = self.suc.len();
        self.suc.push(Vec::with_capacity(successors));
        Ix::new(i)
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
        Ix::new(i)
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
        let row = &mut self.suc[a.index()];
        let rank = row.len();
        row.push(WSuc { suc: b, weight });
        EdgeIndex {
            from: a,
            successor_index: rank,
        }
    }

    /// Updates or adds an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of successors of `a`.
    ///
    /// **Panics** if the source node does not exist.<br>
    pub fn update_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
        let row = &mut self.suc[a.index()];
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

    fn get_edge(&self, e: EdgeIndex<Ix>) -> Option<&WSuc<E, Ix>> {
        self.suc
            .get(e.from.index())
            .and_then(|row| row.get(e.successor_index))
    }

    fn get_edge_mut(&mut self, e: EdgeIndex<Ix>) -> Option<&mut WSuc<E, Ix>> {
        self.suc
            .get_mut(e.from.index())
            .and_then(|row| row.get_mut(e.successor_index))
    }

    /// Accesses the weight of edge `e`
    ///
    /// Computes in **O(1)**
    pub fn edge_weight(&self, e: EdgeIndex<Ix>) -> Option<&E> {
        self.get_edge(e).map(|x| &x.weight)
    }

    /// Accesses the weight of edge `e`
    ///
    /// Computes in **O(1)**
    pub fn edge_weight_mut(&mut self, e: EdgeIndex<Ix>) -> Option<&mut E> {
        self.get_edge_mut(e).map(|x| &mut x.weight)
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
        let iter = (0..(self.suc[a.index()].len()))
            .zip(std::iter::repeat(a))
            .map(proj);
        OutgoingEdgeIndices { iter }
    }

    /// Lookups whether there is an edge from `a` to `b`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of successors of `a`.
    pub fn contains_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        match self.suc.get(a.index()) {
            None => false,
            Some(row) => row.iter().any(|x| x.suc == b),
        }
    }

    /// Lookups whether there is an edge from `a` to `b`.
    ///
    /// Computes in **O(e')** time, where **e'** is the number of successors of `a`.
    pub fn find_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> Option<EdgeIndex<Ix>> {
        self.suc.get(a.index()).and_then(|row| {
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
            iter: (0..self.suc.len()).map(Ix::new),
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

// FIXME: Why Copy ?
impl<'a, E, Ix> fmt::Debug for EdgeReferences<'a, E, Ix>
where
    E: fmt::Debug + Copy,
    Ix: IndexType,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut edge_list = f.debug_list();
        let iter: Self = self.clone();
        for e in iter {
            if std::mem::size_of::<E>() != 0 {
                edge_list.entry(&(
                    NoPretty((e.source().index(), e.target().index())),
                    e.weight(),
                ));
            } else {
                edge_list.entry(&NoPretty((e.source().index(), e.target().index())));
            }
        }
        edge_list.finish()
    }
}
// FIXME: Why Copy ?
impl<E, Ix> fmt::Debug for List<E, Ix>
where
    E: fmt::Debug + Copy,
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

impl<E, Ix> visit::GraphBase for List<E, Ix>
where
    Ix: IndexType,
{
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
}

impl<E, Ix> visit::Visitable for List<E, Ix>
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

impl<'a, E, Ix: IndexType> visit::IntoNodeIdentifiers for &'a List<E, Ix> {
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

impl<'a, Ix: IndexType, E> visit::IntoNodeReferences for &'a List<E, Ix> {
    type NodeRef = NodeIndex<Ix>;
    type NodeReferences = NodeIndices<Ix>;
    fn node_references(self) -> Self::NodeReferences {
        self.node_indices()
    }
}

impl<E, Ix: IndexType> visit::Data for List<E, Ix> {
    type NodeWeight = ();
    type EdgeWeight = E;
}

impl<'a, E, Ix: IndexType> IntoNeighbors for &'a List<E, Ix> {
    type Neighbors = Neighbors<'a, E, Ix>;
    /// Returns an iterator of all nodes with an edge starting from `a`.
    /// Panics if `a` is out of bounds.
    /// Use `edge_indices_from` instead if you do not want to borrow the adjacency list while
    /// iterating.
    fn neighbors(self, a: NodeIndex<Ix>) -> Self::Neighbors {
        let proj: fn(&WSuc<E, Ix>) -> NodeIndex<Ix> = |x| x.suc;
        let iter = self.suc[a.index()].iter().map(proj);
        Neighbors { iter }
    }
}

type SomeIter<'a, E, Ix> = std::iter::Map<
    std::iter::Zip<std::iter::Enumerate<RowIter<'a, E, Ix>>, std::iter::Repeat<Ix>>,
    fn(((usize, &'a WSuc<E, Ix>), Ix)) -> EdgeReference<'a, E, Ix>,
>;

iterator_wrap! {
impl (Iterator) for
#[derive(Clone)]
struct EdgeReferences<'a, E, Ix> where { Ix: IndexType }
item: EdgeReference<'a, E, Ix>,
iter: std::iter::FlatMap<
    std::iter::Enumerate<
        std::slice::Iter<'a, Row<E, Ix>>
    >,
    SomeIter<'a, E, Ix>,
    fn(
        (usize, &'a Vec<WSuc<E, Ix>>)
    ) -> SomeIter<'a, E, Ix>,
>,
}

fn proj1<'a, E, Ix: IndexType>(
    ((successor_index, edge), from): ((usize, &'a WSuc<E, Ix>), Ix),
) -> EdgeReference<'a, E, Ix> {
    let id = EdgeIndex {
        from,
        successor_index,
    };
    EdgeReference { id, edge }
}
fn proj2<'a, E, Ix: IndexType>(
    (row_index, row): (usize, &'a Vec<WSuc<E, Ix>>),
) -> SomeIter<'a, E, Ix> {
    row.iter()
        .enumerate()
        .zip(std::iter::repeat(Ix::new(row_index)))
        .map(proj1 as _)
}
// FIXME: Why E: Copy ?
impl<'a, Ix: IndexType, E: Copy> visit::IntoEdgeReferences for &'a List<E, Ix> {
    type EdgeRef = EdgeReference<'a, E, Ix>;
    type EdgeReferences = EdgeReferences<'a, E, Ix>;
    fn edge_references(self) -> Self::EdgeReferences {
        let iter = self.suc.iter().enumerate().flat_map(proj2 as _);
        EdgeReferences { iter }
    }
}

iterator_wrap! {
impl (Iterator) for
struct OutgoingEdgeReferences<'a, E, Ix> where { Ix: IndexType }
item: EdgeReference<'a, E, Ix>,
iter: SomeIter<'a, E, Ix>,
}
// FIXME: Why E: Copy ?
impl<'a, Ix: IndexType, E: Copy> visit::IntoEdges for &'a List<E, Ix> {
    type Edges = OutgoingEdgeReferences<'a, E, Ix>;
    fn edges(self, a: Self::NodeId) -> Self::Edges {
        let iter = self.suc[a.index()]
            .iter()
            .enumerate()
            .zip(std::iter::repeat(a))
            .map(proj1 as _);
        OutgoingEdgeReferences { iter }
    }
}
