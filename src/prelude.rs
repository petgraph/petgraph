//! Commonly used items.
//!
//! ```
//! use petgraph::prelude::*;
//! ```

#[doc(no_inline)]
pub use petgraph_core::deprecated::edge::{Directed, Undirected};
#[doc(no_inline)]
pub use petgraph_core::deprecated::visit::EdgeRef;
#[doc(no_inline)]
pub use petgraph_core::deprecated::visit::{Bfs, Dfs, DfsPostOrder};
pub use petgraph_core::edge::{
    Direction,
    Direction::{Incoming, Outgoing},
};
#[doc(no_inline)]
#[cfg(feature = "stable_graph")]
pub use petgraph_graph::stable::{StableDiGraph, StableGraph, StableUnGraph};
#[doc(no_inline)]
pub use petgraph_graph::{DiGraph, EdgeIndex, Graph, NodeIndex, UnGraph};

#[cfg(feature = "graphmap")]
#[doc(no_inline)]
pub use crate::graphmap::{DiGraphMap, GraphMap, UnGraphMap};
