# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Breaking Changes

- Categorized all algorithms, algorithms are no longer top-level modules.
    - `petgraph::algo::astar` is now `petgraph-algorithms::shortest_path::astar`
- `Measure` now requires `TotalOrd` instead of `PartialOrd`. The previous implementation was incorrect and could
  unintended results. `TotalOrd` is implemented for `Ord`, `f32` and `f64`.
- `k_shortest_path` has been renamed to `k_shortest_paths`.

## Added

- Moved `petgraph::algo` to `petgraph-algorithms`

[unreleased]: https://github.com/olivierlacan/keep-a-changelog/compare/petgraph@v0.6.3...HEAD
