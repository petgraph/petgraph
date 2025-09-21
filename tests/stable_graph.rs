#![cfg(feature = "stable_graph")]

extern crate itertools;
extern crate petgraph;
#[macro_use]
extern crate defmac;

use std::collections::HashSet;

use itertools::assert_equal;
use petgraph::adj::IndexType;
use petgraph::algo::{kosaraju_scc, min_spanning_tree, tarjan_scc};
use petgraph::dot::Dot;
use petgraph::prelude::*;
use petgraph::stable_graph::edge_index as e;
use petgraph::stable_graph::node_index as n;
use petgraph::visit::{EdgeIndexable, IntoEdgeReferences, IntoNodeReferences, NodeIndexable};
use petgraph::EdgeType;

fn assert_graph_consistent<N, E, Ty, Ix>(g: &StableGraph<N, E, Ty, Ix>)
where
    Ty: EdgeType,
    Ix: IndexType,
{
    assert_eq!(g.node_count(), g.node_indices().count());
    assert_eq!(g.edge_count(), g.edge_indices().count());
    for edge in g.edge_references() {
        assert!(
            g.find_edge(edge.source(), edge.target()).is_some(),
            "Edge not in graph! {:?} to {:?}",
            edge.source(),
            edge.target()
        );
    }
}

#[test]
fn node_indices() {
    let mut g = StableGraph::<_, ()>::new();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    g.remove_node(b);
    let mut iter = g.node_indices();
    assert_eq!(iter.next(), Some(a));
    assert_eq!(iter.next(), Some(c));
    assert_eq!(iter.next(), None);
}

#[test]
fn node_bound() {
    let mut g = StableGraph::<_, ()>::new();
    assert_eq!(g.node_bound(), g.node_count());
    for i in 0..10 {
        g.add_node(i);
        assert_eq!(g.node_bound(), g.node_count());
    }
    let full_count = g.node_count();
    g.remove_node(n(0));
    g.remove_node(n(2));
    assert_eq!(g.node_bound(), full_count);
    g.clear();
    assert_eq!(g.node_bound(), 0);
}

#[test]
fn edge_bound() {
    let mut g = StableGraph::<_, _>::new();
    assert_eq!(g.edge_bound(), g.edge_count());
    for i in 0..10 {
        g.add_node(i);
    }
    for i in 0..9 {
        g.add_edge(n(i), n(i + 1), i);
        assert_eq!(g.edge_bound(), g.edge_count());
    }
    let full_count = g.edge_count();
    g.remove_edge(e(0));
    g.remove_edge(e(2));
    assert_eq!(g.edge_bound(), full_count);
    g.clear();
    assert_eq!(g.edge_bound(), 0);
}

#[test]
fn clear_edges() {
    let mut gr = scc_graph();
    gr.remove_node(n(1));
    gr.clear_edges();
    // check that we use the free list for the vacancies
    assert_eq!(gr.add_node(()), n(1));
    assert_eq!(gr.add_node(()), n(4));
    assert!(gr.edge_references().next().is_none());
    assert!(gr.node_indices().all(|i| gr.neighbors(i).next().is_none()));
}

fn assert_sccs_eq(mut res: Vec<Vec<NodeIndex>>, normalized: Vec<Vec<NodeIndex>>) {
    // normalize the result and compare with the answer.
    for scc in &mut res {
        scc.sort();
    }
    // sort by minimum element
    res.sort_by(|v, w| v[0].cmp(&w[0]));
    assert_eq!(res, normalized);
}

fn scc_graph() -> StableGraph<(), ()> {
    let mut gr: StableGraph<(), ()> = StableGraph::from_edges([
        (6, 0),
        (0, 3),
        (3, 6),
        (8, 6),
        (8, 2),
        (2, 5),
        (5, 8),
        (7, 5),
        (1, 7),
        (7, 4),
        (4, 1),
    ]);
    // make an identical replacement of n(4) and leave a hole
    let x = gr.add_node(());
    gr.add_edge(n(7), x, ());
    gr.add_edge(x, n(1), ());
    gr.remove_node(n(4));
    gr
}

#[test]
fn test_scc() {
    let gr = scc_graph();
    println!("{gr:?}");

    let x = n(gr.node_bound() - 1);
    assert_sccs_eq(
        kosaraju_scc(&gr),
        vec![
            vec![n(0), n(3), n(6)],
            vec![n(1), n(7), x],
            vec![n(2), n(5), n(8)],
        ],
    );
}

#[test]
fn test_tarjan_scc() {
    let gr = scc_graph();

    let x = n(gr.node_bound() - 1);
    assert_sccs_eq(
        tarjan_scc(&gr),
        vec![
            vec![n(0), n(3), n(6)],
            vec![n(1), n(7), x],
            vec![n(2), n(5), n(8)],
        ],
    );
}

