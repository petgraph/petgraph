#![no_std]
extern crate alloc;

use core::hash::Hash;

use hashbrown::HashMap;
use petgraph_core::{storage::reverse::ReverseGraphStorage, GraphDirectionality, Node};
use petgraph_dino::{DinoStorage, NodeId};

pub struct Entry<K, V> {
    key: K,
    value: V,
}

pub enum MapError {
    DinoError,
    NodeExists,
    EdgeExists,
}

// TODO: better name
// TODO: reduce generics
pub struct MapStorage<NK, NV, EK, EV, D>
where
    D: GraphDirectionality,
    NK: Eq + Hash,
    EK: Eq + Hash,
{
    inner: DinoStorage<Entry<NK, NV>, Entry<EK, EV>, D>,
    nodes: HashMap<NK, NodeId>,
    edges: HashMap<EK, NodeId>,
}

// impl<NK, NV, EK, EV, D> ReverseGraphStorage for MapStorage<NK, NV, EK, EV, D>
// where
//     D: GraphDirectionality,
//     NK: Eq + Hash,
//     EK: Eq + Hash,
// {
//     type EdgeKey = EK;
//     type NodeKey = NK;
//
//     fn node_by_weight(&self, weight: &Self::NodeKey) -> Option<Node<Self>> {
//         let id = self.nodes.get(weight).copied()?;
//
//         self.node(id)
//     }
//
//     fn contains_node_weight(&self, weight: &Self::NodeKey) -> bool {
//         self.nodes.contains_key(weight)
//     }
//
//     fn edge_by_weight(&self, weight: &Self::EdgeKey) -> Option<Node<Self>> {
//         let id = self.edges.get(weight).copied()?;
//
//         self.edge(id)
//     }
//
//     fn contains_edge_weight(&self, weight: &Self::EdgeKey) -> bool {
//         self.edges.contains_key(weight)
//     }
// }
