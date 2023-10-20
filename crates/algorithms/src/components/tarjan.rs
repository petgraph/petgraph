use alloc::vec::Vec;
use core::num::NonZeroUsize;

use petgraph_core::deprecated::visit::{IntoNeighbors, IntoNodeIdentifiers, NodeIndexable};

#[derive(Copy, Clone, Debug)]
struct NodeData {
    rootindex: Option<NonZeroUsize>,
}

/// A reusable state for computing the *strongly connected components* using [Tarjan's
/// algorithm][1].
///
/// [1]: https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm
#[derive(Debug)]
pub struct TarjanScc<N> {
    index: usize,
    componentcount: usize,
    nodes: Vec<NodeData>,
    stack: Vec<N>,
}

impl<N> Default for TarjanScc<N> {
    fn default() -> Self {
        Self::new()
    }
}

impl<N> TarjanScc<N> {
    /// Creates a new `TarjanScc`
    pub fn new() -> Self {
        TarjanScc {
            index: 1, // Invariant: index < componentcount at all times.
            componentcount: usize::MAX, /* Will hold if componentcount is initialized to
                       * number of nodes - 1 or higher. */
            nodes: Vec::new(),
            stack: Vec::new(),
        }
    }

    /// \[Generic\] Compute the *strongly connected components* using Algorithm 3 in
    /// [A Space-Efficient Algorithm for Finding Strongly Connected Components][1] by David J.
    /// Pierce, which is a memory-efficient variation of [Tarjan's algorithm][2].
    ///
    ///
    /// [1]: https://homepages.ecs.vuw.ac.nz/~djp/files/P05.pdf
    /// [2]: https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm
    ///
    /// Calls `f` for each strongly strongly connected component (scc).
    /// The order of node ids within each scc is arbitrary, but the order of
    /// the sccs is their postorder (reverse topological sort).
    ///
    /// For an undirected graph, the sccs are simply the connected components.
    ///
    /// This implementation is recursive and does one pass over the nodes.
    pub fn run<G, F>(&mut self, g: G, mut f: F)
    where
        G: IntoNodeIdentifiers<NodeId = N> + IntoNeighbors<NodeId = N> + NodeIndexable<NodeId = N>,
        F: FnMut(&[N]),
        N: Copy + PartialEq,
    {
        self.nodes.clear();
        self.nodes
            .resize(g.node_bound(), NodeData { rootindex: None });

        for n in g.node_identifiers() {
            let visited = self.nodes[g.to_index(n)].rootindex.is_some();
            if !visited {
                self.visit(n, g, &mut f);
            }
        }

        debug_assert!(self.stack.is_empty());
    }

    fn visit<G, F>(&mut self, v: G::NodeId, g: G, f: &mut F)
    where
        G: IntoNeighbors<NodeId = N> + NodeIndexable<NodeId = N>,
        F: FnMut(&[N]),
        N: Copy + PartialEq,
    {
        macro_rules! node {
            ($node:expr) => {
                self.nodes[g.to_index($node)]
            };
        }

        let node_v = &mut node![v];
        debug_assert!(node_v.rootindex.is_none());

        let mut v_is_local_root = true;
        let v_index = self.index;
        node_v.rootindex = NonZeroUsize::new(v_index);
        self.index += 1;

        for w in g.neighbors(v) {
            if node![w].rootindex.is_none() {
                self.visit(w, g, f);
            }
            if node![w].rootindex < node![v].rootindex {
                node![v].rootindex = node![w].rootindex;
                v_is_local_root = false
            }
        }

        if v_is_local_root {
            // Pop the stack and generate an SCC.
            let mut indexadjustment = 1;
            let c = NonZeroUsize::new(self.componentcount);
            let nodes = &mut self.nodes;
            let start = self
                .stack
                .iter()
                .rposition(|&w| {
                    if nodes[g.to_index(v)].rootindex > nodes[g.to_index(w)].rootindex {
                        true
                    } else {
                        nodes[g.to_index(w)].rootindex = c;
                        indexadjustment += 1;
                        false
                    }
                })
                .map(|x| x + 1)
                .unwrap_or_default();
            nodes[g.to_index(v)].rootindex = c;
            self.stack.push(v); // Pushing the component root to the back right before getting rid of it is somewhat ugly, but it lets it be included in f.
            f(&self.stack[start..]);
            self.stack.truncate(start);
            self.index -= indexadjustment; // Backtrack index back to where it was before we ever encountered the component.
            self.componentcount -= 1;
        } else {
            self.stack.push(v); // Stack is filled up when backtracking, unlike in Tarjans original algorithm.
        }
    }

    /// Returns the index of the component in which v has been assigned. Allows for using self as a
    /// lookup table for an scc decomposition produced by self.run().
    pub fn node_component_index<G>(&self, g: G, v: N) -> usize
    where
        G: IntoNeighbors<NodeId = N> + NodeIndexable<NodeId = N>,
        N: Copy + PartialEq,
    {
        let rindex: usize = self.nodes[g.to_index(v)]
            .rootindex
            .map(NonZeroUsize::get)
            .unwrap_or(0); // Compiles to no-op.
        debug_assert!(
            rindex != 0,
            "Tried to get the component index of an unvisited node."
        );
        debug_assert!(
            rindex > self.componentcount,
            "Given node has been visited but not yet assigned to a component."
        );

        usize::MAX - rindex
    }
}