fn make_graph<Ty>() -> StableGraph<(), i32, Ty>
where
    Ty: EdgeType,
{
    let mut gr = StableGraph::default();
    let mut c = 0..;
    let mut e = || -> i32 { c.next().unwrap() };
    gr.extend_with_edges([
        (6, 0, e()),
        (0, 3, e()),
        (3, 6, e()),
        (8, 6, e()),
        (8, 2, e()),
        (2, 5, e()),
        (5, 8, e()),
        (7, 5, e()),
        (1, 7, e()),
        (7, 4, e()),
        (8, 6, e()), // parallel edge
        (4, 1, e()),
    ]);
    // make an identical replacement of n(4) and leave a hole
    let x = gr.add_node(());
    gr.add_edge(n(7), x, e());
    gr.add_edge(x, n(1), e());
    gr.add_edge(x, x, e()); // make two self loops
    let rm_self_loop = gr.add_edge(x, x, e());
    gr.add_edge(x, x, e());
    gr.remove_node(n(4));
    gr.remove_node(n(6));
    gr.remove_edge(rm_self_loop);
    gr
}

defmac!(edges ref gr, x => gr.edges(x).map(|r| (r.target(), *r.weight())));

#[test]
fn test_edges_directed() {
    let gr = make_graph::<Directed>();
    let x = n(9);
    assert_equal(edges!(&gr, x), vec![(x, 16), (x, 14), (n(1), 13)]);
    assert_equal(edges!(&gr, n(0)), vec![(n(3), 1)]);
    assert_equal(edges!(&gr, n(4)), vec![]);
}

#[test]
fn test_edge_references() {
    let gr = make_graph::<Directed>();
    assert_eq!(gr.edge_count(), gr.edge_references().count());
}

#[test]
fn test_edges_undirected() {
    let gr = make_graph::<Undirected>();
    let x = n(9);
    assert_equal(
        edges!(&gr, x),
        vec![(x, 16), (x, 14), (n(1), 13), (n(7), 12)],
    );
    assert_equal(edges!(&gr, n(0)), vec![(n(3), 1)]);
    assert_equal(edges!(&gr, n(4)), vec![]);
}

#[test]
fn test_edge_iterators_directed() {
    let gr = make_graph::<Directed>();
    for i in gr.node_indices() {
        itertools::assert_equal(gr.edges_directed(i, Outgoing), gr.edges(i));
        for edge in gr.edges_directed(i, Outgoing) {
            assert_eq!(
                edge.source(),
                i,
                "outgoing edges should have a fixed source"
            );
        }
    }
    let mut incoming = vec![Vec::new(); gr.node_bound()];

    for i in gr.node_indices() {
        for j in gr.neighbors(i) {
            incoming[j.index()].push(i);
        }
    }

    println!("{gr:#?}");
    for i in gr.node_indices() {
        itertools::assert_equal(
            gr.edges_directed(i, Incoming).map(|e| e.source()),
            incoming[i.index()].iter().rev().cloned(),
        );
        for edge in gr.edges_directed(i, Incoming) {
            assert_eq!(
                edge.target(),
                i,
                "incoming edges should have a fixed target"
            );
        }
    }
}

#[test]
fn test_edge_iterators_undir() {
    let gr = make_graph::<Undirected>();
    for i in gr.node_indices() {
        itertools::assert_equal(gr.edges_directed(i, Outgoing), gr.edges(i));
        for edge in gr.edges_directed(i, Outgoing) {
            assert_eq!(
                edge.source(),
                i,
                "outgoing edges should have a fixed source"
            );
        }
    }
    for i in gr.node_indices() {
        itertools::assert_equal(gr.edges_directed(i, Incoming), gr.edges(i));
        for edge in gr.edges_directed(i, Incoming) {
            assert_eq!(
                edge.target(),
                i,
                "incoming edges should have a fixed target"
            );
        }
    }
}

#[test]
#[should_panic(expected = "is not a node")]
fn add_edge_vacant() {
    let mut g = StableGraph::<_, _>::new();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let _ = g.add_node(2);
    let _ = g.remove_node(b);
    g.add_edge(a, b, 1);
}

#[test]
#[should_panic(expected = "is not a node")]
fn add_edge_oob() {
    let mut g = StableGraph::<_, _>::new();
    let a = g.add_node(0);
    let _ = g.add_node(1);
    let _ = g.add_node(2);
    g.add_edge(a, n(4), 1);
}

#[test]
fn test_node_references() {
    let gr = scc_graph();

    itertools::assert_equal(gr.node_references().map(|(i, _)| i), gr.node_indices());
}

