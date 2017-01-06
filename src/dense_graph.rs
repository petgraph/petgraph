use std::collections::hash_set;
use std::marker::PhantomData;
use std::ops::{Index, IndexMut};
use std::cmp;

use dense_mapping::{DenseMapping, IdIterator};
use fixedbitset::FixedBitSet;

use {
    Directed,
    Direction,
    EdgeType,
    Incoming,
    IntoWeightedEdge,
    //Outgoing,
    Undirected,
};

pub use graph::{
    DefaultIx,
    EdgeIndex,
    IndexType,
    NodeIndex,
};

use visit::{
    GetAdjacencyMatrix,
    GraphBase,
    IntoNeighbors,
    IntoNeighborsDirected,
    IntoNodeIdentifiers,
    NodeCount,
    Visitable
};

pub use graph::{node_index, edge_index};

type OptionVec<T> = Vec<Option<T>>;

const INITIAL_RESIZE: usize = 10;

fn ensure_len<T>(vec: &mut OptionVec<T>, min_len: usize) where T: Clone {
    if vec.len() <= min_len {
        let next_power_of_2_len = (min_len + 2) & !1;
        vec.resize(cmp::max(next_power_of_2_len, INITIAL_RESIZE), None);
    }
}

type Matrix = OptionVec<OptionVec<EdgeList>>;

type EdgeList = hash_set::HashSet<usize>;
type EdgeIter<'a> = hash_set::Iter<'a, usize>;

pub struct DenseGraph<N, E, Ty = Directed, Ix = DefaultIx> {
    matrix: Matrix,
    nodes: DenseMapping<N>,
    edges: DenseMapping<E>,
    ty: PhantomData<Ty>,
    ix: PhantomData<Ix>,
}

pub type DenseDiGraph<N, E, Ix = DefaultIx> = DenseGraph<N, E, Directed, Ix>;
pub type DenseUnGraph<N, E, Ix = DefaultIx> = DenseGraph<N, E, Undirected, Ix>;

impl<N, E, Ty, Ix> DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone
{
    /// Create a new `DenseGraph` with estimated capacity.
    pub fn with_capacity(nodes: usize, edges: usize) -> Self {
        DenseGraph {
            matrix: Matrix::with_capacity(nodes),
            nodes: DenseMapping::with_capacity(nodes),
            edges: DenseMapping::with_capacity(edges),
            ty: PhantomData,
            ix: PhantomData,
        }
    }

