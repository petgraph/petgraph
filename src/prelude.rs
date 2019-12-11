
//! Commonly used items.
//!
//! ```
//! use petgraph::prelude::*;
//! ```

#[doc(no_inline)]
pub use crate::graph::{
    Graph,
    NodeIndex,
    EdgeIndex,
    DiGraph,
    UnGraph,
};
#[cfg(feature = "graphmap")]
#[doc(no_inline)]
pub use crate::graphmap::{
    GraphMap,
    DiGraphMap,
    UnGraphMap,
};
#[doc(no_inline)]
#[cfg(feature = "stable_graph")]
pub use crate::stable_graph::{
    StableGraph,
    StableDiGraph,
    StableUnGraph,
};
#[doc(no_inline)]
pub use crate::visit::{
    Bfs,
    Dfs,
    DfsPostOrder,
};
#[doc(no_inline)]
pub use crate::{
    Direction,
    Incoming,
    Outgoing,
    Directed,
    Undirected,
};

#[doc(no_inline)]
pub use crate::visit::{
    EdgeRef,
};
