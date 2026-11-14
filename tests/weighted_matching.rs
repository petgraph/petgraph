use petgraph::{
    algo::{maximum_matching, maximum_weight_matching},
    prelude::*,
    visit::EdgeRef,
};

#[test]
fn weight_and_cardinality_are_different_objectives() {
    let g = UnGraph::<(), i64>::from_edges([(0, 1, 10), (1, 2, 100), (2, 3, 10)]);
    let weight_first = maximum_weight_matching(&g, |e| *e.weight(), false);
    assert_eq!(weight_first.len(), 1);
    assert!(weight_first.contains_edge(1.into(), 2.into()));
    let cardinality_first = maximum_weight_matching(&g, |e| *e.weight(), true);
    assert_eq!(cardinality_first.len(), 2);
}

// An independent subset DP: choose whether to expose the first remaining
// vertex or pair it with each available neighbor. No blossom state is shared.
fn oracle(weights: &[Vec<Option<i64>>], cardinality_first: bool) -> (usize, i128) {
    fn visit(
        mask: usize,
        weights: &[Vec<Option<i64>>],
        cardinality_first: bool,
        memo: &mut [Option<(usize, i128)>],
    ) -> (usize, i128) {
        if mask == 0 {
            return (0, 0);
        }
        if let Some(value) = memo[mask] {
            return value;
        }
        let v = mask.trailing_zeros() as usize;
        let rest = mask ^ (1 << v);
        let mut best = visit(rest, weights, cardinality_first, memo);
        for (w, weight) in weights[v].iter().enumerate() {
            if rest & (1 << w) == 0 {
                continue;
            }
            if let Some(weight) = weight {
                let (count, value) = visit(rest ^ (1 << w), weights, cardinality_first, memo);
                let candidate = (count + 1, value + i128::from(*weight));
                if (cardinality_first && candidate > best)
                    || (!cardinality_first && candidate.1 > best.1)
                {
                    best = candidate;
                }
            }
        }
        memo[mask] = Some(best);
        best
    }
    visit(
        (1 << weights.len()) - 1,
        weights,
        cardinality_first,
        &mut vec![None; 1 << weights.len()],
    )
}

fn check(g: &UnGraph<(), i64>, cardinality_first: bool) {
    let n = g.node_count();
    let mut weights = vec![vec![None; n]; n];
    for edge in g.edge_references() {
        let (u, v) = (edge.source().index(), edge.target().index());
        if u != v {
            let weight = weights[u][v].unwrap_or(i64::MIN).max(*edge.weight());
            weights[u][v] = Some(weight);
            weights[v][u] = Some(weight);
        }
    }
    let expected = oracle(&weights, cardinality_first);
    let matching = maximum_weight_matching(g, |e| *e.weight(), cardinality_first);
    let mut actual = 0;
    let mut endpoints = vec![false; n];
    for (u, v) in matching.edges() {
        assert_ne!(u, v);
        assert!(!endpoints[u.index()] && !endpoints[v.index()]);
        endpoints[u.index()] = true;
        endpoints[v.index()] = true;
        assert_eq!(matching.mate(v), Some(u));
        actual += i128::from(weights[u.index()][v.index()].unwrap());
    }
    assert_eq!(
        actual, expected.1,
        "cardinality_first={cardinality_first}, graph={g:?}"
    );
    if cardinality_first {
        assert_eq!(matching.len(), expected.0);
        assert_eq!(matching.len(), maximum_matching(g).len());
    }
}

#[test]
fn exhaustive_topologies_with_signed_weights() {
    let n = if cfg!(miri) { 4 } else { 6 };
    for topology in 0u32..1 << (n * (n - 1) / 2) {
        let mut g = UnGraph::<(), i64>::default();
        for _ in 0..n {
            g.add_node(());
        }
        let mut bit = 0;
        for u in 0..n {
            for v in u + 1..n {
                if topology & (1 << bit) != 0 {
                    let weight = ((u * 13 + v * 7 + topology as usize) % 19) as i64 - 9;
                    g.add_edge(NodeIndex::new(u), NodeIndex::new(v), weight);
                }
                bit += 1;
            }
        }
        check(&g, false);
        check(&g, true);
    }
}

