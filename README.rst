
petgraph
========

Graph data structure library.

Please read the `API documentation here`__

__ http://bluss.github.io/petulant-avenger-graphlibrary/

|build_status|_ |crates|_

.. |build_status| image:: https://travis-ci.org/bluss/petulant-avenger-graphlibrary.svg?branch=master
.. _build_status: https://travis-ci.org/bluss/petulant-avenger-graphlibrary

.. |crates| image:: http://meritbadge.herokuapp.com/petgraph
.. _crates: https://crates.io/crates/petgraph

Recent Changes
--------------

- 0.2.0

  - New Features

    - Add Graph::neighbors().detach() to step edges without borrowing.
      This is more general than, and replaces now deprecated
      walk_edges_directed. (#39)
    - Implement Default for Graph, GraphMap
    - Add method EdgeDirection::opposite()

  - Breaking changes

    - Graph::neighbors() for undirected graphs and Graph::neighbors_undirected
      for any graph now visit self loop edges once, not twice. (#31)
    - Renamed Graph::without_edges to Graph::externals
    - GraphMap::add_edge now returns ``Option<E>``
    - Element type of ``GraphMap<N, E>::all_edges()`` changed to ``(N, N, &E)``

  - Minor breaking changes

    - IntoWeightedEdge changed a type parameter to associated type
    - IndexType is now an unsafe trait
    - Removed IndexType::{one, zero}, use method new instead.
    - Removed MinScored
    - Ptr moved to the graphmap module.
    - Directed, Undirected are now void enums.
    - Fields of graphmap::Edges are now private (#19)

- 0.1.18

  - Fix bug on calling GraphMap::add_edge with existing edge (#35)

- 0.1.17

  - Add Graph::capacity(), GraphMap::capacity()
  - Fix bug in Graph::reverse()
  - Graph and GraphMap have `quickcheck::Arbitrary` implementations,
    if optional feature `quickcheck` is enabled.

- 0.1.16

  - Add Graph::node_indices(), Graph::edge_indices()
  - Add Graph::retain_nodes(), Graph::retain_edges()
  - Add Graph::extend_with_edges(), Graph::from_edges()
  - Add functions petgraph::graph::{edge_index, node_index};
  - Add GraphMap::extend(), GraphMap::from_edges()
  - Add petgraph::dot::Dot for simple graphviz dot output

- 0.1.15

  - Add Graph::clear_edges()
  - Add Graph::edge_endpoints()
  - Add Graph::map() and Graph::filter_map()

- 0.1.14

  - Add new topological order visitor Topo
  - New graph traits NeighborsDirected, Externals, Revisitable

- 0.1.13

  - Add iterator GraphMap::all_edges

- 0.1.12

  - Fix an algorithm error in scc (#14)

- 0.1.11

  - Update for well-formedness warnings (Rust RFC 1214), adding
    new lifetime bounds on NeighborIter and Dfs, impact should be minimal.

- 0.1.10
  
  - Fix bug in WalkEdges::next_neighbor()

- 0.1.9

  - Fix Dfs/Bfs for a rustc bugfix that disallowed them
  - Add method next_neighbor() to WalkEdges

- 0.1.8

  - Add Graph::walk_edges_directed()
  - Add Graph::index_twice_mut()

- 0.1.7

  - Add Graph::edges_directed()

- 0.1.6

  - Add Graph::node_weights_mut and Graph::edge_weights_mut

- 0.1.4

  - Add back DfsIter, BfsIter

License
-------

Dual-licensed to be compatible with the Rust project.

Licensed under the Apache License, Version 2.0
http://www.apache.org/licenses/LICENSE-2.0 or the MIT license
http://opensource.org/licenses/MIT, at your
option. This file may not be copied, modified, or distributed
except according to those terms.


