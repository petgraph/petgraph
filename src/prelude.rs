
//! Commonly used items.
//!
//! ```
//! use petgraph::prelude::*;
//! ```

#[doc(no_inline)]
pub use graph::{
    Graph,
    NodeIndex,
    EdgeIndex,
};
#[doc(no_inline)]
pub use graphmap::{
    GraphMap,
};
#[doc(no_inline)]
#[cfg(feature = "stable_graph")]
pub use stable_graph::{
    StableGraph,
};
#[doc(no_inline)]
pub use visit::{
    Bfs,
    Dfs,
    DfsPostOrder,
};
#[doc(no_inline)]
pub use ::{
    Direction,
    Incoming,
    Outgoing,
    Directed,
    Undirected,
};
