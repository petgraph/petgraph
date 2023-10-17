//! Compatability implementations for deprecated graph traits.
#![allow(deprecated)]

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

use bitvec::{boxed::BitBox, vec::BitVec};

#[cfg(feature = "alloc")]
use crate::deprecated::data::{Build, Create, DataMap, FromElements};
#[cfg(feature = "fixedbitset")]
use crate::deprecated::visit::GetAdjacencyMatrix;
use crate::{
    deprecated::visit::{
        Data, EdgeCount, EdgeIndexable, FilterEdge, FilterNode, GraphBase, IntoEdgeReferences,
        IntoEdges, IntoEdgesDirected, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers,
        IntoNodeReferences, NodeCompactIndexable, NodeCount, NodeIndexable, VisitMap, Visitable,
    },
    edge::{Direction, Edge},
    graph::Graph,
    id::{Continuous, ContinuousIndexMapper, IndexMapper, LinearGraphId, ManagedGraphId},
    node::Node,
    storage::{DirectedGraphStorage, GraphStorage},
};

impl<S> GraphBase for Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type EdgeId = S::EdgeId;
    type NodeId = S::NodeId;
}

// TODO: GraphProp?!

impl<S> NodeCount for Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    fn node_count(&self) -> usize {
        self.num_nodes()
    }
}

impl<S> NodeIndexable for Graph<S>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Copy,
    S::EdgeId: Copy,
{
    fn node_bound(&self) -> usize {
        self.num_nodes()
    }

    fn to_index(&self, a: Self::NodeId) -> usize {
        S::NodeId::index_mapper(&self.storage).map(&a)
    }

    fn from_index(&self, i: usize) -> Self::NodeId {
        S::NodeId::index_mapper(&self.storage)
            .reverse(&i)
            .expect("unable to determine index")
            .into_owned()
    }
}

impl<S> NodeCompactIndexable for Graph<S>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Copy,
    for<'a> <S::NodeId as LinearGraphId<S>>::Mapper<'a>:
        IndexMapper<S::NodeId, usize, Continuity = Continuous>,
    S::EdgeId: Copy,
{
}

impl<S> EdgeCount for Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    fn edge_count(&self) -> usize {
        self.num_edges()
    }
}

impl<S> EdgeIndexable for Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: LinearGraphId<S> + Copy,
    for<'a> <S::EdgeId as LinearGraphId<S>>::Mapper<'a>:
        IndexMapper<S::EdgeId, usize, Continuity = Continuous>,
{
    fn edge_bound(&self) -> usize {
        self.num_edges()
    }

    fn to_index(&self, a: Self::EdgeId) -> usize {
        S::EdgeId::index_mapper(&self.storage).map(&a)
    }

    fn from_index(&self, i: usize) -> Self::EdgeId {
        S::EdgeId::index_mapper(&self.storage)
            .reverse(&i)
            .expect("unable to determine index")
            .into_owned()
    }
}

impl<S> Data for Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type EdgeWeight = S::EdgeWeight;
    type NodeWeight = S::NodeWeight;
}

impl<S> IntoNodeIdentifiers for &Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type NodeIdentifiers = impl Iterator<Item = Self::NodeId>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.nodes().map(|node| *node.id())
    }
}

impl<'a, S> IntoNodeReferences for &'a Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type NodeRef = Node<'a, S>;

    type NodeReferences = impl Iterator<Item = Self::NodeRef>;

    fn node_references(self) -> Self::NodeReferences {
        self.nodes()
    }
}

impl<'a, S> IntoEdgeReferences for &'a Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type EdgeRef = Edge<'a, S>;

    type EdgeReferences = impl Iterator<Item = Self::EdgeRef>;

    fn edge_references(self) -> Self::EdgeReferences {
        self.edges()
    }
}

#[cfg(feature = "alloc")]
impl<'a, S> IntoNeighbors for &'a Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type Neighbors = impl Iterator<Item = Self::NodeId>;

    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors {
        // This is a bit inefficient, but just glue code to deprecated trait impls, so fine.
        // We make use of `&n` in the trait and the iterator is bound to it, which means that we
        // need to collect to avoid having a lifetime we created in this code.
        // we _could_ in theory also use `Cow` here, but that's a bit overkill.
        self.neighbours(&n)
            .map(|node| *node.id())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(feature = "alloc")]
impl<'a, S> IntoNeighborsDirected for &'a Graph<S>
where
    S: DirectedGraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type NeighborsDirected = impl Iterator<Item = Self::NodeId>;

    fn neighbors_directed(self, n: Self::NodeId, d: Direction) -> Self::NeighborsDirected {
        // see comment in `IntoNeighbors` for why we need to collect here.
        self.neighbours_directed(&n, d)
            .map(|node| *node.id())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(feature = "alloc")]