#[test]
fn iterators_undir() {
    let mut g = StableUnGraph::<_, _>::default();
    let a = g.add_node(0);
    let b = g.add_node(1);
    let c = g.add_node(2);
    let d = g.add_node(3);
    g.extend_with_edges([(a, b, 1), (a, c, 2), (b, c, 3), (c, c, 4), (a, d, 5)]);
    g.remove_node(b);

    itertools::assert_equal(g.neighbors(a), vec![d, c]);
    itertools::assert_equal(g.neighbors(c), vec![c, a]);
    itertools::assert_equal(g.neighbors(d), vec![a]);

    // the node that was removed
    itertools::assert_equal(g.neighbors(b), vec![]);

    // remove one more
    g.remove_node(c);
    itertools::assert_equal(g.neighbors(c), vec![]);
}

#[test]
fn iter_multi_edges() {
    let mut gr = StableGraph::new();
    let a = gr.add_node("a");
    let b = gr.add_node("b");
    let c = gr.add_node("c");

    let mut connecting_edges = HashSet::new();

    gr.add_edge(a, a, ());
    connecting_edges.insert(gr.add_edge(a, b, ()));
    gr.add_edge(a, c, ());
    gr.add_edge(c, b, ());
    connecting_edges.insert(gr.add_edge(a, b, ()));
    gr.add_edge(b, a, ());

    let mut iter = gr.edges_connecting(a, b);

    let edge_id = iter.next().unwrap().id();
    assert!(connecting_edges.contains(&edge_id));
    connecting_edges.remove(&edge_id);

    let edge_id = iter.next().unwrap().id();
    assert!(connecting_edges.contains(&edge_id));
    connecting_edges.remove(&edge_id);

    assert_eq!(None, iter.next());
    assert!(connecting_edges.is_empty());
}

#[test]
fn iter_multi_undirected_edges() {
    let mut gr: StableUnGraph<_, _> = Default::default();
    let a = gr.add_node("a");
    let b = gr.add_node("b");
    let c = gr.add_node("c");

    let mut connecting_edges = HashSet::new();

    gr.add_edge(a, a, ());
    connecting_edges.insert(gr.add_edge(a, b, ()));
    gr.add_edge(a, c, ());
    gr.add_edge(c, b, ());
    connecting_edges.insert(gr.add_edge(a, b, ()));
    connecting_edges.insert(gr.add_edge(b, a, ()));

    let mut iter = gr.edges_connecting(a, b);

    let edge_id = iter.next().unwrap().id();
    assert!(connecting_edges.contains(&edge_id));
    connecting_edges.remove(&edge_id);

    let edge_id = iter.next().unwrap().id();
    assert!(connecting_edges.contains(&edge_id));
    connecting_edges.remove(&edge_id);

    let edge_id = iter.next().unwrap().id();
    assert!(connecting_edges.contains(&edge_id));
    connecting_edges.remove(&edge_id);

    assert_eq!(None, iter.next());
    assert!(connecting_edges.is_empty());
}

#[test]
fn dot() {
    let mut gr = StableGraph::new();
    let a = gr.add_node("x");
    let b = gr.add_node("y");
    gr.add_edge(a, a, "10");
    gr.add_edge(a, b, "20");
    let dot_output = format!("{}", Dot::new(&gr));
    assert_eq!(
        dot_output,
        r#"digraph {
    0 [ label = "x" ]
    1 [ label = "y" ]
    0 -> 0 [ label = "10" ]
    0 -> 1 [ label = "20" ]
}
"#
    );
}

defmac!(iter_eq a, b => a.eq(b));
defmac!(nodes_eq ref a, ref b => a.node_references().eq(b.node_references()));
defmac!(edgew_eq ref a, ref b => a.edge_references().eq(b.edge_references()));
defmac!(edges_eq ref a, ref b =>
        iter_eq!(
            a.edge_references().map(|e| (e.source(), e.target())),
            b.edge_references().map(|e| (e.source(), e.target()))));

#[test]
fn from() {
    let mut gr1 = StableGraph::new();
    let a = gr1.add_node(1);
    let b = gr1.add_node(2);
    let c = gr1.add_node(3);
    gr1.add_edge(a, a, 10);
    gr1.add_edge(a, b, 20);
    gr1.add_edge(b, c, 30);
    gr1.add_edge(a, c, 40);

    let gr2 = Graph::from(gr1.clone());
    let gr3 = StableGraph::from(gr2);
    assert!(nodes_eq!(&gr1, &gr3));
    assert!(edgew_eq!(&gr1, &gr3));
    assert!(edges_eq!(&gr1, &gr3));

    gr1.remove_node(b);

    let gr4 = Graph::from(gr1);
    let gr5 = StableGraph::from(gr4.clone());

    let mut ans = StableGraph::new();
    let a = ans.add_node(1);
    let c = ans.add_node(3);
    ans.add_edge(a, a, 10);
    ans.add_edge(a, c, 40);

    assert!(nodes_eq!(&gr4, &ans));
    assert!(edges_eq!(&gr4, &ans));

    assert!(nodes_eq!(&gr5, &ans));
    assert!(edgew_eq!(&gr5, &ans));
    assert!(edges_eq!(&gr5, &ans));
}