/// \[Generic\] Compute the *strongly connected components* using [Tarjan's algorithm][1].
///
/// [1]: https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm
/// [2]: https://homepages.ecs.vuw.ac.nz/~djp/files/P05.pdf
///
/// Return a vector where each element is a strongly connected component (scc).
/// The order of node ids within each scc is arbitrary, but the order of
/// the sccs is their postorder (reverse topological sort).
///
/// For an undirected graph, the sccs are simply the connected components.
///
/// This implementation is recursive and does one pass over the nodes. It is based on
/// [A Space-Efficient Algorithm for Finding Strongly Connected Components][2] by David J. Pierce,
/// to provide a memory-efficient implementation of [Tarjan's algorithm][1].
pub fn tarjan_scc<G>(g: G) -> Vec<Vec<G::NodeId>>
where
    G: IntoNodeIdentifiers + IntoNeighbors + NodeIndexable,
{
    let mut sccs = Vec::new();
    {
        let mut tarjan_scc = TarjanScc::new();
        tarjan_scc.run(g, |scc| sccs.push(scc.to_vec()));
    }
    sccs
}

#[cfg(test)]
mod tests {
    use alloc::{vec, vec::Vec};

    use petgraph_core::{edge::Directed, visit::Reversed};
    use petgraph_graph::{Graph, NodeIndex};
    use proptest::prelude::*;

    use super::tarjan_scc;
    use crate::tests::assert_subset_topologically_sorted;

    /// Test that the algorithm works on a graph with a single component.
    ///
    /// ```text
    /// 0 → 1
    ///   ↖ ↓
    ///     2
    /// ```
    #[test]
    fn single_component() {
        let graph: Graph<(), (), Directed, _> = Graph::from_edges([
            (0u32, 1), //
            (1, 2),
            (2, 0),
        ]);

        let scc = tarjan_scc(&graph);

        assert_eq!(scc.len(), 1);

        assert_eq!(scc, [vec![
            NodeIndex::new(2),
            NodeIndex::new(1),
            NodeIndex::new(0),
        ]]);
    }

    /// Test that the algorithm works on a graph with multiple components.
    ///
    /// ```text
    /// 0 → 1   3
    ///   ↖ ↓ ↗ ↓ ↖
    ///     2   4 → 5
    /// ```
    #[test]
    fn multiple_components() {
        let graph: Graph<(), (), Directed, _> = Graph::from_edges([
            (0u32, 1), //
            (1, 2),
            (2, 0),
            (3, 4),
            (4, 5),
            (5, 3),
        ]);

        let scc = tarjan_scc(&graph);

        assert_eq!(scc.len(), 2);

        assert_eq!(scc, [
            vec![NodeIndex::new(2), NodeIndex::new(1), NodeIndex::new(0)],
            vec![NodeIndex::new(5), NodeIndex::new(4), NodeIndex::new(3)],
        ]);
    }

    /// Test that even if we reverse the graph, the algorithm still works.
    ///
    ///
    /// Uses the same graph as `single_components`.
    #[test]
    fn reversed_single_components() {
        let graph: Graph<(), (), Directed, _> = Graph::from_edges([
            (0u32, 1), //
            (1, 2),
            (2, 0),
        ]);

        let graph = Reversed(&graph);

        let scc = tarjan_scc(graph);

        assert_eq!(scc.len(), 1);

        assert_eq!(scc, [vec![
            NodeIndex::new(1),
            NodeIndex::new(2),
            NodeIndex::new(0),
        ]]);
    }

    /// Test that even if we have a disconnected graph, the algorithm still works.
    ///
    /// ```text
    /// 0 → 1   3
    ///   ↖ ↓   ↓ ↖
    ///     2   4 → 5
    /// ```
    #[test]
    fn disconnected() {
        let graph: Graph<(), (), Directed, _> = Graph::from_edges([
            (0u32, 1), //
            (1, 2),
            (2, 0),
            (3, 4),
            (4, 5),
            (5, 3),
        ]);

        let scc = tarjan_scc(&graph);

        assert_eq!(scc.len(), 2);

        assert_eq!(scc, [
            vec![NodeIndex::new(2), NodeIndex::new(1), NodeIndex::new(0)],
            vec![NodeIndex::new(5), NodeIndex::new(4), NodeIndex::new(3)],
        ]);
    }

    /// Test against the regression discovered in [issue #14].
    ///
    /// [issue #14]: https://github.com/petgraph/petgraph/issues/14
    #[test]
    fn regression_issue_14() {
        let mut graph: Graph<_, ()> = Graph::new();
        let a = graph.add_node(0);
        let b = graph.add_node(1);
        let c = graph.add_node(2);
        let d = graph.add_node(3);

        graph.extend_with_edges([(d, c), (d, b), (c, a), (b, a)]);

        let scc = tarjan_scc(&graph);

        assert_eq!(scc, [vec![a], vec![b], vec![c], vec![d]]);
    }

    /// Test against the regression discovered in [issue #60].
    ///
    /// [issue #60]: https://github.com/petgraph/petgraph/issues/60
    #[test]
    fn regression_issue_60() {
        let mut graph = Graph::<(), ()>::new();
        graph.extend_with_edges([(0, 0), (1, 0), (2, 0), (2, 1), (2, 2)]);
        graph.add_node(());

        let scc = tarjan_scc(&graph);

        // even if we extend with edges and add a node, the algorithm should still work.
        assert_eq!(scc, [
            vec![NodeIndex::new(0)],
            vec![NodeIndex::new(1)],
            vec![NodeIndex::new(2)],
            vec![NodeIndex::new(3)],
        ]);
    }

    #[cfg(not(miri))]
    proptest! {
        #[test]
        fn topologically_sorted(graph in any::<Graph<(), (), Directed, u8>>()) {
            let order = tarjan_scc(&graph);
            let firsts = order.iter().rev().map(|v| v[0]).collect::<Vec<_>>();

            assert_subset_topologically_sorted(&graph, &firsts);
        }
    }
}
