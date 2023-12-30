use core::marker::PhantomData;

use petgraph_core::{
    edge::{
        marker::{Directed, Undirected},
        Direction,
    },
    DirectedGraphStorage, Edge, GraphDirectionality, GraphStorage,
};

pub(in crate::shortest_paths) struct NodeConnections<'graph, S, D>
where
    S: GraphStorage,
    D: GraphDirectionality,
{
    storage: &'graph S,
    _marker: PhantomData<fn() -> *const D>,
}

impl<'graph, S> NodeConnections<'graph, S, Directed>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) fn directed(storage: &'graph S) -> Self {
        Self {
            storage,
            _marker: PhantomData,
        }
    }
}

impl<'graph, S> NodeConnections<'graph, S, Undirected>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) fn undirected(storage: &'graph S) -> Self {
        Self {
            storage,
            _marker: PhantomData,
        }
    }
}

pub(in crate::shortest_paths) trait Connections<'a, S>
where
    S: GraphStorage + 'a,
{
    fn connections(&self, node: S::NodeId) -> impl Iterator<Item = Edge<'a, S>> + 'a;
}

impl<'graph, S> Connections<'graph, S> for NodeConnections<'graph, S, Directed>
where
    S: DirectedGraphStorage,
{
    fn connections(&self, node: S::NodeId) -> impl Iterator<Item = Edge<'graph, S>> + 'graph {
        self.storage
            .node_directed_connections(node, Direction::Outgoing)
    }
}

impl<'graph, S> Connections<'graph, S> for NodeConnections<'graph, S, Undirected>
where
    S: GraphStorage,
{
    fn connections(&self, node: S::NodeId) -> impl Iterator<Item = Edge<'graph, S>> + 'graph {
        self.storage.node_connections(node)
    }
}
