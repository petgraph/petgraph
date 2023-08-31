//! `petgraph` is a graph data structure library.
//!
//! Graphs are collections of nodes, and edges between nodes. `petgraph`
//! provides several [graph types](index.html#graph-types) (each differing in the
//! tradeoffs taken in their internal representation),
//! [algorithms](./algo/index.html#functions) on those graphs, and functionality to
//! [output graphs](./dot/struct.Dot.html) in
//! [`graphviz`](https://www.graphviz.org/) format. Both nodes and edges
//! can have arbitrary associated data, and edges may be either directed or undirected.
//!
//! # Example
//!
//! ```rust
//! use petgraph::{
//!     algorithms::{shortest_paths::dijkstra, tree::minimum_spanning_tree},
//!     data::FromElements,
//!     dot::{Dot, RenderOption},
//!     graph::{NodeIndex, UnGraph},
//! };
//!
//! // Create an undirected graph with `i32` nodes and edges with `()` associated data.
//! let g = UnGraph::<i32, ()>::from_edges(&[(1, 2), (2, 3), (3, 4), (1, 4)]);
//!
//! // Find the shortest path from `1` to `4` using `1` as the cost for every edge.
//! let node_map = dijkstra(&g, 1.into(), Some(4.into()), |_| 1);
//! assert_eq!(&1i32, node_map.get(&NodeIndex::new(4)).unwrap());
//!
//! // Get the minimum spanning tree of the graph as a new graph, and check that
//! // one edge was trimmed.
//! let mst = UnGraph::<_, _>::from_elements(minimum_spanning_tree(&g));
//! assert_eq!(g.raw_edges().len() - 1, mst.raw_edges().len());
//!
//! // Output the tree to `graphviz` `DOT` format
//! println!(
//!     "{:?}",
//!     Dot::with_config(&mst, &[RenderOption::NoEdgeLabels])
//! );
//! // graph {
//! //     0 [label="\"0\""]
//! //     1 [label="\"0\""]
//! //     2 [label="\"0\""]
//! //     3 [label="\"0\""]
//! //     1 -- 2
//! //     3 -- 4
//! //     2 -- 3
//! // }
//! ```
//!
//! # Graph types
//!
//! * [`Graph`](./graph/struct.Graph.html) - An adjacency list graph with arbitrary associated data.
//! * [`StableGraph`](./stable_graph/struct.StableGraph.html) - Similar to `Graph`, but it keeps
//!   indices stable across removals.
//! * [`GraphMap`](./graphmap/struct.GraphMap.html) - An adjacency list graph backed by a hash
//!   table. The node identifiers are the keys into the table.
//! * [`MatrixGraph`](./matrix_graph/struct.MatrixGraph.html) - An adjacency matrix graph.
//! * [`CSR`](./csr/struct.Csr.html) - A sparse adjacency matrix graph with arbitrary associated
//!   data.
//!
//! ### Generic parameters
//!
//! Each graph type is generic over a handful of parameters. All graphs share 3 common
//! parameters, `N`, `E`, and `Ty`. This is a broad overview of what those are. Each
//! type's documentation will have finer detail on these parameters.
//!
//! `N` & `E` are called *weights* in this implementation, and are associated with
//! nodes and edges respectively. They can generally be of arbitrary type, and don't have to
//! be what you might conventionally consider weight-like. For example, using `&str` for `N`
//! will work. Many algorithms that require costs let you provide a cost function that
//! translates your `N` and `E` weights into costs appropriate to the algorithm. Some graph
//! types and choices do impose bounds on `N` or `E`.
//! [`min_spanning_tree`](./algo/fn.min_spanning_tree.html) for example requires edge weights that
//! implement [`PartialOrd`](https://doc.rust-lang.org/stable/core/cmp/trait.PartialOrd.html).
//! [`GraphMap`](./graphmap/struct.GraphMap.html) requires node weights that can serve as hash
//! map keys, since that graph type does not create standalone node indices.
//!
//! `Ty` controls whether edges are [`Directed`](./enum.Directed.html) or
//! [`Undirected`](./enum.Undirected.html).
//!
//! `Ix` appears on graph types that use indices. It is exposed so you can control
//! the size of node and edge indices, and therefore the memory footprint of your graphs.
//! Allowed values are `u8`, `u16`, `u32`, and `usize`, with `u32` being the default.
//!
//! ### Shorthand types
//!
//! Each graph type vends a few shorthand type definitions that name some specific
//! generic choices. For example, [`DiGraph<_, _>`](./graph/type.DiGraph.html) is shorthand
//! for [`Graph<_, _, Directed>`](graph/struct.Graph.html).
//! [`UnMatrix<_, _>`](./matrix_graph/type.UnMatrix.html) is shorthand for
//! [`MatrixGraph<_, _, Undirected>`](./matrix_graph/struct.MatrixGraph.html). Each graph type's
//! module documentation lists the available shorthand types.
//!
//! # Crate features
// TODO: rework
//!
//! * **serde-1** -
//!   Defaults off. Enables serialization for ``Graph, StableGraph, GraphMap`` using
//!   [`serde 1.0`](https://crates.io/crates/serde). May require a more recent version
//!   of Rust than petgraph alone.
//! * **graphmap** - Defaults on. Enables [`GraphMap`](./graphmap/struct.GraphMap.html).
//! * **stable_graph** - Defaults on. Enables
//!   [`StableGraph`](./stable_graph/struct.StableGraph.html).
//! * **matrix_graph** - Defaults on. Enables
//!   [`MatrixGraph`](./matrix_graph/struct.MatrixGraph.html).

