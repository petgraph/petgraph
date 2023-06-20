# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Breaking Changes

* `NodeIndex` no longer implements `IndexType`. This technically allowed implementations to have types
  like `NodeIndex<NodeIndex<u32>>`, which is not intended. This change is not expected to affect any users.

## Added

- Moved `petgraph::graph` to `petgraph-graph`
- Moved `petgraph::stable_graph` to `petgraph-graph`

[unreleased]: https://github.com/olivierlacan/keep-a-changelog/compare/petgraph@v0.6.3...HEAD
