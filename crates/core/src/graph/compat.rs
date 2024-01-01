//! Compatability implementations for deprecated graph traits.
#![allow(deprecated)]

#[cfg(feature = "alloc")]
use alloc::vec::Vec;

#[cfg(feature = "alloc")]
use crate::deprecated::data::{Build, Create, DataMap, FromElements};
#[cfg(feature = "fixedbitset")]
use crate::deprecated::visit::GetAdjacencyMatrix;
use crate::{
    deprecated::visit::{
        Data, EdgeCount, EdgeIndexable, FilterNode, GraphBase, IntoEdgeReferences, IntoEdges,
        IntoEdgesDirected, IntoNeighbors, IntoNeighborsDirected, IntoNodeIdentifiers,
        IntoNodeReferences, NodeCompactIndexable, NodeCount, NodeIndexable, VisitMap, Visitable,
    },
    edge::{Direction, Edge, EdgeId},
    graph::Graph,
    node::{Node, NodeId},
    storage::{
        auxiliary::{BooleanGraphStorage, Hints},
        linear::IndexMapper,
        AuxiliaryGraphStorage, DirectedGraphStorage, GraphStorage, LinearGraphStorage,
    },
};

impl<S> GraphBase for Graph<S>
where
    S: GraphStorage,
{
    type EdgeId = EdgeId;
    type NodeId = NodeId;
}

// TODO: GraphProp?!

impl<S> NodeCount for Graph<S>
where
    S: GraphStorage,
{
    fn node_count(&self) -> usize {
        self.num_nodes()
    }
}

impl<S> NodeIndexable for Graph<S>
where
    S: LinearGraphStorage,
{
    fn node_bound(&self) -> usize {
        self.num_nodes()
    }

    fn to_index(&self, a: Self::NodeId) -> usize {
        self.storage.node_index_mapper().index(a)
    }

    fn from_index(&self, i: usize) -> Self::NodeId {
        self.storage
            .node_index_mapper()
            .reverse(i)
            .expect("unable to determine index")
    }
}

impl<S> NodeCompactIndexable for Graph<S> where S: LinearGraphStorage {}

impl<S> EdgeCount for Graph<S>
where
    S: GraphStorage,
{
    fn edge_count(&self) -> usize {
        self.num_edges()
    }
}

impl<S> EdgeIndexable for Graph<S>
where
    S: LinearGraphStorage,
{
    fn edge_bound(&self) -> usize {
        self.num_edges()
    }

    fn to_index(&self, a: Self::EdgeId) -> usize {
        self.storage.edge_index_mapper().index(a)
    }

    fn from_index(&self, i: usize) -> Self::EdgeId {
        self.storage
            .edge_index_mapper()
            .reverse(i)
            .expect("unable to determine index")
    }
}

impl<S> Data for Graph<S>
where
    S: GraphStorage,
{
    type EdgeWeight = S::EdgeWeight;
    type NodeWeight = S::NodeWeight;
}

impl<S> IntoNodeIdentifiers for &Graph<S>
where
    S: GraphStorage,
{
    type NodeIdentifiers = impl Iterator<Item = Self::NodeId>;

    fn node_identifiers(self) -> Self::NodeIdentifiers {
        self.nodes().map(|node| node.id())
    }
}