#[doc(no_inline)]
pub use petgraph_graph::Graph;

#[deprecated(since = "0.7.0", note = "use explicit imports instead of the prelude")]
pub mod prelude;

/// `Graph<N, E, Ty, Ix>` is a graph datastructure using an adjacency list representation.
pub mod graph {
    pub use petgraph_core::id::{DefaultIx, IndexType};
    pub use petgraph_graph::*;
}

#[cfg(feature = "stable-graph")]
#[deprecated(
    since = "0.7.0",
    note = "use `graph::stable` instead of `stable_graph`"
)]
pub mod stable_graph {
    pub use petgraph_core::id::{DefaultIx, IndexType};
    pub use petgraph_graph::{
        edge_index, node_index,
        stable::{
            EdgeIndices, EdgeReference, EdgeReferences, Edges, EdgesConnecting, Externals,
            Neighbors, NodeIndices, NodeReferences, StableDiGraph, StableGraph, StableUnGraph,
            WalkNeighbors,
        },
        GraphIndex, NodeIndex,
    };
}

#[cfg(feature = "adjacency-matrix")]
#[deprecated(since = "0.7.0", note = "use `adjacency_matrix` instead of `adj`")]
pub mod adj {
    #[deprecated(since = "0.7.0", note = "use `AdjacencyMatrix` instead")]
    pub use petgraph_adjacency_matrix::AdjacencyList as List;
    #[deprecated(since = "0.7.0", note = "use `UnweightedAdjacencyMatrix` instead")]
    pub use petgraph_adjacency_matrix::UnweightedAdjacencyList as UnweightedList;
    pub use petgraph_adjacency_matrix::*;
    pub use petgraph_core::id::{DefaultIx, IndexType};
}

#[cfg(feature = "adjacency-matrix")]
pub mod adjacency_matrix {
    pub use petgraph_adjacency_matrix::*;
}

#[cfg(feature = "csr")]
pub mod csr {
    pub use petgraph_core::id::{DefaultIx, IndexType};
    pub use petgraph_csr::*;
}

#[cfg(feature = "graphmap")]
pub mod graphmap {
    pub use petgraph_graphmap::*;
}

#[cfg(feature = "matrix-graph")]
pub mod matrix_graph {
    pub use petgraph_core::id::IndexType;
    pub use petgraph_matrix_graph::*;
}

#[deprecated(since = "0.7.0", note = "use `algorithms` instead of `algo`")]
pub mod algo {
    pub use petgraph_algorithms::{
        components::{condensation, connected_components, kosaraju_scc, tarjan_scc},
        connectivity::has_path_connecting,
        cycles::{greedy_feedback_arc_set, is_cyclic_directed, is_cyclic_undirected},
        dag::toposort,
        heuristics::{greedy_matching, maximum_matching, Matching},
        isomorphism::{
            is_isomorphic, is_isomorphic_matching, is_isomorphic_subgraph,
            is_isomorphic_subgraph_matching,
        },
        shortest_paths::{
            astar, bellman_ford, dijkstra, find_negative_cycle, floyd_warshall,
            k_shortest_path_length,
        },
        simple_paths::all_simple_paths,
        tree::minimum_spanning_tree,
    };
}

pub mod algorithms {
    pub use petgraph_algorithms::*;
}

#[deprecated(
    since = "0.7.0",
    note = "use `algorithms::operators` instead of `operator`"
)]
pub mod operator {
    pub use petgraph_algorithms::operators::*;
}

#[cfg(feature = "unstable-generate")]
#[deprecated(since = "0.7.0", note = "use `generator` instead of `generate`")]
pub mod generate {
    pub use petgraph_generate::*;
}

#[cfg(feature = "unstable-generate")]
pub mod generator {
    pub use petgraph_generate::*;
}

#[deprecated(since = "0.7.0", note = "use `core` instead of direct imports")]
pub use petgraph_core::{
    data,
    deprecated::IntoWeightedEdge,
    edge::{Directed, Direction, EdgeType, Incoming, Outgoing, Undirected},
    visit,
};

pub mod core {
    pub use petgraph_core::*;
}

#[cfg(feature = "io")]
#[deprecated(since = "0.7.0", note = "use `io` instead of `dot`")]
pub mod dot {
    pub use petgraph_io::dot::*;
}

#[cfg(feature = "io")]
pub mod io {
    pub use petgraph_io::*;
}
