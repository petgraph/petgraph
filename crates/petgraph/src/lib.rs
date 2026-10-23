/*!
`petgraph` is a graph data structure library.

Graphs are collections of nodes, and edges between nodes. `petgraph`
provides several [graph types](index.html#graph-types) (each differing in the
tradeoffs taken in their internal representation),
[algorithms](./algo/index.html) on those graphs, and functionality to
[output graphs](./dot/struct.Dot.html) in
[`Graphviz`](https://www.graphviz.org/) format. Both nodes and edges
can have arbitrary associated data, and edges may be either directed or undirected.

TODO
*/
#![no_std]

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(feature = "std")]
extern crate std;

pub use petgraph_core as core;
#[cfg(feature = "petgraph-old")]
pub use petgraph_old;
