#![cfg(feature="quickcheck")]
extern crate quickcheck;
extern crate rand;
extern crate petgraph;

use rand::Rng;

use petgraph::{Graph, GraphMap, Undirected, Directed, EdgeType, Incoming, Outgoing};
use petgraph::algo::{
    min_spanning_tree,
    is_cyclic_undirected,
    is_isomorphic,
    is_isomorphic_matching,
};
use petgraph::graph::{IndexType, node_index, edge_index};
#[cfg(feature = "stable_graph")]
use petgraph::graph::stable::StableGraph;

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
    fn prop<Ty: EdgeType>(mut g: Graph<i32, i32, Ty>) -> bool {
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

        // check against filter_map
        let filtered = og.filter_map(|_, w| if *w >= 0 { Some(*w) } else { None },
                                     |_, w| Some(*w));
        assert_eq!(g.node_count(), filtered.node_count());
        /*
        println!("Iso of graph with nodes={}, edges={}",
                 g.node_count(), g.edge_count());
                 */
        assert!(is_isomorphic_matching(&filtered, &g, PartialEq::eq, PartialEq::eq));

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
fn isomorphism_1() {
    // using small weights so that duplicates are likely
    fn prop<Ty: EdgeType>(g: Graph<i8, i8, Ty>) -> bool {
        let mut rng = rand::thread_rng();
        // several trials of different isomorphisms of the same graph
        // mapping of node indices
        let mut map = g.node_indices().collect::<Vec<_>>();
        let mut ng = Graph::<_, _, Ty>::with_capacity(g.node_count(), g.edge_count());
        for _ in 0..1 {
            rng.shuffle(&mut map);
            ng.clear();

            for _ in g.node_indices() {
                ng.add_node(0);
            }
            // Assign node weights
            for i in g.node_indices() {
                ng[map[i.index()]] = g[i];
            }
            // Add edges
            for i in g.edge_indices() {
                let (s, t) = g.edge_endpoints(i).unwrap();
                ng.add_edge(map[s.index()],
                            map[t.index()],
                            g[i]);
            }
            if g.node_count() < 20 && g.edge_count() < 50 {
                assert!(is_isomorphic(&g, &ng));
            }
            assert!(is_isomorphic_matching(&g, &ng, PartialEq::eq, PartialEq::eq));
        }
        true
    }
    quickcheck::quickcheck(prop::<Undirected> as fn(_) -> bool);
    quickcheck::quickcheck(prop::<Directed> as fn(_) -> bool);
}

#[test]
fn isomorphism_modify() {
    // using small weights so that duplicates are likely
    fn prop<Ty: EdgeType>(g: Graph<i16, i8, Ty>, node: u8, edge: u8) -> bool {
        let mut ng = g.clone();
        let i = node_index(node as usize);
        let j = edge_index(edge as usize);
        if i.index() < g.node_count() {
            ng[i] = (g[i] == 0) as i16;
        }
        if j.index() < g.edge_count() {
            ng[j] = (g[j] == 0) as i8;
        }
        if i.index() < g.node_count() || j.index() < g.edge_count() {
            assert!(!is_isomorphic_matching(&g, &ng, PartialEq::eq, PartialEq::eq));
        } else {
            assert!(is_isomorphic_matching(&g, &ng, PartialEq::eq, PartialEq::eq));
        }
        true
    }
    quickcheck::quickcheck(prop::<Undirected> as fn(_, _, _) -> bool);
    quickcheck::quickcheck(prop::<Directed> as fn(_, _, _) -> bool);
}

#[test]
fn graph_remove_edge() {
    fn prop<Ty: EdgeType>(mut g: Graph<(), (), Ty>, a: u8, b: u8) -> bool {
        let a = node_index(a as usize);
        let b = node_index(b as usize);
        let edge = g.find_edge(a, b);
        if !g.is_directed() {
            assert_eq!(edge.is_some(), g.find_edge(b, a).is_some());
        }
        if let Some(ex) = edge {
            assert!(g.remove_edge(ex).is_some());
        }
        assert_graph_consistent(&g);
        assert!(g.find_edge(a, b).is_none());
        assert!(g.neighbors(a).find(|x| *x == b).is_none());
        if !g.is_directed() {
            assert!(g.neighbors(b).find(|x| *x == a).is_none());
        }
        true
    }
    quickcheck::quickcheck(prop as fn(Graph<_, _, Undirected>, _, _) -> bool);
    quickcheck::quickcheck(prop as fn(Graph<_, _, Directed>, _, _) -> bool);
}

#[cfg(feature = "stable_graph")]
#[test]
fn stable_graph_remove_edge() {
    fn prop<Ty: EdgeType>(mut g: StableGraph<(), (), Ty>, a: u8, b: u8) -> bool {
        let a = node_index(a as usize);
        let b = node_index(b as usize);
        let edge = g.find_edge(a, b);
        if !g.is_directed() {
            assert_eq!(edge.is_some(), g.find_edge(b, a).is_some());
        }
        if let Some(ex) = edge {
            assert!(g.remove_edge(ex).is_some());
        }
        //assert_graph_consistent(&g);
        assert!(g.find_edge(a, b).is_none());
        assert!(g.neighbors(a).find(|x| *x == b).is_none());
        if !g.is_directed() {
            assert!(g.find_edge(b, a).is_none());
            assert!(g.neighbors(b).find(|x| *x == a).is_none());
        }
        true
    }
    quickcheck::quickcheck(prop as fn(StableGraph<_, _, Undirected>, _, _) -> bool);
    quickcheck::quickcheck(prop as fn(StableGraph<_, _, Directed>, _, _) -> bool);
}

#[cfg(feature = "stable_graph")]
#[test]
fn stable_graph_add_remove_edges() {
    fn prop<Ty: EdgeType>(mut g: StableGraph<(), (), Ty>, edges: Vec<(u8, u8)>) -> bool {
        for &(a, b) in &edges {
            let a = node_index(a as usize);
            let b = node_index(b as usize);
            let edge = g.find_edge(a, b);

            if edge.is_none() && g.contains_node(a) && g.contains_node(b) {
                let _index = g.add_edge(a, b, ());
                continue;
            }

            if !g.is_directed() {
                assert_eq!(edge.is_some(), g.find_edge(b, a).is_some());
            }
            if let Some(ex) = edge {
                assert!(g.remove_edge(ex).is_some());
            }
            //assert_graph_consistent(&g);
            assert!(g.find_edge(a, b).is_none(), "failed to remove edge {:?} from graph {:?}", (a, b), g);
            assert!(g.neighbors(a).find(|x| *x == b).is_none());
            if !g.is_directed() {
                assert!(g.find_edge(b, a).is_none());
                assert!(g.neighbors(b).find(|x| *x == a).is_none());
            }
        }
        true
    }
    quickcheck::quickcheck(prop as fn(StableGraph<_, _, Undirected>, _) -> bool);
    quickcheck::quickcheck(prop as fn(StableGraph<_, _, Directed>, _) -> bool);
}

#[test]
fn graphmap_remove() {
    fn prop(mut g: GraphMap<i8, ()>, a: i8, b: i8) -> bool {
        let contains = g.contains_edge(a, b);
        assert_eq!(contains, g.contains_edge(b, a));
        assert_eq!(g.remove_edge(a, b).is_some(), contains);
        assert!(!g.contains_edge(a, b) &&
            g.neighbors(a).find(|x| *x == b).is_none() &&
            g.neighbors(b).find(|x| *x == a).is_none());
        assert!(g.remove_edge(a, b).is_none());
        true
    }
    quickcheck::quickcheck(prop as fn(_, _, _) -> bool);
}

#[test]
fn graphmap_add_remove() {
    fn prop(mut g: GraphMap<i8, ()>, a: i8, b: i8) -> bool {
        assert_eq!(g.contains_edge(a, b), g.add_edge(a, b, ()).is_some());
        g.remove_edge(a, b);
        !g.contains_edge(a, b) &&
            g.neighbors(a).find(|x| *x == b).is_none() &&
            g.neighbors(b).find(|x| *x == a).is_none()
    }
    quickcheck::quickcheck(prop as fn(_, _, _) -> bool);
}
