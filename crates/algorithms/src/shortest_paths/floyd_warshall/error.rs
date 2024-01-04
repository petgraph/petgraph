use alloc::vec::Vec;
use core::{
    fmt,
    fmt::{Debug, Display, Formatter},
};

use error_stack::Context;
use petgraph_core::node::NodeId;

/// An error that can occur during the Floyd-Warshall algorithm.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum FloydWarshallError {
    /// The graph contains a negative cycle.
    ///
    /// The error attaches all nodes where a negative cycle was detected via the [`NegativeCycle`]
    /// type.
    ///
    /// Note that multiple negative cycles may exist in the graph, and each attachment only means
    /// that it is part of a negative cycle and not which cycle(s) it is part of.
    ///
    /// To find all negative cycles, use [`BellmanFord`] instead.
    ///
    /// [`BellmanFord`]: crate::shortest_paths::BellmanFord
    NegativeCycle,
}

impl Display for FloydWarshallError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::NegativeCycle => f.write_str("graph contains a negative cycle"),
        }
    }
}

impl Context for FloydWarshallError {}

/// A node that is part of a negative cycle.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct NegativeCycle(NodeId);

impl NegativeCycle {
    pub(crate) fn new(node: NodeId) -> Self {
        Self(node)
    }

    /// Returns the node that is part of a negative cycle.
    pub fn node(&self) -> NodeId {
        self.0
    }
}