use petgraph::data::FromElements;
use petgraph::stable_graph::StableGraph;

#[test]
fn from_min_spanning_tree() {
    let mut g = StableGraph::new();
    let mut nodes = Vec::new();
    for _ in 0..6 {
        nodes.push(g.add_node(()));
    }
    let es = [(4, 5), (3, 4), (3, 5)];
    for &(a, b) in es.iter() {
        g.add_edge(NodeIndex::new(a), NodeIndex::new(b), ());
    }
    for &node in nodes.iter().take(3) {
        let _ = g.remove_node(node);
    }
    let _ = StableGraph::<(), (), Undirected, usize>::from_elements(min_spanning_tree(&g));
}

#[test]
fn weights_mut_iterator() {
    let mut gr = StableGraph::new();
    let a = gr.add_node(1);
    let b = gr.add_node(2);
    let c = gr.add_node(3);
    let e1 = gr.add_edge(a, a, 10);
    let e2 = gr.add_edge(a, b, 20);
    let e3 = gr.add_edge(b, c, 30);
    let e4 = gr.add_edge(a, c, 40);

    for n in gr.node_weights_mut() {
        *n += 1;
    }
    assert_eq!(gr[a], 2);
    assert_eq!(gr[b], 3);
    assert_eq!(gr[c], 4);

    for e in gr.edge_weights_mut() {
        *e -= 1;
    }
    assert_eq!(gr[e1], 9);
    assert_eq!(gr[e2], 19);
    assert_eq!(gr[e3], 29);
    assert_eq!(gr[e4], 39);

    // test on deletion
    gr.remove_node(b);
    assert_eq!(gr.node_weights_mut().count(), gr.node_count());
    assert_eq!(gr.edge_weights_mut().count(), gr.edge_count());
}

#[test]
fn test_map() {
    let mut g: StableGraph<_, _, Undirected> = StableGraph::with_capacity(0, 0);
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let ab = g.add_edge(a, b, 7);
    let bc = g.add_edge(b, c, 14);
    let ca = g.add_edge(c, a, 9);

    let g2 = g.map(|_, name| format!("map-{name}"), |_, weight| weight * 2);
    assert_eq!(g2.node_count(), 3);
    assert_eq!(g2.node_weight(a).map(|s| &**s), Some("map-A"));
    assert_eq!(g2.node_weight(b).map(|s| &**s), Some("map-B"));
    assert_eq!(g2.node_weight(c).map(|s| &**s), Some("map-C"));
    assert_eq!(g2.edge_count(), 3);
    assert_eq!(g2.edge_weight(ab), Some(&14));
    assert_eq!(g2.edge_weight(bc), Some(&28));
    assert_eq!(g2.edge_weight(ca), Some(&18));
}

#[test]
fn test_map_owned() {
    let mut g: StableGraph<_, _, Undirected> = StableGraph::with_capacity(0, 0);
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let ab = g.add_edge(a, b, 7);
    let bc = g.add_edge(b, c, 14);
    let ca = g.add_edge(c, a, 9);

    let g2 = g.map_owned(|_, name| format!("map-{name}"), |_, weight| weight * 2);
    assert_eq!(g2.node_count(), 3);
    assert_eq!(g2.node_weight(a).map(|s| &**s), Some("map-A"));
    assert_eq!(g2.node_weight(b).map(|s| &**s), Some("map-B"));
    assert_eq!(g2.node_weight(c).map(|s| &**s), Some("map-C"));
    assert_eq!(g2.edge_count(), 3);
    assert_eq!(g2.edge_weight(ab), Some(&14));
    assert_eq!(g2.edge_weight(bc), Some(&28));
    assert_eq!(g2.edge_weight(ca), Some(&18));
}

#[test]
fn test_filter_map() {
    let mut g: StableGraph<_, _, Undirected> = StableGraph::with_capacity(0, 0);
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");
    let f = g.add_node("F");
    g.add_edge(a, b, 7);
    g.add_edge(c, a, 9);
    g.add_edge(a, d, 14);
    g.add_edge(b, c, 10);
    g.add_edge(d, c, 2);
    g.add_edge(d, e, 9);
    g.add_edge(b, f, 15);
    g.add_edge(c, f, 11);
    g.add_edge(e, f, 6);
    println!("{g:?}");

    let g2 = g.filter_map(
        |_, name| Some(*name),
        |_, &weight| if weight >= 10 { Some(weight) } else { None },
    );
    assert_eq!(g2.edge_count(), 4);
    for weight in g2.edge_weights() {
        assert!(*weight >= 10);
    }
    assert_eq!(g2.node_count(), g.node_count());
    // Check if node indices are compatible
    for i in g.node_indices() {
        assert_eq!(g2.node_weight(i), g.node_weight(i));
    }

    let g3 = g.filter_map(
        |i, &name| if i == a || i == e { None } else { Some(name) },
        |i, &weight| {
            let (source, target) = g.edge_endpoints(i).unwrap();
            // don't map edges from a removed node
            assert!(source != a);
            assert!(target != a);
            assert!(source != e);
            assert!(target != e);
            Some(weight)
        },
    );
    assert_eq!(g3.node_count(), g.node_count() - 2);
    assert_eq!(g3.edge_count(), g.edge_count() - 5);
    assert_graph_consistent(&g3);
    // Check if node indices are compatible
    for i in g3.node_indices() {
        assert_eq!(g3.node_weight(i), g.node_weight(i));
    }

    let mut g4 = g.clone();
    g4.retain_edges(|gr, i| {
        let (s, t) = gr.edge_endpoints(i).unwrap();
        !(s == a || s == e || t == a || t == e)
    });
    assert_eq!(g4.edge_count(), g.edge_count() - 5);
    assert_graph_consistent(&g4);
}