    /// Remove all nodes and edges
    pub fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
        self.matrix.clear();
    }

    /// Return the number of nodes (vertices) in the graph.
    ///
    /// Computes in **O(1)** time.
    pub fn node_count(&self) -> usize {
        self.nodes.size()
    }

    /// Return the number of edges in the graph.
    ///
    /// Computes in **O(1)** time.
    pub fn edge_count(&self) -> usize {
        self.edges.size()
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
    pub fn add_node(&mut self, weight: N) -> NodeIndex<Ix> {
        let (id, _) = self.nodes.add(weight);

        // Resize the matrix if necessary
        ensure_len(&mut self.matrix, id);

        NodeIndex::new(id)
    }

    /// Remove `a` from the graph if it exists, and return its weight.
    /// If it doesn't exist in the graph, return `None`.
    ///
    /// The node index `a` is invalidated, but none other.
    /// Edge indices are invalidated as they would be following the removal of
    /// each edge with an endpoint in `a`.
    ///
    /// Computes in **O(V)** time, due to the removal of edges with other nodes.
    pub fn remove_node(&mut self, ix: NodeIndex<Ix>) -> Option<N> {
        let removed_node = self.nodes.remove(ix.index());

        for node_neighbors in self.matrix.iter_mut().filter_map(|x| x.as_mut()) {
            if ix.index() < node_neighbors.len() {
                node_neighbors[ix.index()] = None;
            }
        }

        self.matrix[ix.index()] = None;

        removed_node
    }

    pub fn contains_node(&self, ix: NodeIndex<Ix>) -> bool {
        self.nodes.exists(ix.index())
    }

    /// Add an edge from `a` to `b` to the graph, with its associated
    /// data `weight`.
    ///
    /// Return the index of the new edge.
    ///
    /// Computes in **O(1)** time.
    ///
    /// **Panics** if any of the nodes don't exist.<br>
    /// **Panics** if the Graph is at the maximum number of edges for its index
    /// type.
    ///
    /// **Note:** `DenseGraph` allows adding parallel (“duplicate”) edges. If you want to avoid
    /// this, use [`.update_edge(a, b, weight)`](#method.update_edge) instead.
    pub fn add_edge(&mut self, a: NodeIndex<Ix>, b: NodeIndex<Ix>, weight: E) -> EdgeIndex<Ix> {
        let (id, _) = self.edges.add(weight);

        {
            let is_directed = self.is_directed();

            let mut insert_edge = |source: usize, target: usize| {
                let ref mut neighbors = self.matrix[source];
                if neighbors.is_none() {
                    *neighbors = Some(Vec::new());
                }

                // Make sure node "source" has an entry for "target"
                let mut neighbors = neighbors.as_mut().unwrap();
                ensure_len(&mut neighbors, target);

                let ref mut edges = neighbors[target];
                if edges.is_none() {
                    *edges = Some(EdgeList::new());
                }
                edges.as_mut().unwrap().insert(id);
            };

            insert_edge(a.index(), b.index());

            if !is_directed {
                insert_edge(b.index(), a.index());
            }
        }

        EdgeIndex::new(id)
    }

    /// Return an iterator of all nodes with an edge starting from `a`.
    ///
    /// **Panics** if the node doesn't exist. Iterator element type is `NodeIndex<Ix>`.
    pub fn neighbors(&self, ix: NodeIndex<Ix>) -> Neighbors<Ix> {
        Neighbors::new(MatrixNeighbors::on_columns(&self.matrix, ix))
    }

    pub fn edges(&self, ix: NodeIndex<Ix>) -> Edges<Ix> {
        Edges::new(MatrixNeighbors::on_columns(&self.matrix, ix))
    }

    /// Return an iterator of all neighbors that have an edge between them and
    /// `a`, in the specified direction.
    ///
    /// - `Outgoing`: All edges from `a`.
    /// - `Incoming`: All edges to `a`.
    ///
    /// **Panics** if the node doesn't exist. Iterator element type is `NodeIndex<Ix>`.
    pub fn neighbors_directed(&self, ix: NodeIndex<Ix>, dir: Direction) -> Neighbors<Ix> {
        if dir == Incoming {
            Neighbors::new(MatrixNeighbors::on_rows(&self.matrix, ix))
        } else {
            self.neighbors(ix)
        }
    }

    pub fn edges_directed(&self, ix: NodeIndex<Ix>, dir: Direction) -> Edges<Ix> {
        if dir == Incoming {
            Edges::new(MatrixNeighbors::on_rows(&self.matrix, ix))
        } else {
            self.edges(ix)
        }
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
              <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex<Ix>>,
              N: Default,
    {
        let iter = iterable.into_iter();

        for elt in iter {
            let (source, target, weight) = elt.into_weighted_edge();
            let (source, target) = (source.into(), target.into());
            let nx = cmp::max(source, target);
            while nx.index() >= self.node_count() {
                self.add_node(N::default());
            }
            self.add_edge(source, target, weight);
        }
    }

    /// Create a new `DenseGraph` from an iterable of edges.
    ///
    /// Node weights `N` are set to default values.
    /// Edge weights `E` may either be specified in the list,
    /// or they are filled with default values.
    ///
    /// Nodes are inserted automatically to match the edges.
    pub fn from_edges<I>(iterable: I) -> Self
        where I: IntoIterator,
              I::Item: IntoWeightedEdge<E>,
              <I::Item as IntoWeightedEdge<E>>::NodeId: Into<NodeIndex<Ix>>,
              N: Default,
    {
        let mut g = Self::with_capacity(0, 0);
        g.extend_with_edges(iterable);
        g
    }

    /// Return `true` if the edge connecting `a` with `b` is contained in the graph.
    pub fn contains_edge(&self, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        self.matrix[a.index()]
            .as_ref()
            .map(|ns| ns[b.index()].is_some())
            .unwrap_or(false)
    }

    pub fn node_indices(&self) -> NodeIndices<Ix> {
        NodeIndices::new(self.nodes.iter_ids())
    }
}

pub struct NodeIndices<'a, Ix>
    where Ix: IndexType,
{
    iter: IdIterator<'a>,
    ix: PhantomData<Ix>,
}

impl<'a, Ix> NodeIndices<'a, Ix>
    where Ix: IndexType,
{
    fn new(iter: IdIterator<'a>) -> NodeIndices<'a, Ix> {
        NodeIndices {
            iter: iter,
            ix: PhantomData,
        }
    }
}

impl<'a, Ix> Iterator for NodeIndices<'a, Ix>
    where Ix: IndexType
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<NodeIndex<Ix>> {
        self.iter.next().map(node_index)
    }
}

impl<N, E, Ix> DenseGraph<N, E, Directed, Ix>
    where N: Clone,
          E: Clone,
          Ix: IndexType,
{
    /// Create a new `DenseGraph` with directed edges.
    ///
    /// This is a convenience method. Use `DenseGraph::with_capacity` or `DenseGraph::default` for
    /// a constructor that is generic in all the type parameters of `DenseGraph`.
    pub fn new() -> Self {
        DenseGraph::with_capacity(0, 0)
    }
}

impl<N, E, Ix> DenseGraph<N, E, Undirected, Ix>
    where N: Clone,
          E: Clone,
          Ix: IndexType
{
    /// Create a new `DenseGraph` with undirected edges.
    ///
    /// This is a convenience method. Use `DenseGraph::with_capacity` or `DenseGraph::default` for
    /// a constructor that is generic in all the type parameters of `DenseGraph`.
    pub fn new_undirected() -> Self {
        DenseGraph::with_capacity(0, 0)
    }

    pub fn neighbors_undirected(&self, ix: NodeIndex<Ix>) -> Neighbors<Ix> {
        self.neighbors(ix)
    }
}

/// Create a new empty `DenseGraph`.
impl<N, E, Ty, Ix> Default for DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone
{
    fn default() -> Self {
        Self::with_capacity(0, 0)
    }
}

/// Index the `DenseGraph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> Index<NodeIndex<Ix>> for DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone
{
    type Output = N;
    fn index(&self, index: NodeIndex<Ix>) -> &N {
        &self.nodes[index.index()]
    }
}

/// Index the `DenseGraph` by `NodeIndex` to access node weights.
///
/// **Panics** if the node doesn't exist.
impl<N, E, Ty, Ix> IndexMut<NodeIndex<Ix>> for DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone
{
    fn index_mut(&mut self, index: NodeIndex<Ix>) -> &mut N {
        &mut self.nodes[index.index()]
    }
}

/// Index the `DenseGraph` by `EdgeIndex` to access edge weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> Index<EdgeIndex<Ix>> for DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone
{
    type Output = E;
    fn index(&self, index: EdgeIndex<Ix>) -> &E {
        &self.edges[index.index()]
    }
}

/// Index the `DenseGraph` by `EdgeIndex` to access node weights.
///
/// **Panics** if the edge doesn't exist.
impl<N, E, Ty, Ix> IndexMut<EdgeIndex<Ix>> for DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone
{
    fn index_mut(&mut self, index: EdgeIndex<Ix>) -> &mut E {
        &mut self.edges[index.index()]
    }
}

/// The resulting cloned graph has the same graph indices as `self`.
impl<N, E, Ty, Ix: IndexType> Clone for DenseGraph<N, E, Ty, Ix>
    where N: Clone,
          E: Clone
{
    fn clone(&self) -> Self {
        DenseGraph {
            nodes: self.nodes.clone(),
            edges: self.edges.clone(),
            matrix: self.matrix.clone(),
            ty: self.ty,
            ix: self.ix,
        }
    }

    fn clone_from(&mut self, rhs: &Self) {
        self.nodes.clone_from(&rhs.nodes);
        self.edges.clone_from(&rhs.edges);
        self.matrix.clone_from(&rhs.matrix);
        self.ty = rhs.ty;
    }
}

struct MatrixNeighbors<'a, Ix>
    where Ix: IndexType
{
    matrix: &'a Matrix,
    position: (usize, usize),
    direction: MatrixDirection,
    ix: PhantomData<Ix>,
}

struct MatrixPosition<'a> {
    matrix: &'a Matrix,
    position: (usize, usize),
    direction: MatrixDirection,
}

