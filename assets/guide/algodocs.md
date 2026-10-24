# Algorithm documentation template

The following is a template for documenting the `run` method of algorithm structs.

## Template

```rust
/// Title or summary
///
/// Main description (optional)
///
/// # Arguments
/// The following arguments need to be set in the algorithm struct before calling this method:
/// 
/// # Returns
/// A struct wrapping ...
/// 
/// # Complexity
/// * Time complexity: **O(..)**.
/// * Auxiliary space: **O(..)**.
/// 
/// Where **|V|** is the number of nodes and **|E|** is the number of edges in the graph.
/// The exact complexity depends on the graph backend. That is, an algorithm with runtime O(|V|) 
/// may have effective running time O(|V|^2) if accessing the graph is O(|V|) instead of O(1).
/// 
/// # Errors (optional)
/// ...
/// 
/// # Example
/// Rust code...
```

## Template description

The documentation of the algorithm should consist of:

1. Title or summary that captures the algorithm’s purpose or name.
2. An optional text section with a detailed description, if additional explanation is needed.
3. `Arguments` - a list and explanation of the parameters that need to be set in the algorithm struct before calling the `run` method.
4. `Returns` - a description of the output produced by the algorithm which should be a struct wrapping the result of the algorithm.
5. `Complexity` - time complexity and the amount of auxiliary memory. This should be expressed in terms of the number of accesses/iterations of the graph in terms of the number of nodes **|V|** and edges **|E|**.
6. `Examples` - sample usage to illustrate the algorithm and help users get started quickly.

The description of the algorithm should also include a reference to the literature. It can be either a Wikipedia page or a link to a journal article, etc.

## Example

The [`dinics`](../../crates/algorithms/src/flows/max_flow/dinics_mod.rs) algorithm documentation is given below as an example:

```rust
/// Run [Dinic's (or Dinitz's) algorithm][dinics] to find a [Maximum Flow][maximum_flow] from
/// `source` to `destination`.
///
/// The algorithm works by building successive level graphs using breadth-first search and finds
/// blocking flows within them through depth-first searches.
///
/// Edge Data of the provided graph is interpreted as capacities of edges.
///
/// See also the [`max_flow`](../index.html) module for other maximum flow algorithms.
///
/// # Arguments
/// The following arguments need to be set in the algorithm struct before calling this method:
/// - `source`: Source node for the flow.
/// - `destination`: Sink node for the flow.
/// - `capacities` (optional): Function that returns the non-negative capacity of an edge given
///   its ID. If not set, the edge data of the graph is used as capacity.
///
/// # Returns
/// Returns a struct wrapping the maximum flow value and the flow of each edge.
///
/// # Complexity
/// - Time complexity:
///   - In general: **O(|V|²|E|)**
///   - In networks with only unit capacities: **O(min{|V|²ᐟ³, |E|¹ᐟ²} |E|)**
/// - Auxiliary space: **O(|V| + |E|)**.
///
/// Where **|V|** is the number of nodes and **|E|** is the number of edges in the graph.
/// The exact complexity depends on the graph backend. That is, an algorithm with runtime O(|V|)
/// may have effective running time O(|V|^2) if accessing the graph is O(|V|) instead of O(1).
///
/// # Errors
/// Returns an error if the source or destination node is not set.
///
/// # Example
///
///
/// [maximum_flow]: https://en.wikipedia.org/wiki/Maximum_flow_problem
/// [dinics]: https://en.wikipedia.org/wiki/Dinic%27s_algorithm
```
