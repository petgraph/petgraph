TODOs:

- Redo Docstrings for Algorithms
- Add examples in algorithm docstrings
- Open PR with adapted Lifetime names for Graph trait
- Adapt Algodocs according to changes to docstrings
- Continue Algorithm Port Instructions
- Solve Storage/IndexId problem

# Checklist

Algorithm

- [ ] Move algorithm to petgraph-algorithms crate into correct directory and module
- [ ] Use new traits for algorithm
- [ ] Use structs for algorithm
- [ ] Use struct for algorithm output

Docs

- [ ] Write algorithm documentation according to the ALGODOCS (todo)
- [ ] Write Module level documentation

Tests

- [ ] Move tests from petgraph to petgraph-algorithms crate
- [ ] Adapt Quickcheck tests

Benchmarks

- [ ] Adapt benchmarks to new traits

Pre-PR Checks

- [ ] `just ci` passes
