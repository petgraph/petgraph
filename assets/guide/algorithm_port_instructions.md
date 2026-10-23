# Algorithm Porting Instructions

These instructions are supposed to help port algorithms from the `petgraph` crate to the `petgraph-algorithms` crate.
In particular, the algorithms in the [crates/algorithms/src/flows/max_flow](../../crates/algorithms/src/flows/max_flow) module have been ported to the new traits and serve as a reference for further algorithms.

## Checklist

- [ ] Move algorithm to petgraph-algorithms crate into correct directory and module

- [ ] Use new traits for algorithm
  - [ ] Write your algorithm without using `alloc` or `std` features. Allocations should be done through
    the `StorableGraph` trait implicitly.
  - [ ] Use `IndexId` only if strictly necessary.
  - [ ] Refer to the trait translation table (TODO) to see which new traits replace the old ones.

- [ ] Use structs and builder pattern for algorithm
  - [ ] Missing required parameters return an error instead of panicking

- [ ] Use struct for algorithm output

- [ ] If a problem has multiple algorithms, create a default algorithm wrapper which internally calls one of the algorithms. Ideally, the default algorithm called is the one with roughly the best running time.

- [ ] Check whether there are open issues or PRs for the algorithm or problem type in the petgraph repository. If possible address them or possibly close them if they are no longer relevant.

### Documentation

- [ ] Write algorithm documentation according to the [algodocs template](../../assets/guide/algodocs.md)
- [ ] Write Module level documentation explaining the problem and the algorithms implemented in the module. Include references to literature and which algorithms to chose in which situation.

### Tests

- [ ] Move tests from petgraph to [crates/algorithms/tests](../../crates/algorithms/tests)
- [ ] Write a macro to run the tests on multiple graph implementations.
- [ ] Todo: Port quickcheck tests (for now, this is not possible because we have not yet setup quickcheck and/or proptest)

### Benchmarks

- [ ] Convert benchmarks to criterion benchmarks and move them to [crates/algorithms/benches](../../crates/algorithms/benches)
- [ ] Adapt benchmarks to new traits
- [ ] Reuse the graph helpers
- [ ] Ideally replicate the instance 1:1 of the original benchmark to compare the performance of the new implementation with the old one.

### Pre-PR Checks

- [ ] `just ci` passes
