Version 0.5.1 (2020-05-23)
==========================

- Implement ``Default`` for traversals.
- Export ``EdgesConnecting`` publicly.
- Implement ``is_bipartite_graph``.
- Add ``FilterNode`` implementation for ``FixedBitSet`` and ``HashSet``.
- Implement ``node_weights_mut`` and ``edge_weights_mut`` for ``StableGraph``.
- Add configurable functions for adding attributes to dotfile features.

Version 0.5.0 (2019-12-25)
==========================

Breaking changes
----------------

- The iterative DFS implementation, ``Dfs``, now marks nodes visited when
  they are pushed onto the stack, not when they're popped off. This may
  require changes to callers that use ``Dfs::from_parts`` or manipulate
  its internals.
- The ``IntoEdgesDirected`` trait now has a stricter contract for
  undirected graphs. Custom implementations of this trait may have to be
  updated. See the `trait documentation`__ for more.

Other changes
-------------

- Upgrade to Rust 2018 edition
- Fix clippy warnings and unify code formatting
- Improved and enhanced documentation
- Update dependencies including modern quickcheck
- Numerous bugfixes and refactorings
- Added ``MatrixGraph`` implementation

__ https://docs.rs/petgraph/0.5/petgraph/visit/trait.IntoEdgesDirected.html

Version 0.4.13 (2018-08-26)
===========================

- Fix clippy warnings by @jonasbb
- Add docs for ``Csr`` by @ksadorf
- Fix conflict with new stable method ``find_map`` in new Rust

Version 0.4.12 (2018-03-26)
===========================

- Newtype ``Time`` now also implements ``Hash``
- Documentation updates for ``Frozen``.

Version 0.4.11 (2018-01-07)
===========================

- Fix ``petgraph::graph::NodeReferences`` to be publicly visible
- Small doc typo and code style files by @shepmaster and @waywardmonkeys
- Fix a future compat warning with pointer casts

Version 0.4.10 (2017-08-15)
===========================

- Add graph trait ``IntoEdgesDirected``
- Update dependencies

Version 0.4.9 (2017-10-02)
==========================