impl<'a, S> IntoNodeReferences for &'a Graph<S>
where
    S: GraphStorage,
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
{
    type Neighbors = impl Iterator<Item = Self::NodeId>;

    fn neighbors(self, n: Self::NodeId) -> Self::Neighbors {
        // This is a bit inefficient, but just glue code to deprecated trait impls, so fine.
        // We make use of `&n` in the trait and the iterator is bound to it, which means that we
        // need to collect to avoid having a lifetime we created in this code.
        // we _could_ in theory also use `Cow` here, but that's a bit overkill.
        self.neighbours(n)
            .map(|node| node.id())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(feature = "alloc")]
impl<'a, S> IntoNeighborsDirected for &'a Graph<S>
where
    S: DirectedGraphStorage,
{
    type NeighborsDirected = impl Iterator<Item = Self::NodeId>;

    fn neighbors_directed(self, n: Self::NodeId, d: Direction) -> Self::NeighborsDirected {
        // see comment in `IntoNeighbors` for why we need to collect here.
        self.neighbours_directed(n, d)
            .map(|node| node.id())
            .collect::<Vec<_>>()
            .into_iter()
    }
}

#[cfg(feature = "alloc")]
impl<'a, S> IntoEdges for &'a Graph<S>
where
    S: GraphStorage,
{
    type Edges = impl Iterator<Item = Self::EdgeRef>;

    fn edges(self, a: Self::NodeId) -> Self::Edges {
        // see comment in `IntoNeighbors` for why we need to collect here.
        self.connections(a).collect::<Vec<_>>().into_iter()
    }
}

#[cfg(feature = "alloc")]
impl<'a, S> IntoEdgesDirected for &'a Graph<S>
where
    S: DirectedGraphStorage,
{
    type EdgesDirected = impl Iterator<Item = Self::EdgeRef>;

    fn edges_directed(self, a: Self::NodeId, dir: Direction) -> Self::EdgesDirected {
        // see comment in `IntoNeighbors` for why we need to collect here.
        self.connections_directed(a, dir)
            .collect::<Vec<_>>()
            .into_iter()
    }
}

pub struct NodeVisitationMap<'a, S>
where
    S: AuxiliaryGraphStorage + 'a,
{
    inner: S::BooleanNodeStorage<'a>,
}

impl<'a, S> VisitMap<NodeId> for NodeVisitationMap<'a, S>
where
    S: AuxiliaryGraphStorage + 'a,
{
    fn visit(&mut self, a: NodeId) -> bool {
        self.inner.set(a, true).is_none()
    }

    fn is_visited(&self, a: &NodeId) -> bool {
        self.inner.get(*a).unwrap_or(false)
    }
}

impl<'a, S> FilterNode<NodeId> for NodeVisitationMap<'a, S>
where
    S: AuxiliaryGraphStorage + 'a,
{
    fn include_node(&self, node: NodeId) -> bool {
        self.is_visited(&node)
    }
}

impl<'a, S> Visitable for &'a Graph<S>
where
    S: AuxiliaryGraphStorage,
{
    type Map = NodeVisitationMap<'a, S>;

    fn visit_map(&self) -> Self::Map {
        NodeVisitationMap {
            inner: self.storage.boolean_node_storage(Hints::default()),
        }
    }

    fn reset_map(&self, map: &mut Self::Map) {
        map.inner.fill(false);
    }
}

impl<S> GetAdjacencyMatrix for Graph<S>
where
    S: GraphStorage,
{
    type AdjMatrix = ();

    fn adjacency_matrix(&self) -> Self::AdjMatrix {}

    fn is_adjacent(&self, (): &Self::AdjMatrix, a: Self::NodeId, b: Self::NodeId) -> bool {
        self.edges_between(a, b).next().is_some()
    }
}

#[cfg(feature = "alloc")]
impl<S> DataMap for Graph<S>
where
    S: GraphStorage,
{
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.node(id).map(|node| node.weight())
    }

    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        self.edge(id).map(|edge| edge.weight())
    }
}

#[cfg(feature = "alloc")]
impl<S> Build for Graph<S>
where
    S: GraphStorage,
{
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId {
        self.insert_node(weight).id()
    }

    fn update_edge(
        &mut self,
        a: Self::NodeId,
        b: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Self::EdgeId {
        self.insert_edge(weight, a, b).id()
    }
}

#[cfg(feature = "alloc")]
impl<S> Create for Graph<S>
where
    S: GraphStorage,
{
    fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self::with_capacity(Some(nodes), Some(edges))
    }
}

#[cfg(feature = "alloc")]
impl<S> FromElements for Graph<S> where S: GraphStorage {}
