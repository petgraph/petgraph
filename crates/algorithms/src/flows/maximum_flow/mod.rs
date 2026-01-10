//! Collection of algorithms for the [Maximum Flow Problem][max_flow_wikipedia].
//!
//!
//!
//! Currently, `petgraph` provides two algorithms to compute the maximum flow
//! in a flow network:
//! - [Dinic's Algorithm][dinics_wikipedia]
//! - [Edmonds-Karp Algorithm][edmonds_karp_wikipedia]
//! They are implemented in the functions [`dinics`] and [`ford_fulkerson`] and can be found
//! in their respective submodules.
//!
//! [Dinics] and [Edmonds] have different time complexities, and
//! their performance can vary significantly depending on the input graph.
//! In general, [dinics] is faster, especially on dense graphs, graphs with
//! unit capacities, and bipartite graphs.
//! [ford_fulkerson] may be a better choice when working with small or
//! sparse graphs.
//!
//! For more information about each algorithm and their detailed time
//! complexity, check their respective documentation.
//!
//! [dinics_wikipedia]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
//! [edmonds_karp_wikipedia]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
//! [max_flow_wikipedia]: https://en.wikipedia.org/wiki/Maximum_flow_problem

mod dinics;
mod ford_fulkerson;

pub use dinics::dinics;
pub use ford_fulkerson::ford_fulkerson;