#[test]
fn test_filter_map_owned() {
    let mut g: StableGraph<_, _, Undirected> = StableGraph::with_capacity(0, 0);
    let a = g.add_node("A".to_owned());
    let b = g.add_node("B".to_owned());
    let c = g.add_node("C".to_owned());
    let d = g.add_node("D".to_owned());
    let e = g.add_node("E".to_owned());
    let f = g.add_node("F".to_owned());
    g.add_edge(a, b, 7);
    g.add_edge(c, a, 9);
    g.add_edge(a, d, 14);
    g.add_edge(b, c, 10);
    g.add_edge(d, c, 2);
    g.add_edge(d, e, 9);
    g.add_edge(b, f, 15);
    g.add_edge(c, f, 11);
    g.add_edge(e, f, 6);
    println!("{g:?}");

    let g2 = g.clone().filter_map_owned(
        |_, name| Some(name),
        |_, weight| if weight >= 10 { Some(weight) } else { None },
    );
    assert_eq!(g2.edge_count(), 4);
    for weight in g2.edge_weights() {
        assert!(*weight >= 10);
    }
    assert_eq!(g2.node_count(), g.node_count());
    // Check if node indices are compatible
    for i in g.node_indices() {
        assert_eq!(g2.node_weight(i), g.node_weight(i));
    }

    let g3 = g.clone().filter_map_owned(
        |i, name| if i == a || i == e { None } else { Some(name) },
        |i, weight| {
            let (source, target) = g.edge_endpoints(i).unwrap();
            // don't map edges from a removed node
            assert_ne!(source, a);
            assert_ne!(target, a);
            assert_ne!(source, e);
            assert_ne!(target, e);
            Some(weight)
        },
    );
    assert_eq!(g3.node_count(), g.node_count() - 2);
    assert_eq!(g3.edge_count(), g.edge_count() - 5);
    assert_graph_consistent(&g3);
    // Check if node indices are compatible
    for i in g3.node_indices() {
        assert_eq!(g3.node_weight(i), g.node_weight(i));
    }

    let mut g4 = g.clone();
    g4.retain_edges(|gr, i| {
        let (s, t) = gr.edge_endpoints(i).unwrap();
        !(s == a || s == e || t == a || t == e)
    });
    assert_eq!(g4.edge_count(), g.edge_count() - 5);
    assert_graph_consistent(&g4);
}

/// Test that the order of neighbors returned by `neighbors` is correct.
/// See `neighbors` docs for more details.
#[test]
fn test_neighbors_iteration_order() {
    // The test graph looks like this:
    //      5
    //      |
    //      v
    // 0 -> 1 -> 3
    // |    |
    // v    v
    // 2    4
    let g = StableGraph::<(), (), Directed>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let neighbors_0_dir: Vec<_> = g.neighbors(NodeIndexable::from_index(&g, 0)).collect();
    let neighbors_1_dir: Vec<_> = g.neighbors(NodeIndexable::from_index(&g, 1)).collect();

    assert_eq!(
        neighbors_0_dir,
        vec![
            NodeIndexable::from_index(&g, 2),
            NodeIndexable::from_index(&g, 1)
        ]
    );
    assert_eq!(
        neighbors_1_dir,
        vec![
            NodeIndexable::from_index(&g, 3),
            NodeIndexable::from_index(&g, 4)
        ]
    );

    let g = StableGraph::<(), (), Undirected>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let neighbors_0_undir: Vec<_> = g.neighbors(NodeIndexable::from_index(&g, 0)).collect();
    let neighbors_1_undir: Vec<_> = g.neighbors(NodeIndexable::from_index(&g, 1)).collect();

    assert_eq!(
        neighbors_0_undir,
        vec![
            NodeIndexable::from_index(&g, 2),
            NodeIndexable::from_index(&g, 1)
        ]
    );
    assert_eq!(
        neighbors_1_undir,
        vec![
            NodeIndexable::from_index(&g, 3),
            NodeIndexable::from_index(&g, 4),
            NodeIndexable::from_index(&g, 5),
            NodeIndexable::from_index(&g, 0)
        ]
    );
}

