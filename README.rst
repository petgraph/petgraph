
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


