#![feature(return_position_impl_trait_in_trait)]
#![no_std]

mod closure;
mod edge;
mod node;

extern crate alloc;

use alloc::vec::Vec;

pub use edge::EdgeId;
use hashbrown::{HashMap, HashSet};
pub use node::NodeId;
use petgraph_core::{
    edge::{DetachedEdge, EdgeMut},
    node::{DetachedNode, NodeMut},
    storage::GraphStorage,
};

use crate::{closure::Closures, edge::Edge, node::Node};

pub struct Undirected;
pub struct Directed;

pub struct DinosaurStorage<N, E, D = Directed> {
    nodes: HashMap<NodeId, Node<N>>,
    edges: HashMap<EdgeId, Edge<E>>,

    closures: Closures,

    _marker: core::marker::PhantomData<fn() -> *const D>,
}

impl<N, E, D> DinosaurStorage<N, E, D> {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),

            closures: Closures::new(),

            _marker: core::marker::PhantomData,
        }
    }

    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self {
        Self {
            nodes: HashMap::with_capacity(node_capacity.unwrap_or(0)),
            edges: HashMap::with_capacity(edge_capacity.unwrap_or(0)),

            closures: Closures::new(),

            _marker: core::marker::PhantomData,
        }
    }

    fn get_node(&self, id: NodeId) -> Option<&Node<N>> {
        self.nodes.get(&id)
    }

    fn get_edge(&self, id: EdgeId) -> Option<&Edge<E>> {
        self.edges.get(&id)
    }
}

impl<N, E, D> GraphStorage for DinosaurStorage<N, E, D> {
    type EdgeId = EdgeId;
    type EdgeWeight = E;
    type Error = ();
    type NodeId = NodeId;
    type NodeWeight = N;

    fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self {
        Self::with_capacity(node_capacity, edge_capacity)
    }

    fn into_parts(
        self,
    ) -> (
        impl Iterator<Item = DetachedNode<Self::NodeId, Self::NodeWeight>>,
        impl Iterator<Item = DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>>,
    ) {
        todo!()
    }

    // TODO: give context (with graph)
    fn insert_node(
        &mut self,
        id: Self::NodeId,
        weight: Self::NodeWeight,
    ) -> Result<petgraph_core::node::Node<Self>, Self::Error> {
        // TODO: this won't work (update closure needs mutable self)
        let node = self.nodes.entry(id).insert(Node::new(id, weight)).get();
        self.closures.update_node(id);

        Ok(petgraph_core::node::Node::new(
            graph,
            &node.id,
            &node.weight,
        ))
    }

    fn insert_edge(
        &mut self,
        id: Self::EdgeId,
        source: Self::NodeId,
        target: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Result<petgraph_core::edge::Edge<Self>, Self::Error> {
        let edge = self
            .edges
            .entry(id)
            .insert(Edge::new(id, weight, source, target))
            .get();
        self.closures.update_edge(id, self);

        Ok(petgraph_core::edge::Edge::new(
            graph,
            &edge.id,
            &edge.weight,
            &edge.source,
            &edge.target,
        ))
    }

    fn remove_node(
        &mut self,
        id: &Self::NodeId,
    ) -> Option<DetachedNode<Self::NodeId, Self::NodeWeight>> {
        let node = self.nodes.remove(id)?;
        self.closures.remove_node(*id);

        Some(DetachedNode::new(node.id, node.weight))
    }

    fn remove_edge(
        &mut self,
        id: &Self::EdgeId,
    ) -> Option<DetachedEdge<Self::EdgeId, Self::NodeId, Self::EdgeWeight>> {
        let edge = self.edges.remove(id)?;
        self.closures.remove_edge(*id, self);

        Some(DetachedEdge::new(
            edge.id,
            edge.weight,
            edge.source,
            edge.target,
        ))
    }

    fn node(&self, id: &Self::NodeId) -> Option<petgraph_core::node::Node<Self>> {
        self.nodes
            .get(id)
            .map(|node| petgraph_core::node::Node::new(graph, &node.id, &node.weight))
    }

    fn node_mut(&mut self, id: &Self::NodeId) -> Option<NodeMut<Self>> {
        self.nodes
            .get_mut(id)
            .map(|node| NodeMut::new(graph, &node.id, &mut node.weight))
    }

    fn edge(&self, id: &Self::EdgeId) -> Option<petgraph_core::edge::Edge<Self>> {
        self.edges.get(id).map(|edge| {
            petgraph_core::edge::Edge::new(
                graph,
                &edge.id,
                &edge.weight,
                &edge.source,
                &edge.target,
            )
        })
    }

    fn edge_mut(&mut self, id: &Self::EdgeId) -> Option<EdgeMut<Self>> {
        self.edges.get_mut(id).map(|edge| {
            EdgeMut::new(
                graph,
                &edge.id,
                &edge.source,
                &edge.target,
                &mut edge.weight,
            )
        })
    }

    fn node_connections<'a: 'b, 'b>(
        &'a self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = petgraph_core::edge::Edge<'a, Self>> + 'b {
        todo!()
    }

    fn node_connections_mut<'a: 'b, 'b>(
        &'a mut self,
        id: &'b Self::NodeId,
    ) -> impl Iterator<Item = EdgeMut<'a, Self>> + 'b {
        todo!()
    }

    fn nodes(&self) -> impl Iterator<Item = petgraph_core::node::Node<Self>> {
        todo!()
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<Self>> {
        todo!()
    }

    fn edges(&self) -> impl Iterator<Item = petgraph_core::edge::Edge<Self>> {
        todo!()
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<Self>> {
        todo!()
    }
}
