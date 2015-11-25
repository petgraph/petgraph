#![cfg(feature="quickcheck")]
extern crate quickcheck;

extern crate petgraph;

use petgraph::{Graph, Undirected, Directed, EdgeType};
use petgraph::algo::{min_spanning_tree, is_cyclic_undirected, is_isomorphic};

fn prop(g: Graph<(), u32>) -> bool {
    // filter out isolated nodes
    let no_singles = g.filter_map(
        |nx, w| g.neighbors_undirected(nx).next().map(|_| w),
        |_, w| Some(w));
    for i in no_singles.node_indices() {
        assert!(no_singles.neighbors_undirected(i).count() > 0);
    }
    assert_eq!(no_singles.edge_count(), g.edge_count());
    let mst = min_spanning_tree(&no_singles);
    assert!(!is_cyclic_undirected(&mst));
    true
}

fn prop_undir(g: Graph<(), u32, Undirected>) -> bool {
    // filter out isolated nodes
    let no_singles = g.filter_map(
        |nx, w| g.neighbors_undirected(nx).next().map(|_| w),
        |_, w| Some(w));
    for i in no_singles.node_indices() {
        assert!(no_singles.neighbors_undirected(i).count() > 0);
    }
    assert_eq!(no_singles.edge_count(), g.edge_count());
    let mst = min_spanning_tree(&no_singles);
    assert!(!is_cyclic_undirected(&mst));
    true
}

#[test]
fn arbitrary() {
    quickcheck::quickcheck(prop as fn(_) -> bool);
    quickcheck::quickcheck(prop_undir as fn(_) -> bool);
}

#[test]
fn retain_nodes() {
    fn prop<Ty: EdgeType>(mut g: Graph<i32, (), Ty>) -> bool {
        // Remove all negative nodes, these should be randomly spread
        let og = g.clone();
        let nodes = g.node_count();
        let num_negs = g.raw_nodes().iter().filter(|n| n.weight < 0).count();
        let mut removed = 0;
        g.retain_nodes(|g, i| {
            let keep = g[i] >= 0;
            if !keep {
                removed += 1;
            }
            keep
        });
        let num_negs_post = g.raw_nodes().iter().filter(|n| n.weight < 0).count();
        let num_pos_post = g.raw_nodes().iter().filter(|n| n.weight >= 0).count();
        assert_eq!(num_negs_post, 0);
        assert_eq!(removed, num_negs);
        assert_eq!(num_negs + g.node_count(), nodes);
        assert_eq!(num_pos_post, g.node_count());
        if og.edge_count() < 30 {
            // check against filter_map
            let filtered = og.filter_map(|_, w| if *w >= 0 { Some(*w) } else { None },
                                         |_, w| Some(*w));
            assert_eq!(g.node_count(), filtered.node_count());
            assert!(is_isomorphic(&filtered, &g));
        }
        true
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Directed>) -> bool);
    quickcheck::quickcheck(prop as fn(Graph<_, _, Undirected>) -> bool);
}

#[test]
fn retain_edges() {
    fn prop<Ty: EdgeType>(mut g: Graph<(), i32, Ty>) -> bool {
        // Remove all negative edges, these should be randomly spread
        let og = g.clone();
        let edges = g.edge_count();
        let num_negs = g.raw_edges().iter().filter(|n| n.weight < 0).count();
        let mut removed = 0;
        g.retain_edges(|g, i| {
            let keep = g[i] >= 0;
            if !keep {
                removed += 1;
            }
            keep
        });
        let num_negs_post = g.raw_edges().iter().filter(|n| n.weight < 0).count();
        let num_pos_post = g.raw_edges().iter().filter(|n| n.weight >= 0).count();
        assert_eq!(num_negs_post, 0);
        assert_eq!(removed, num_negs);
        assert_eq!(num_negs + g.edge_count(), edges);
        assert_eq!(num_pos_post, g.edge_count());
        if og.edge_count() < 30 {
            // check against filter_map
            let filtered = og.filter_map(
                |_, w| Some(*w),
                |_, w| if *w >= 0 { Some(*w) } else { None });
            assert_eq!(g.node_count(), filtered.node_count());
            assert!(is_isomorphic(&filtered, &g));
        }
        true
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Directed>) -> bool);
    quickcheck::quickcheck(prop as fn(Graph<_, _, Undirected>) -> bool);
}
