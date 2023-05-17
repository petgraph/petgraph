# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Breaking Changes

- Categorized all algorithms, algorithms are no longer top-level modules.
    - `petgraph::algo::astar` is now `petgraph-algorithms::shortest_path::astar`, etc.
- `Measure` now requires `TotalOrd` instead of `PartialOrd`. The previous implementation relied on quirks
  of `PartialOrd` to produce an `Ord` for floating point numbers and would not produce the desired results for
  non-numeric types. `TotalOrd` is implemented for all primitive numeric types. Users that implemented `Measure` before
  now also need to implement `TotalOrd`.
- `k_shortest_path` has been renamed to `k_shortest_paths`.
- `dominators` has been renamed to `dominance`
- `Measure` now requires stricter bounds, types must now satisfy `funty::Numeric`, doing so enables a lot more
  flexibility in future additions. This change is unlikely to affect any users.
- `FloatMeasure` now requires stricter bounds, types must now satisfy `Measure + funty::Floating`, the same
  reasoning as above applies. This change is unlikely to affect any users.
- `BoundedMeasure` now requires `Measure`. This change is unlikely to affect any users.
- `BoundedMeasure` now requires `checked_add` instead of `overflowing_add`. This change is unlikely to affect any users.

## Added

- Moved `petgraph::algo` to `petgraph-algorithms`

[unreleased]: https://github.com/olivierlacan/keep-a-changelog/compare/petgraph@v0.6.3...HEAD