- Fix ``bellman_ford`` to work correctly with undirected graphs (#152) by
  @carrutstick
- Performance improvements for ``Graph, Stablegraph``'s ``.map()``.

Version 0.4.8 (2017-09-20)
==========================

- ``StableGraph`` learned new methods nearing parity with ``Graph``.  Note
  that the ``StableGraph`` methods preserve index stability even in the batch
  removal methods like ``filter_map`` and ``retain_edges``.

  + Added ``.filter_map()``, which maps associated node and edge data
  + Added ``.retain_edges()``, ``.edge_indices()`` and ``.clear_edges()``

- Existing ``Graph`` iterators gained some trait impls:

  + ``.node_indices(), .edge_indices()`` are ``ExactSizeIterator``
  + ``.node_references()`` is now
    ``DoubleEndedIterator + ExactSizeIterator``.
  + ``.edge_references()`` is now ``ExactSizeIterator``.

- Implemented ``From<StableGraph>`` for ``Graph``.

Version 0.4.7 (2017-09-16)
==========================

- New algorithm by @jmcomets: A* search algorithm in ``petgraph::algo::astar``
- One ``StableGraph`` bug fix whose patch was supposed to be in the previous
  version:

  + ``add_edge(m, n, _)`` now properly always panics if nodes m or n don't
    exist in the graph.

Version 0.4.6 (2017-09-12)
==========================

- New optional crate feature: ``"serde-1"``, which enables serialization
  for ``Graph`` and ``StableGraph`` using serde.
- Add methods ``new``, ``add_node`` to ``Csr`` by @jmcomets
- Add indexing with ``[]`` by node index, ``NodeCompactIndexable`` for
  ``Csr`` by @jmcomets
- Amend doc for ``GraphMap::into_graph`` (it has a case where it can panic)
- Add implementation of ``From<Graph>`` for ``StableGraph``.
- Add implementation of ``IntoNodeReferences`` for ``&StableGraph``.
- Add method ``StableGraph::map`` that maps associated data
- Add method ``StableGraph::find_edge_undirected``
- Many ``StableGraph`` bug fixes involving node vacancies (holes left by
  deletions):

  + ``neighbors(n)`` and similar neighbor and edge iterator methods now
    handle n being a vacancy properly. (This produces an empty iterator.)
  + ``find_edge(m, n)`` now handles m being a vacancy correctly too
  + ``StableGraph::node_bound`` was fixed for empty graphs and returns 0

- Add implementation of ``DoubleEndedIterator`` to ``Graph, StableGraph``'s
  edge references iterators.
- Debug output for ``Graph`` now shows node and edge count. ``Graph, StableGraph``
  show nothing for the edges list if it's empty (no label).
- ``Arbitrary`` implementation for ``StableGraph`` now can produce graphs with
  vacancies (used by quickcheck)

Version 0.4.5 (2017-06-16)
==========================

- Fix ``max`` ambiguity error with current rust nightly by @daboross (#153)

Version 0.4.4 (2017-03-14)
==========================

- Add ``GraphMap::all_edges_mut()`` iterator by @Binero
- Add ``StableGraph::retain_nodes`` by @Rupsbant
- Add ``StableGraph::index_twice_mut`` by @christolliday

Version 0.4.3 (2017-01-21)
==========================

- Add crate categories

Version 0.4.2 (2017-01-06)
==========================

- Move the ``visit.rs`` file due to changed rules for a module’s directory
  ownership in Rust, resolving a future compat warning.
- The error types ``Cycle, NegativeCycle`` now implement ``PartialEq``.

Version 0.4.1 (2016-10-26)
==========================

- Add new algorithm ``simple_fast`` for computing dominators in a control-flow
  graph.

Version 0.4.0 (2016-10-17)
==========================

Breaking changes in ``Graph``
-----------------------------

- ``Graph::edges`` and the other edges methods now return an iterator of
  edge references

Other breaking changes
----------------------

- ``toposort`` now returns an error if the graph had a cycle.
- ``is_cyclic_directed`` no longer takes a dfs space argument. It is
  now recursive.
- ``scc`` was renamed to ``kosaraju_scc``.
- ``min_spanning_tree`` now returns an iterator that needs to be
  made into a specific graph type deliberately.
- ``dijkstra`` now uses the ``IntoEdges`` trait.
- ``NodeIndexable`` changed its method signatures.
- ``IntoExternals`` was removed, and many other smaller adjustments
  in graph traits. ``NodeId`` must now implement ``PartialEq``, for example.
- ``DfsIter, BfsIter`` were removed in favour of a more general approach
  with the ``Walker`` trait and its iterator conversion.

New features
------------

- New graph traits, for example ``IntoEdges`` which returns
  an iterator of edge references. Everything implements the graph traits
  much more consistently.
- Traits for associated data access and building graphs: ``DataMap``,
  ``Build, Create, FromElements``.
- Graph adaptors: ``EdgeFiltered``. ``Filtered`` was renamed to ``NodeFiltered``.
- New algorithms: bellman-ford
- New graph: compressed sparse row (``Csr``).
- ``GraphMap`` implements ``NodeIndexable``.
- ``Dot`` was generalized

Version 0.3.2 (2016-10-11)
==========================

  - Add ``depth_first_search``, a recursive dfs visitor that emits discovery,
    finishing and edge classification events.
  - Add graph adaptor ``Filtered``.
  - impl ``Debug, NodeIndexable`` for ``Reversed``.

Version 0.3.1 (2016-10-05)
==========================

- Add ``.edges(), .edges_directed()`` to ``StableGraph``. Note that these
  differ from ``Graph``, because this is the signature they will all use
  in the future.
- Add ``.update_edge()`` to ``StableGraph``.
- Add reexports of common items in ``stable_graph`` module (for example
  ``NodeIndex``).
- Minor performance improvements to graph iteration
- Improved docs for ``visit`` module.

Version 0.3.0 (2016-10-03)
==========================

- Overhaul all graph visitor traits so that they use the ``IntoIterator``
  style. This makes them composable.

  - Multiple graph algorithms use new visitor traits.
  - **Help is welcome to port more algorithms (and create new graph traits in
    the process)!**

- ``GraphMap`` can now have directed edges. ``GraphMap::new`` is now generic
  in the edge type. ``DiGraphMap`` and ``UnGraphMap`` are new type aliases.
- Add type aliases ``DiGraph, UnGraph, StableDiGraph, StableUnGraph``
- ``GraphMap`` is based on the indexmap crate. Deterministic iteration
  order, faster iteration, no side tables needed to convert to ``Graph``.
- Improved docs for a lot of types and functions.
- Add graph visitor ``DfsPostOrder``
- ``Dfs`` gained new methods ``from_parts`` and ``reset``.
- New algo ``has_path_connecting``.
- New algo ``tarjan_scc``, a second scc implementation.
- Document traversal order in ``Dfs, DfsPostOrder, scc, tarjan_scc``.
- Optional graph visitor workspace reuse in ``has_path_connecting``,
  ``is_cyclic_directed, toposort``.
- Improved ``Debug`` formatting for ``Graph, StableGraph``.
- Add a prelude module
- ``GraphMap`` now has a method ``.into_graph()`` that makes a ``Graph``.
- ``Graph::retain_nodes, retain_edges`` now expose the self graph only
  as wrapped in ``Frozen``, so that weights can be mutated but the
  graph structure not.
- Enable ``StableGraph`` by default
- Add method ``Graph::contains_edge``.
- Renamed ``EdgeDirection`` → ``Direction``.
- Remove ``SubTopo``.
- Require Rust 1.12 or later

Version 0.2.10 (2016-07-27)
===========================

- Fix compilation with rust nightly

Version 0.2.9 (2016-10-01)
==========================

- Fix a bug in SubTopo (#81)

Version 0.2.8 (2016-09-12)
==========================

- Add Graph methods reserve_nodes, reserve_edges, reserve_exact_nodes,
  reserve_exact_edges, shrink_to_fit_edges, shrink_to_fit_nodes, shrink_to_fit

Version 0.2.7 (2016-04-22)
==========================

- Update URLs

Version 0.2.6 (2016-04-20)
==========================

- Fix warning about type parameter defaults (no functional change)

Version 0.2.5 (2016-04-10)
==========================

- Add SubTopo, a topo walker for the subgraph reachable from a starting point.
- Add condensation, which forms the graph of a graph’s strongly connected
  components.

Version 0.2.4 (2016-04-05)
==========================

- Fix an algorithm error in scc (#61). This time we have a test that
  crosschecks the result of the algorithm vs another implementation, for
  greater confidence in its correctness.

Version 0.2.3 (2016-02-22)
==========================

- Require Rust 1.6: Due to changes in how rust uses type parameter defaults.
- Implement Graph::clone_from.

Version 0.2.2 (2015-12-14)
==========================

- Require Rust 1.5
- ``Dot`` passes on the alternate flag to node and edge label formatting
- Add ``Clone`` impl for some iterators
- Document edge iteration order for ``Graph::neighbors``
- Add *experimental feature* ``StableGraph``, using feature flag ``stable_graph``

Version 0.2.1 (2015-12-06)
==========================

- Add algorithm ``is_isomorphic_matching``

Version 0.2.0 (2015-12-03)
==========================

New Features
------------

- Add Graph::neighbors().detach() to step edges without borrowing.
  This is more general than, and replaces now deprecated
  walk_edges_directed. (#39)
- Implement Default for Graph, GraphMap
- Add method EdgeDirection::opposite()

Breaking changes
----------------

- Graph::neighbors() for undirected graphs and Graph::neighbors_undirected
  for any graph now visit self loop edges once, not twice. (#31)
- Renamed Graph::without_edges to Graph::externals
- Removed Graph::edges_both
- GraphMap::add_edge now returns ``Option<E>``
- Element type of ``GraphMap<N, E>::all_edges()`` changed to ``(N, N, &E)``

Minor breaking changes
----------------------

- IntoWeightedEdge changed a type parameter to associated type
- IndexType is now an unsafe trait
- Removed IndexType::{one, zero}, use method new instead.
- Removed MinScored
- Ptr moved to the graphmap module.
- Directed, Undirected are now void enums.
- Fields of graphmap::Edges are now private (#19)

Version 0.1.18 (2015-11-30)
===========================

- Fix bug on calling GraphMap::add_edge with existing edge (#35)

Version 0.1.17 (2015-11-25)
===========================

- Add Graph::capacity(), GraphMap::capacity()
- Fix bug in Graph::reverse()
- Graph and GraphMap have `quickcheck::Arbitrary` implementations,
  if optional feature `check` is enabled.

Version 0.1.16 (2015-11-25)
===========================

- Add Graph::node_indices(), Graph::edge_indices()
- Add Graph::retain_nodes(), Graph::retain_edges()
- Add Graph::extend_with_edges(), Graph::from_edges()
- Add functions petgraph::graph::{edge_index, node_index};
- Add GraphMap::extend(), GraphMap::from_edges()
- Add petgraph::dot::Dot for simple graphviz dot output

Version 0.1.15 (2015-11-20)
===========================

- Add Graph::clear_edges()
- Add Graph::edge_endpoints()
- Add Graph::map() and Graph::filter_map()

Version 0.1.14 (2015-11-19)
===========================

- Add new topological order visitor Topo
- New graph traits NeighborsDirected, Externals, Revisitable

Version 0.1.13 (2015-11-11)
===========================

- Add iterator GraphMap::all_edges

Version 0.1.12 (2015-11-07)
===========================

- Fix an algorithm error in scc (#14)

Version 0.1.11 (2015-08-16)
===========================

- Update for well-formedness warnings (Rust RFC 1214), adding
  new lifetime bounds on NeighborIter and Dfs, impact should be minimal.

Version 0.1.10 (2015-06-22)
===========================
  
- Fix bug in WalkEdges::next_neighbor()

Version 0.1.9 (2015-06-17)
==========================

- Fix Dfs/Bfs for a rustc bugfix that disallowed them
- Add method next_neighbor() to WalkEdges

Version 0.1.8 (2015-06-08)
==========================

- Add Graph::walk_edges_directed()
- Add Graph::index_twice_mut()

Version 0.1.7 (2015-06-08)
==========================

- Add Graph::edges_directed()

Version 0.1.6 (2015-06-04)
==========================

- Add Graph::node_weights_mut and Graph::edge_weights_mut

Version 0.1.4 (2015-05-20)
==========================

- Add back DfsIter, BfsIter