/// Test that the order of neighbors returned by `neighbors_directed` is correct.
/// See `neighbors_directed` docs for more details.
#[test]
fn test_neighbors_directed_iteration_order() {
    // The test graph looks like this:
    //      5
    //      |
    //      v
    // 0 -> 1 -> 3
    // |    |
    // v    v
    // 2    4
    let g = StableGraph::<(), (), Directed>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let neighbors_0_outgoing_dir: Vec<_> = g
        .neighbors_directed(NodeIndexable::from_index(&g, 0), Outgoing)
        .collect();
    let neighbors_0_incoming_dir: Vec<_> = g
        .neighbors_directed(NodeIndexable::from_index(&g, 0), Incoming)
        .collect();
    let neighbors_1_outgoing_dir: Vec<_> = g
        .neighbors_directed(NodeIndexable::from_index(&g, 1), Outgoing)
        .collect();
    let neighbors_1_incoming_dir: Vec<_> = g
        .neighbors_directed(NodeIndexable::from_index(&g, 1), Incoming)
        .collect();

    assert_eq!(
        neighbors_0_outgoing_dir,
        vec![
            NodeIndexable::from_index(&g, 2),
            NodeIndexable::from_index(&g, 1)
        ]
    );
    assert_eq!(neighbors_0_incoming_dir, vec![]);
    assert_eq!(
        neighbors_1_outgoing_dir,
        vec![
            NodeIndexable::from_index(&g, 3),
            NodeIndexable::from_index(&g, 4)
        ]
    );
    assert_eq!(
        neighbors_1_incoming_dir,
        vec![
            NodeIndexable::from_index(&g, 5),
            NodeIndexable::from_index(&g, 0)
        ]
    );

    let g = StableGraph::<(), (), Undirected>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let neighbors_0_outgoing_undir: Vec<_> = g
        .neighbors_directed(NodeIndexable::from_index(&g, 0), Outgoing)
        .collect();
    let neighbors_0_incoming_undir: Vec<_> = g
        .neighbors_directed(NodeIndexable::from_index(&g, 0), Incoming)
        .collect();
    let neighbors_1_outgoing_undir: Vec<_> = g
        .neighbors_directed(NodeIndexable::from_index(&g, 1), Outgoing)
        .collect();
    let neighbors_1_incoming_undir: Vec<_> = g
        .neighbors_directed(NodeIndexable::from_index(&g, 1), Incoming)
        .collect();

    assert_eq!(
        neighbors_0_outgoing_undir,
        vec![
            NodeIndexable::from_index(&g, 2),
            NodeIndexable::from_index(&g, 1)
        ]
    );
    assert_eq!(
        neighbors_0_incoming_undir,
        vec![
            NodeIndexable::from_index(&g, 2),
            NodeIndexable::from_index(&g, 1)
        ]
    );
    assert_eq!(
        neighbors_1_outgoing_undir,
        vec![
            NodeIndexable::from_index(&g, 3),
            NodeIndexable::from_index(&g, 4),
            NodeIndexable::from_index(&g, 5),
            NodeIndexable::from_index(&g, 0)
        ]
    );
    assert_eq!(
        neighbors_1_incoming_undir,
        vec![
            NodeIndexable::from_index(&g, 3),
            NodeIndexable::from_index(&g, 4),
            NodeIndexable::from_index(&g, 5),
            NodeIndexable::from_index(&g, 0)
        ]
    );
}