#[test]
fn seeded_graphs_against_subset_oracle() {
    let mut state = 0x1F68_593B_2D09_ABE5u64;
    let cases = if cfg!(miri) { 10 } else { 1000 };
    for case in 0usize..cases {
        let n = case % 13;
        let mut g = UnGraph::<(), i64>::default();
        for _ in 0..n {
            g.add_node(());
        }
        for u in 0..n {
            for v in u + 1..n {
                state ^= state << 13;
                state ^= state >> 7;
                state ^= state << 17;
                if state % 5 < (case % 5) as u64 {
                    let weight = match case % 5 {
                        0 => (state % 3) as i64,
                        1 => (state % 1000) as i64 - 500,
                        2 => (state % 1000) as i64 - 2000,
                        3 => (state % 1000) as i64 * 1_000_000_000_000,
                        _ => state as i64,
                    };
                    g.add_edge(NodeIndex::new(u), NodeIndex::new(v), weight);
                }
            }
        }
        check(&g, false);
        check(&g, true);
        // Relabel vertices and reverse the edge iteration order. Only the
        // selected pairs may change; the two objective values must not.
        let mut permuted = UnGraph::<(), i64>::default();
        for _ in 0..n {
            permuted.add_node(());
        }
        for edge in g.edge_references().collect::<Vec<_>>().into_iter().rev() {
            permuted.add_edge(
                NodeIndex::new(n - 1 - edge.target().index()),
                NodeIndex::new(n - 1 - edge.source().index()),
                *edge.weight(),
            );
        }
        check(&permuted, false);
        check(&permuted, true);
    }
}

#[test]
fn full_signed_range_and_parallel_edges() {
    for weight in [i64::MIN, i64::MIN + 1, -1, 0, 1, i64::MAX - 1, i64::MAX] {
        let mut g = UnGraph::<(), i64>::from_edges([
            (0, 1, weight),
            (1, 2, i64::MAX),
            (2, 0, i64::MIN),
            (2, 3, weight),
            (0, 3, i64::MAX),
            (1, 3, i64::MIN),
        ]);
        g.add_edge(0.into(), 1.into(), i64::MIN);
        g.add_edge(0.into(), 0.into(), i64::MAX);
        g.add_edge(3.into(), 3.into(), i64::MIN);
        check(&g, false);
        check(&g, true);
    }
}

#[test]
#[cfg(all(feature = "graphmap", feature = "stable_graph"))]
fn graphmap_and_sparse_stable_graph() {
    let g: UnGraphMap<&str, i64> =
        UnGraphMap::from_edges([("a", "b", 9i64), ("b", "c", 100), ("c", "d", 9)]);
    let m = maximum_weight_matching(&g, |e| *e.weight(), true);
    assert_eq!(m.len(), 2);
    assert!(m.contains_edge("a", "b") && m.contains_edge("c", "d"));

    let mut g = StableUnGraph::<(), i64>::default();
    let nodes: Vec<_> = (0..10).map(|_| g.add_node(())).collect();
    for i in [0, 2, 4, 6, 8] {
        g.remove_node(nodes[i]);
    }
    g.add_edge(nodes[1], nodes[3], 50);
    g.add_edge(nodes[3], nodes[5], 100);
    g.add_edge(nodes[5], nodes[7], 50);
    let m = maximum_weight_matching(&g, |e| *e.weight(), true);
    assert_eq!(m.len(), 2);
    assert!(m.contains_edge(nodes[1], nodes[3]));
    assert!(m.contains_edge(nodes[5], nodes[7]));
    assert_eq!(m.mate(nodes[0]), None);
    assert_eq!(m.mate(nodes[9]), None);
}

#[cfg(feature = "quickcheck")]
#[test]
fn quickcheck_weighted_matching_against_oracle() {
    fn property(edges: Vec<(u8, u8, i16)>, cardinality_first: bool) -> bool {
        let mut graph = UnGraph::<(), i64>::default();
        for _ in 0..8 {
            graph.add_node(());
        }
        for (u, v, weight) in edges.into_iter().take(60) {
            graph.add_edge(
                NodeIndex::new(usize::from(u % 8)),
                NodeIndex::new(usize::from(v % 8)),
                i64::from(weight),
            );
        }
        check(&graph, cardinality_first);
        true
    }
    quickcheck::QuickCheck::new()
        .tests(if cfg!(miri) { 10 } else { 500 })
        .quickcheck(property as fn(Vec<(u8, u8, i16)>, bool) -> bool);
}

#[test]
fn callback_evaluated_once_per_non_loop_edge_and_ties_repeat() {
    let g = UnGraph::<(), i64>::from_edges([
        (0, 0, i64::MAX),
        (0, 1, 5),
        (0, 1, 10),
        (1, 2, 10),
        (0, 2, 10),
        (2, 3, 10),
        (1, 3, 10),
    ]);
    let mut calls = 0;
    let m = maximum_weight_matching(
        &g,
        |e| {
            calls += 1;
            *e.weight()
        },
        false,
    );
    assert_eq!(calls, g.edge_count() - 1);
    let pairs = m.edges().collect::<Vec<_>>();
    for _ in 0..10 {
        assert_eq!(
            maximum_weight_matching(&g, |e| *e.weight(), false)
                .edges()
                .collect::<Vec<_>>(),
            pairs
        );
    }
}