impl<'a> MatrixPosition<'a> {
    fn edges(&self) -> EdgeIter<'a> {
        let (i, j) = self.position;
        let ref neighbors = self.matrix[i].as_ref().unwrap();
        neighbors[j].as_ref().unwrap().iter()
    }

    fn project(&self) -> usize {
        let (i, j) = self.position;

        match self.direction {
            MatrixDirection::Rows    => i,
            MatrixDirection::Columns => j,
        }
    }
}

#[derive(Eq, PartialEq, Copy, Clone)]
enum MatrixDirection {
    Rows,
    Columns,
}

#[derive(Eq, PartialEq)]
enum MatrixNeighborsState {
    Stop,
    Continue,
    Yield,
}

impl<'a, Ix> MatrixNeighbors<'a, Ix>
    where Ix: IndexType
{
    fn new(matrix: &'a Matrix, position: (usize, usize), direction: MatrixDirection) -> MatrixNeighbors<'a, Ix> {
        MatrixNeighbors {
            matrix: matrix,
            position: position,
            direction: direction,
            ix: PhantomData,
        }
    }

    fn on_columns(matrix: &'a Matrix, ix: NodeIndex<Ix>) -> MatrixNeighbors<'a, Ix> {
        let position = (ix.index(), 0);
        let direction = MatrixDirection::Columns;

        MatrixNeighbors::new(matrix, position, direction)
    }

    fn on_rows(matrix: &'a Matrix, ix: NodeIndex<Ix>) -> MatrixNeighbors<'a, Ix> {
        let position = (0, ix.index());
        let direction = MatrixDirection::Rows;

        MatrixNeighbors::new(matrix, position, direction)
    }
}

impl<'a, Ix> Iterator for MatrixNeighbors<'a, Ix>
    where Ix: IndexType
{
    type Item = MatrixPosition<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let (i, j) = self.position;

            // check state before continuing
            let state = {
                if i >= self.matrix.len() {
                    MatrixNeighborsState::Stop
                } else if let Some(ref columns) = self.matrix[i] {
                    if j >= columns.len() {
                        MatrixNeighborsState::Stop
                    } else if columns[j].is_some() {
                        MatrixNeighborsState::Yield
                    } else {
                        MatrixNeighborsState::Continue
                    }
                } else if self.direction == MatrixDirection::Columns {
                    MatrixNeighborsState::Stop
                } else {
                    MatrixNeighborsState::Continue
                }
            };

            // advance unless at end
            if state != MatrixNeighborsState::Stop {
                let (ref mut y, ref mut x) = self.position;

                let mut n = match self.direction {
                    MatrixDirection::Rows    => y,
                    MatrixDirection::Columns => x,
                };

                *n += 1;
            }

            // early return if there's a value for the current state
            match state {
                MatrixNeighborsState::Yield => {
                    return Some(MatrixPosition {
                        position: (i, j),
                        matrix: &self.matrix,
                        direction: self.direction,
                    })
                },

                MatrixNeighborsState::Stop     => return None,
                MatrixNeighborsState::Continue => {}
            };
        }
    }
}

pub struct Neighbors<'a, Ix>
    where Ix: IndexType,
{
    neighbors: MatrixNeighbors<'a, Ix>,
}

impl<'a, Ix> Neighbors<'a, Ix>
    where Ix: IndexType
{
    fn new(neighbors: MatrixNeighbors<'a, Ix>) -> Neighbors<'a, Ix> {
        Neighbors {
            neighbors: neighbors,
        }
    }
}

impl<'a, Ix> Iterator for Neighbors<'a, Ix>
    where Ix: IndexType
{
    type Item = NodeIndex<Ix>;

    fn next(&mut self) -> Option<NodeIndex<Ix>> {
        self.neighbors.next()
            .map(|p| p.project())
            .map(node_index)
    }
}

pub struct Edges<'a, Ix>
    where Ix: IndexType,
{
    neighbors: MatrixNeighbors<'a, Ix>,
    iter: Option<EdgeIter<'a>>,
}

impl<'a, Ix> Edges<'a, Ix>
    where Ix: IndexType,
{
    fn new(neighbors: MatrixNeighbors<'a, Ix>) -> Edges<'a, Ix> {
        Edges {
            neighbors: neighbors,
            iter: None,
        }
    }
}

impl<'a, Ix> Edges<'a, Ix>
    where Ix: IndexType,
{
    fn wrapped_next(&mut self) -> Option<&usize> {
        self.iter = self.neighbors.next().map(|p| p.edges());

        self.iter
            .as_mut()
            .and_then(|mut it| it.next())
    }
}

impl<'a, Ix> Iterator for Edges<'a, Ix>
    where Ix: IndexType,
{
    type Item = EdgeIndex<Ix>;

    fn next(&mut self) -> Option<Self::Item> {
        let edge = if self.iter.is_none() {
            self.wrapped_next()
        } else {
            let edge = self.iter.as_mut().unwrap().next();
            edge.or_else(|| self.wrapped_next())
        };

        edge.map(|e| EdgeIndex::new(*e))
    }
}

impl<N, E, Ty, Ix> GraphBase for DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone
{
    type NodeId = NodeIndex<Ix>;
    type EdgeId = EdgeIndex<Ix>;
}

impl<N, E, Ty, Ix> NodeCount for DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone
{
    fn node_count(&self) -> usize {
        (*self).node_count()
    }
}


impl<N, E, Ty, Ix> GetAdjacencyMatrix for DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone,
{
    type AdjMatrix = ();

    fn adjacency_matrix(&self) -> Self::AdjMatrix {
    }

    fn is_adjacent(&self, _: &Self::AdjMatrix, a: NodeIndex<Ix>, b: NodeIndex<Ix>) -> bool {
        self.contains_edge(a, b)
    }
}

impl<N, E, Ty, Ix> Visitable for DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone,
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

impl<'a, N, E: 'a, Ty, Ix> IntoNeighbors for &'a DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone,
{
    type Neighbors = Neighbors<'a, Ix>;

    fn neighbors(self, n: NodeIndex<Ix>) -> Neighbors<'a, Ix>
    {
        DenseGraph::neighbors(self, n)
    }
}

impl<'a, N, E: 'a, Ix> IntoNeighborsDirected for &'a DenseGraph<N, E, Directed, Ix>
    where Ix: IndexType,
          N: Clone,
          E: Clone,
{
    type NeighborsDirected = Neighbors<'a, Ix>;

    fn neighbors_directed(self, n: NodeIndex<Ix>, d: Direction) -> Neighbors<'a, Ix>
    {
        DenseGraph::neighbors_directed(self, n, d)
    }
}

impl<'a, N, E, Ty, Ix> IntoNodeIdentifiers for &'a DenseGraph<N, E, Ty, Ix>
    where Ty: EdgeType,
          Ix: IndexType,
          N: Clone,
          E: Clone,
{
    type NodeIdentifiers = NodeIndices<'a, Ix>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        DenseGraph::node_indices(self)
    }
}