impl<'a, S> IntoEdges for &'a Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type Edges = impl Iterator<Item = Self::EdgeRef>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        // see comment in `IntoNeighbors` for why we need to collect here.
        self.connections(&a).collect::<Vec<_>>().into_iter()
    }
}

#[cfg(feature = "alloc")]
impl<'a, S> IntoEdgesDirected for &'a Graph<S>
where
    S: DirectedGraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type EdgesDirected = impl Iterator<Item = Self::EdgeRef>;

    fn edges_directed(self, a: Self::NodeId, dir: Direction) -> Self::EdgesDirected {
        // see comment in `IntoNeighbors` for why we need to collect here.
        self.connections_directed(&a, dir)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

pub struct VisitationMap<'a, S, T>
where
    S: GraphStorage + 'a,
    T: LinearGraphId<S> + Clone + 'a,
{
    inner: BitBox,
    mapper: ContinuousIndexMapper<<T as LinearGraphId<S>>::Mapper<'a>, T>,
}

impl<'a, S> VisitationMap<'a, S, S::NodeId>
where
    S: GraphStorage + 'a,
    S::NodeId: LinearGraphId<S> + Clone,
{
    fn new_node(size: usize, mapper: <S::NodeId as LinearGraphId<S>>::Mapper<'a>) -> Self {
        Self {
            inner: BitVec::repeat(false, size).into_boxed_bitslice(),
            mapper: ContinuousIndexMapper::new(mapper),
        }
    }
}

impl<'a, S, T> VisitMap<T> for VisitationMap<'a, S, T>
where
    S: GraphStorage + 'a,
    T: LinearGraphId<S> + Clone,
{
    fn visit(&mut self, a: T) -> bool {
        let index = self.mapper.map(&a);

        !self.inner.replace(index, true)
    }

    fn is_visited(&self, a: &T) -> bool {
        let Some(index) = self.mapper.lookup(a) else {
            return false;
        };

        let Some(bit) = self.inner.get(index) else {
            return false;
        };

        *bit
    }
}

impl<'a, S> FilterNode<S::NodeId> for VisitationMap<'a, S, S::NodeId>
where
    S: GraphStorage + 'a,
    S::NodeId: LinearGraphId<S> + Clone,
{
    fn include_node(&self, node: S::NodeId) -> bool {
        self.is_visited(&node)
    }
}

impl<'a, S> FilterEdge<S::EdgeId> for VisitationMap<'a, S, S::EdgeId>
where
    S: GraphStorage + 'a,
    S::EdgeId: LinearGraphId<S> + Clone,
{
    fn include_edge(&self, edge: S::EdgeId) -> bool {
        self.is_visited(&edge)
    }
}

impl<'a, S> Visitable for &'a Graph<S>
where
    S: GraphStorage,
    S::NodeId: LinearGraphId<S> + Copy,
    S::EdgeId: Copy,
{
    type Map = VisitationMap<'a, S, S::NodeId>;

    fn visit_map(&self) -> Self::Map {
        VisitationMap::new_node(self.num_nodes(), S::NodeId::index_mapper(&self.storage))
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.inner.fill(false);
    }
}

#[cfg(feature = "fixedbitset")]
impl<S> GetAdjacencyMatrix for Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    type AdjMatrix = ();

    fn adjacency_matrix(&self) -> Self::AdjMatrix {}

    fn is_adjacent(&self, _: &Self::AdjMatrix, a: Self::NodeId, b: Self::NodeId) -> bool {
        self.edges_between(&a, &b).next().is_some()
    }
}

#[cfg(feature = "alloc")]
impl<S> DataMap for Graph<S>
where
    S: GraphStorage,
    S::NodeId: Copy,
    S::EdgeId: Copy,
{
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.node(&id).map(|node| node.weight())
    }

    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        self.edge(&id).map(|edge| edge.weight())
    }
}

#[cfg(feature = "alloc")]
impl<S> Build for Graph<S>
where
    S: GraphStorage,
    S::NodeId: ManagedGraphId + Copy,
    S::EdgeId: ManagedGraphId + Copy,
{
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId {
        *self.insert_node(weight).id()
    }

    fn update_edge(
        &mut self,
        a: Self::NodeId,
        b: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Self::EdgeId {
        *self.insert_edge(weight, &a, &b).id()
    }
}

#[cfg(feature = "alloc")]
impl<S> Create for Graph<S>
where
    S: GraphStorage,
    S::NodeId: ManagedGraphId + Copy,
    S::EdgeId: ManagedGraphId + Copy,
{
    fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self::with_capacity(Some(nodes), Some(edges))
    }
}

#[cfg(feature = "alloc")]
impl<S> FromElements for Graph<S>
where
    S: GraphStorage,
    S::NodeId: ManagedGraphId + Copy,
    S::EdgeId: ManagedGraphId + Copy,
{
}
