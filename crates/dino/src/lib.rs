#![no_std]

mod closure;
mod edge;
mod node;

extern crate alloc;

use alloc::vec::Vec;

use hashbrown::{HashMap, HashSet};

use crate::{
    edge::{Edge, EdgeId},
    node::{Node, NodeId},
};

pub struct DinosaurStorage<N, E> {
    nodes: HashMap<NodeId, Node<N>>,
    edges: HashMap<EdgeId, Edge<E>>,

    // closures
    closures: Closures,
}

impl<N, E> DinosaurStorage<N, E> {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            edges: HashMap::new(),

            closures: Closures::new(),
        }
    }

    fn get_node(&self, id: NodeId) -> Option<&Node<N>> {
        self.nodes.get(&id)
    }

    fn get_edge(&self, id: EdgeId) -> Option<&Edge<E>> {
        self.edges.get(&id)
    }
}
