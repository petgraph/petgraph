#![cfg(feature="quickcheck")]
extern crate quickcheck;

extern crate petgraph;

use petgraph::{Graph, GraphMap, Undirected, Directed, EdgeType, Incoming, Outgoing};
use petgraph::algo::{min_spanning_tree, is_cyclic_undirected, is_isomorphic};
use petgraph::graph::IndexType;

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
fn reverse_undirected() {
    fn prop<Ty: EdgeType>(g: Graph<(), (), Ty>) -> bool {
        if g.edge_count() > 30 {
            return true; // iso too slow
        }
        let mut h = g.clone();
        h.reverse();
        is_isomorphic(&g, &h)
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Undirected>) -> bool);
}

fn assert_graph_consistent<N, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>)
    where Ty: EdgeType,
          Ix: IndexType,
{
    assert_eq!(g.node_count(), g.node_indices().count());
    assert_eq!(g.edge_count(), g.edge_indices().count());
    for edge in g.raw_edges() {
        assert!(g.find_edge(edge.source(), edge.target()).is_some(),
                "Edge not in graph! {:?} to {:?}", edge.source(), edge.target());
    }
}

#[test]
fn reverse_directed() {
    fn prop<Ty: EdgeType>(mut g: Graph<(), (), Ty>) -> bool {
        let node_outdegrees = g.node_indices()
                                .map(|i| g.neighbors_directed(i, Outgoing).count())
                                .collect::<Vec<_>>();
        let node_indegrees = g.node_indices()
                                .map(|i| g.neighbors_directed(i, Incoming).count())
                                .collect::<Vec<_>>();

        g.reverse();
        let new_outdegrees = g.node_indices()
                                .map(|i| g.neighbors_directed(i, Outgoing).count())
                                .collect::<Vec<_>>();
        let new_indegrees = g.node_indices()
                                .map(|i| g.neighbors_directed(i, Incoming).count())
                                .collect::<Vec<_>>();
        assert_eq!(node_outdegrees, new_indegrees);
        assert_eq!(node_indegrees, new_outdegrees);
        assert_graph_consistent(&g);
        true
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Directed>) -> bool);
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

#[test]
fn qc_graphmap() {
    fn prop(g: GraphMap<i32, ()>) -> bool {
        true
    }
    quickcheck::quickcheck(prop as fn(_) -> bool);
}
