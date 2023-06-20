# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## Breaking Changes

- The safety contract of `IndexType` has been changed. It is now required that `IndexType::from_usize`
  and `IndexType::to_usize` are inverses of each other and that no wrapping must take place.
  This is a breaking change for any user-defined `IndexType` implementations.
- The underlying `IndexType` must now implement `Unsigned` and `AtMostUsize` from the `funty` crate.
  This is a breaking change for any user-defined `IndexType` implementations.

## Changed

- The implementation of `VisitMap` for `FixedBitSet` has been relaxed. Instead of restricting to `IndexType` it now only
  requires `SafeCast<usize>`.

## Added

- Moved `petgraph::visit` to `petgraph-core`
- Moved `petgraph::EdgeType` to `petgraph-core`
- Moved `petgraph::Direction` to `petgraph-core`
- Moved `petgraph::Incoming` to `petgraph-core`
- Moved `petgraph::Outgoing` to `petgraph-core`
- Moved `petgraph::EdgeDirection` to `petgraph-core`
- Moved `petgraph::graph::IndexType` to `petgraph-core`
- Moved `petgraph::IntoWeightedEdge` to `petgraph-core`
- Moved `petgraph::macros` to `petgraph-core`
- `IndexMap` now implements `VisitMap`
- New transitional trait `SafeCast` for casting between `usize` and `IndexType`
- New transitional trait `FromIndexType` for casting from `IndexType` to `NodeIndex`/`EdgeIndex`
    - This trait is not covered by `From`, because it allows for generic functions that switch between
      different `NodeIndex` types easily, without needing to specify the `IndexType` explicitly.
- New transitional trait `IntoIndexType` for casting from `NodeIndex`/`EdgeIndex` to `IndexType`
    - This trait is not covered by `Into`, because it allows for generic functions that switch between
      different `NodeIndex` types easily, without needing to specify the `IndexType` explicitly.

## Deprecations

- `petgraph-core::visit` is deprecated in favor of a set of traits in `petgraph-core`
- `IndexType::index` is deprecated in favor of `IndexType::to_usize`
- `IndexType::new` is deprecated in favor of `IndexType::from_usize`

[unreleased]: https://github.com/olivierlacan/keep-a-changelog/compare/petgraph@v0.6.3...HEAD