/// Test that the order of neighbors returned by `neighbors_undirected` is correct.
/// See `neighbors_undirected` docs for more details.
#[test]
fn test_neighbors_undirected_iteration_order() {
    // The test graph looks like this:
    //      5
    //      |
    //      v
    // 0 -> 1 -> 3
    // |    |
    // v    v
    // 2    4
    let g = StableGraph::<(), (), Directed>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let neighbors_0_outgoing_dir: Vec<_> = g
        .neighbors_undirected(NodeIndexable::from_index(&g, 0))
        .collect();
    let neighbors_0_incoming_dir: Vec<_> = g
        .neighbors_undirected(NodeIndexable::from_index(&g, 0))
        .collect();
    let neighbors_1_outgoing_dir: Vec<_> = g
        .neighbors_undirected(NodeIndexable::from_index(&g, 1))
        .collect();
    let neighbors_1_incoming_dir: Vec<_> = g
        .neighbors_undirected(NodeIndexable::from_index(&g, 1))
        .collect();

    assert_eq!(
        neighbors_0_outgoing_dir,
        vec![
            NodeIndexable::from_index(&g, 2),
            NodeIndexable::from_index(&g, 1)
        ]
    );
    assert_eq!(
        neighbors_0_incoming_dir,
        vec![
            NodeIndexable::from_index(&g, 2),
            NodeIndexable::from_index(&g, 1)
        ]
    );
    assert_eq!(
        neighbors_1_outgoing_dir,
        vec![
            NodeIndexable::from_index(&g, 3),
            NodeIndexable::from_index(&g, 4),
            NodeIndexable::from_index(&g, 5),
            NodeIndexable::from_index(&g, 0)
        ]
    );
    assert_eq!(
        neighbors_1_incoming_dir,
        vec![
            NodeIndexable::from_index(&g, 3),
            NodeIndexable::from_index(&g, 4),
            NodeIndexable::from_index(&g, 5),
            NodeIndexable::from_index(&g, 0)
        ]
    );

    let g = StableGraph::<(), (), Undirected>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let neighbors_0_outgoing_undir: Vec<_> = g
        .neighbors_undirected(NodeIndexable::from_index(&g, 0))
        .collect();
    let neighbors_0_incoming_undir: Vec<_> = g
        .neighbors_undirected(NodeIndexable::from_index(&g, 0))
        .collect();
    let neighbors_1_outgoing_undir: Vec<_> = g
        .neighbors_undirected(NodeIndexable::from_index(&g, 1))
        .collect();
    let neighbors_1_incoming_undir: Vec<_> = g
        .neighbors_undirected(NodeIndexable::from_index(&g, 1))
        .collect();

    assert_eq!(
        neighbors_0_outgoing_undir,
        vec![
            NodeIndexable::from_index(&g, 2),
            NodeIndexable::from_index(&g, 1)
        ]
    );
    assert_eq!(
        neighbors_0_incoming_undir,
        vec![
            NodeIndexable::from_index(&g, 2),
            NodeIndexable::from_index(&g, 1)
        ]
    );
    assert_eq!(
        neighbors_1_outgoing_undir,
        vec![
            NodeIndexable::from_index(&g, 3),
            NodeIndexable::from_index(&g, 4),
            NodeIndexable::from_index(&g, 5),
            NodeIndexable::from_index(&g, 0)
        ]
    );
    assert_eq!(
        neighbors_1_incoming_undir,
        vec![
            NodeIndexable::from_index(&g, 3),
            NodeIndexable::from_index(&g, 4),
            NodeIndexable::from_index(&g, 5),
            NodeIndexable::from_index(&g, 0)
        ]
    );
}

/// Test that the order of neighbors returned by `edges` is correct.
/// See `edges` docs for more details.
#[test]
fn test_edges_iteration_order() {
    // The test graph looks like this:
    //      5
    //      |
    //      v
    // 0 -> 1 -> 3
    // |    |
    // v    v
    // 2    4
    let g = StableGraph::<(), (), Directed>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let edges_0_dir: Vec<_> = g
        .edges(NodeIndexable::from_index(&g, 0))
        .map(|r| r.id())
        .collect();
    let edges_1_dir: Vec<_> = g
        .edges(NodeIndexable::from_index(&g, 1))
        .map(|r| r.id())
        .collect();

    assert_eq!(
        edges_0_dir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 2)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 1)
            )
            .unwrap()
        ]
    );
    assert_eq!(
        edges_1_dir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 3)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 4)
            )
            .unwrap(),
        ]
    );

    let g = StableGraph::<(), (), Undirected>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let edges_0_undir: Vec<_> = g
        .edges(NodeIndexable::from_index(&g, 0))
        .map(|r| r.id())
        .collect();
    let edges_1_undir: Vec<_> = g
        .edges(NodeIndexable::from_index(&g, 1))
        .map(|r| r.id())
        .collect();

    assert_eq!(
        edges_0_undir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 2)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 1)
            )
            .unwrap(),
        ]
    );

    assert_eq!(
        edges_1_undir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 3)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 4)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 5)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 0)
            )
            .unwrap()
        ]
    );
}

