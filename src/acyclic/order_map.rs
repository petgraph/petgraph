//! A bi-map between node indices and their position in a topological order.
//!
//! This data structure is an implementation detail and is not exposed in the
//! public API.
use std::{collections::BTreeSet, fmt};

use crate::{
    algo::{toposort, Cycle},
    visit::{GraphBase, IntoNeighborsDirected, IntoNodeIdentifiers, NodeIndexable, Visitable},
};

/// A bijective map between node indices and their position in a topological order.
#[derive(Clone)]
pub(super) struct OrderMap<N> {
    /// The topological order of the nodes.
    order: Vec<N>,
    /// The inverse of `order`, i.e. for each node index, its position in `order`
    /// (requires `NodeIndexable`).
    order_inv: Vec<usize>,
}

impl<N> Default for OrderMap<N> {
    fn default() -> Self {
        Self {
            order: Default::default(),
            order_inv: Default::default(),
        }
    }
}

impl<N: fmt::Debug> fmt::Debug for OrderMap<N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OrderMap")
            .field("order", &self.order)
            .finish()
    }
}

impl<N: Copy> OrderMap<N> {
    pub(super) fn try_from_graph<G>(graph: G) -> Result<Self, Cycle<G::NodeId>>
    where
        G: NodeIndexable<NodeId = N> + IntoNeighborsDirected + IntoNodeIdentifiers + Visitable,
    {
        let order = toposort(graph, None)?;
        let mut order_inv = vec![0; graph.node_bound()];
        for (i, &id) in order.iter().enumerate() {
            order_inv[graph.to_index(id)] = i;
        }
        Ok(Self { order, order_inv })
    }

    pub(super) fn with_capacity(nodes: usize) -> Self {
        Self {
            order: Vec::with_capacity(nodes),
            order_inv: Vec::with_capacity(nodes),
        }
    }

    /// Map a node to its position in the topological order.
    pub(super) fn get_position(&self, id: N, graph: impl NodeIndexable<NodeId = N>) -> usize {
        self.order_inv[graph.to_index(id)]
    }

    /// Map a position in the topological order to a node.
    pub(super) fn at_position(&self, pos: usize) -> N {
        self.order[pos]
    }

    pub(super) fn as_slice(&self) -> &[N] {
        &self.order
    }

    pub(super) fn add_node(&mut self, id: N, graph: impl NodeIndexable<NodeId = N>) {
        self.order.push(id);
        let pos = self.order.len() - 1;
        let idx = graph.to_index(id);
        // Make sure the order_inv is large enough.
        if idx >= self.order_inv.len() {
            self.order_inv.resize(graph.node_bound(), 0);
        }
        self.order_inv[idx] = pos;
    }

    /// Remove a node from the order map.
    ///
    /// This is currently inefficient (O(v) runtime) because the topological
    /// order is stored in a contiguous array.
    pub(super) fn remove_node(&mut self, id: N, graph: impl NodeIndexable<NodeId = N>) {
        let idx = graph.to_index(id);
        let pos = self.order_inv[idx];
        self.order_inv[idx] = 0;
        self.order.remove(pos);
        // Adjust the positions for the nodes that have moved up.
        for n in &self.order[pos..] {
            let n_idx = graph.to_index(*n);
            self.order_inv[n_idx] -= 1;
        }
    }

    /// Remove multiple nodes from the order map.
    ///
    /// This function exists because in the current implementation, removing one
    /// node costs the same as removing many.
    pub(super) fn remove_nodes(
        &mut self,
        nodes: impl IntoIterator<Item = N>,
        graph: &impl NodeIndexable<NodeId = N>,
    ) {
        let nodes = nodes.into_iter();

        // Get and reset the positions of the nodes to remove.
        let drain_positions = nodes.map(|n| {
            let idx = graph.to_index(n);
            let pos = self.order_inv[idx];
            self.order_inv[idx] = 0;
            pos
        });
        let positions: BTreeSet<_> = drain_positions.collect();

        // Retain only the nodes that are not in the positions set.
        let mut callback_pos = 0;
        self.order.retain(|_| {
            let keep = !positions.contains(&callback_pos);
            callback_pos += 1;
            keep
        });

        // Adjust the positions for the nodes that have moved up.
        if let Some(&smallest_pos) = positions.first() {
            for (pos_offset, n) in self.order[smallest_pos..].iter().enumerate() {
                let n_idx = graph.to_index(*n);
                self.order_inv[n_idx] = pos_offset + smallest_pos;
            }
        }
    }

    pub(super) fn set_order(&mut self, id: N, pos: usize, graph: impl NodeIndexable<NodeId = N>) {
        self.order[pos] = id;
        self.order_inv[graph.to_index(id)] = pos;
    }
}

impl<G: Visitable> super::Acyclic<G> {
    /// Get the position of a node in the topological sort.
    pub fn get_position<'a>(&'a self, id: G::NodeId) -> usize
    where
        &'a G: NodeIndexable + GraphBase<NodeId = G::NodeId>,
    {
        self.order_map.get_position(id, &self.graph)
    }

    /// Get the node at a given position in the topological sort.
    pub fn at_position(&self, pos: usize) -> G::NodeId {
        self.order_map.at_position(pos)
    }
}
