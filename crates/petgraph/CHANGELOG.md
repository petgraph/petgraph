# Changelog

All notable changes to this project will be documented in this file.


## [0.9.0](https://github.com/petgraph/petgraph/compare/petgraph@v0.8.3...petgraph@v0.9.0) - 2026-10-31

### Bug Fixes

- return zero colors for empty DSATUR graphs ([#1001](https://github.com/petgraph/petgraph/pull/1001))
- correct undirected edge count in Generator::all without self-loops ([#1000](https://github.com/petgraph/petgraph/pull/1000))

### Miscellaneous Tasks

- [**breaking**] Remove old petgraph data and re-export old petgraph as cleanup ([#1069](https://github.com/petgraph/petgraph/pull/1069))

### New Features

- [**breaking**] create new crate layout ([#923](https://github.com/petgraph/petgraph/pull/923))
- Implement `VisitContainer` for `fixedbitset` and add Directed and Undirected structs ([#1072](https://github.com/petgraph/petgraph/pull/1072))