/// Test that the order of neighbors returned by `edges_directed` is correct.
/// See `edges_directed` docs for more details.
#[test]
fn test_edges_directed_iteration_order() {
    // The test graph looks like this:
    //      5
    //      |
    //      v
    // 0 -> 1 -> 3
    // |    |
    // v    v
    // 2    4
    let g = StableGraph::<(), (), Directed>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let edges_directed_0_outgoing_dir: Vec<_> = g
        .edges_directed(NodeIndexable::from_index(&g, 0), Outgoing)
        .map(|r| r.id())
        .collect();
    let edges_directed_0_incoming_dir: Vec<_> = g
        .edges_directed(NodeIndexable::from_index(&g, 0), Incoming)
        .map(|r| r.id())
        .collect();
    let edges_directed_1_outgoing_dir: Vec<_> = g
        .edges_directed(NodeIndexable::from_index(&g, 1), Outgoing)
        .map(|r| r.id())
        .collect();
    let edges_directed_1_incoming_dir: Vec<_> = g
        .edges_directed(NodeIndexable::from_index(&g, 1), Incoming)
        .map(|r| r.id())
        .collect();

    assert_eq!(
        edges_directed_0_outgoing_dir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 2)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 1)
            )
            .unwrap()
        ]
    );
    assert_eq!(edges_directed_0_incoming_dir, vec![]);
    assert_eq!(
        edges_directed_1_outgoing_dir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 3)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 4)
            )
            .unwrap()
        ]
    );
    assert_eq!(
        edges_directed_1_incoming_dir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 5),
                NodeIndexable::from_index(&g, 1)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 1)
            )
            .unwrap()
        ]
    );

    let g = StableGraph::<(), (), Undirected>::from_edges([(0, 1), (0, 2), (1, 4), (1, 3), (5, 1)]);

    let edges_directed_0_outgoing_undir: Vec<_> = g
        .edges_directed(NodeIndexable::from_index(&g, 0), Outgoing)
        .map(|r| r.id())
        .collect();
    let edges_directed_0_incoming_undir: Vec<_> = g
        .edges_directed(NodeIndexable::from_index(&g, 0), Incoming)
        .map(|r| r.id())
        .collect();
    let edges_directed_1_outgoing_undir: Vec<_> = g
        .edges_directed(NodeIndexable::from_index(&g, 1), Outgoing)
        .map(|r| r.id())
        .collect();
    let edges_directed_1_incoming_undir: Vec<_> = g
        .edges_directed(NodeIndexable::from_index(&g, 1), Incoming)
        .map(|r| r.id())
        .collect();

    assert_eq!(
        edges_directed_0_outgoing_undir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 2)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 1)
            )
            .unwrap()
        ]
    );
    assert_eq!(
        edges_directed_0_incoming_undir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 2)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 0),
                NodeIndexable::from_index(&g, 1)
            )
            .unwrap()
        ]
    );
    assert_eq!(
        edges_directed_1_outgoing_undir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 3)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 4)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 5)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 0)
            )
            .unwrap()
        ]
    );
    assert_eq!(
        edges_directed_1_incoming_undir,
        vec![
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 3)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 4)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 5)
            )
            .unwrap(),
            g.find_edge(
                NodeIndexable::from_index(&g, 1),
                NodeIndexable::from_index(&g, 0)
            )
            .unwrap()
        ]
    );
}

/// Test that the order of neighbors returned by `edges_connecting` is correct.
/// See `edges_connecting` docs for more details.
#[test]
fn test_edges_connecting_iteration_order() {
    let mut g = StableGraph::<(), u8, Directed>::new();

    let node_zero = g.add_node(());
    let node_one = g.add_node(());

    // Edges from node_zero to node_one
    let edge_zero = g.add_edge(node_zero, node_one, 1);
    let edge_one = g.add_edge(node_zero, node_one, 2);
    let edge_two = g.add_edge(node_zero, node_one, 3);

    // Edges from node_one to node_zero
    let edge_three = g.add_edge(node_one, node_zero, 4);
    let edge_four = g.add_edge(node_one, node_zero, 5);
    let edge_five = g.add_edge(node_one, node_zero, 6);

    let edges_connecting_one_to_two: Vec<_> = g
        .edges_connecting(node_zero, node_one)
        .map(|r| r.id())
        .collect();

    assert_eq!(
        edges_connecting_one_to_two,
        vec![edge_two, edge_one, edge_zero]
    );

    let edges_connecting_two_to_one: Vec<_> = g
        .edges_connecting(node_one, node_zero)
        .map(|r| r.id())
        .collect();

    assert_eq!(
        edges_connecting_two_to_one,
        vec![edge_five, edge_four, edge_three]
    );

    let mut g = StableGraph::<(), u8, Undirected>::default();

    let node_zero = g.add_node(());
    let node_one = g.add_node(());

    // Edges from node_zero to node_one
    let edge_zero = g.add_edge(node_zero, node_one, 1);
    let edge_one = g.add_edge(node_zero, node_one, 2);
    let edge_two = g.add_edge(node_zero, node_one, 3);

    // Edges from node_one to node_zero
    let edge_three = g.add_edge(node_one, node_zero, 4);
    let edge_four = g.add_edge(node_one, node_zero, 5);
    let edge_five = g.add_edge(node_one, node_zero, 6);

    let edges_connecting_one_to_two: Vec<_> = g
        .edges_connecting(node_zero, node_one)
        .map(|r| r.id())
        .collect();

    assert_eq!(
        edges_connecting_one_to_two,
        vec![edge_two, edge_one, edge_zero, edge_five, edge_four, edge_three]
    );

    let edges_connecting_two_to_one: Vec<_> = g
        .edges_connecting(node_one, node_zero)
        .map(|r| r.id())
        .collect();

    assert_eq!(
        edges_connecting_two_to_one,
        vec![edge_five, edge_four, edge_three, edge_two, edge_one, edge_zero]
    );
}
