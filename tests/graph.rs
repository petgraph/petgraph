extern crate petgraph;

use std::{collections::HashSet, hash::Hash};

use petgraph as pg;
use petgraph::{
    algo::{
        astar, dijkstra, dominators, has_path_connecting, is_bipartite_undirected,
        is_cyclic_undirected, is_isomorphic_matching, min_spanning_tree, DfsSpace,
    },
    dot::Dot,
    graph::{node_index as n, IndexType},
    prelude::*,
    visit::{
        IntoEdges, IntoEdgesDirected, IntoNeighbors, IntoNodeIdentifiers, NodeFiltered, Reversed,
        Topo, VisitMap, Walker,
    },
    EdgeType,
};

fn set<I>(iter: I) -> HashSet<I::Item>
where
    I: IntoIterator,
    I::Item: Hash + Eq,
{
    iter.into_iter().collect()
}

fn assert_is_topo_order<N, E>(gr: &Graph<N, E, Directed>, order: &[NodeIndex]) {
    assert_eq!(gr.node_count(), order.len());
    // check all the edges of the graph
    for edge in gr.raw_edges() {
        let a = edge.source();
        let b = edge.target();
        let ai = order.iter().position(|x| *x == a).unwrap();
        let bi = order.iter().position(|x| *x == b).unwrap();
        println!("Check that {:?} is before {:?}", a, b);
        assert!(
            ai < bi,
            "Topo order: assertion that node {:?} is before {:?} failed",
            a,
            b
        );
    }
}

/// Compare two scc sets. Inside each scc, the order does not matter,
/// but the order of the sccs is significant.
fn assert_sccs_eq(
    mut res: Vec<Vec<NodeIndex>>,
    mut answer: Vec<Vec<NodeIndex>>,
    scc_order_matters: bool,
) {
    // normalize the result and compare with the answer.
    for scc in &mut res {
        scc.sort();
    }
    for scc in &mut answer {
        scc.sort();
    }
    if !scc_order_matters {
        res.sort();
        answer.sort();
    }
    assert_eq!(res, answer);
}

fn make_edge_iterator_graph<Ty: EdgeType>() -> Graph<f64, f64, Ty> {
    let mut gr = Graph::default();
    let a = gr.add_node(0.);
    let b = gr.add_node(0.);
    let c = gr.add_node(0.);
    let d = gr.add_node(0.);
    let e = gr.add_node(0.);
    let f = gr.add_node(0.);
    let g = gr.add_node(0.);
    gr.add_edge(a, b, 7.0);
    gr.add_edge(a, d, 5.);
    gr.add_edge(d, b, 9.);
    gr.add_edge(b, c, 8.);
    gr.add_edge(b, e, 7.);
    gr.add_edge(c, c, 8.);
    gr.add_edge(c, e, 5.);
    gr.add_edge(d, e, 15.);
    gr.add_edge(d, f, 6.);
    gr.add_edge(f, e, 8.);
    gr.add_edge(f, g, 11.);
    gr.add_edge(e, g, 9.);

    gr
}

// TODO: move to core
#[test]
fn toposort_generic() {
    // This is a DAG, visit it in order
    let mut gr = Graph::<_, _>::new();
    let b = gr.add_node(("B", 0.));
    let a = gr.add_node(("A", 0.));
    let c = gr.add_node(("C", 0.));
    let d = gr.add_node(("D", 0.));
    let e = gr.add_node(("E", 0.));
    let f = gr.add_node(("F", 0.));
    let g = gr.add_node(("G", 0.));
    gr.add_edge(a, b, 7.0);
    gr.add_edge(a, d, 5.);
    gr.add_edge(d, b, 9.);
    gr.add_edge(b, c, 8.);
    gr.add_edge(b, e, 7.);
    gr.add_edge(c, e, 5.);
    gr.add_edge(d, e, 15.);
    gr.add_edge(d, f, 6.);
    gr.add_edge(f, e, 8.);
    gr.add_edge(f, g, 11.);
    gr.add_edge(e, g, 9.);

    assert!(!pg::algo::is_cyclic_directed(&gr));
    let mut index = 0.;
    let mut topo = Topo::new(&gr);
    while let Some(nx) = topo.next(&gr) {
        gr[nx].1 = index;
        index += 1.;
    }

    let mut order = Vec::new();
    index = 0.;
    let mut topo = Topo::new(&gr);
    while let Some(nx) = topo.next(&gr) {
        order.push(nx);
        assert_eq!(gr[nx].1, index);
        index += 1.;
    }
    println!("{:?}", gr);
    assert_is_topo_order(&gr, &order);

    {
        order.clear();
        let mut topo = Topo::new(&gr);
        while let Some(nx) = topo.next(&gr) {
            order.push(nx);
        }
        println!("{:?}", gr);
        assert_is_topo_order(&gr, &order);
    }
    let mut gr2 = gr.clone();
    gr.add_edge(e, d, -1.);
    assert!(pg::algo::is_cyclic_directed(&gr));
    assert!(pg::algo::toposort(&gr, None).is_err());
    gr2.add_edge(d, d, 0.);
    assert!(pg::algo::is_cyclic_directed(&gr2));
    assert!(pg::algo::toposort(&gr2, None).is_err());
}

// TODO: move to algo
#[test]
fn test_has_path() {
    // This is a DAG, visit it in order
    let mut gr = Graph::<_, _>::new();
    let b = gr.add_node(("B", 0.));
    let a = gr.add_node(("A", 0.));
    let c = gr.add_node(("C", 0.));
    let d = gr.add_node(("D", 0.));
    let e = gr.add_node(("E", 0.));
    let f = gr.add_node(("F", 0.));
    let g = gr.add_node(("G", 0.));
    gr.add_edge(a, b, 7.0);
    gr.add_edge(a, d, 5.);
    gr.add_edge(d, b, 9.);
    gr.add_edge(b, c, 8.);
    gr.add_edge(b, e, 7.);
    gr.add_edge(c, e, 5.);
    gr.add_edge(d, e, 15.);
    gr.add_edge(d, f, 6.);
    gr.add_edge(f, e, 8.);
    gr.add_edge(f, g, 11.);
    gr.add_edge(e, g, 9.);
    // disconnected island

    let h = gr.add_node(("H", 0.));
    let i = gr.add_node(("I", 0.));
    gr.add_edge(h, i, 2.);
    gr.add_edge(i, h, -2.);

    let mut state = DfsSpace::default();

    gr.add_edge(b, a, 99.);

    assert!(has_path_connecting(&gr, c, c, None));

    for edge in gr.edge_references() {
        assert!(has_path_connecting(&gr, edge.source(), edge.target(), None));
    }
    assert!(has_path_connecting(&gr, a, g, Some(&mut state)));
    assert!(!has_path_connecting(&gr, a, h, Some(&mut state)));
    assert!(has_path_connecting(&gr, a, c, None));
    assert!(has_path_connecting(&gr, a, c, Some(&mut state)));
    assert!(!has_path_connecting(&gr, h, a, Some(&mut state)));
}

fn assert_graph_consistent<N, E, Ty, Ix>(g: &Graph<N, E, Ty, Ix>)
where
    Ty: EdgeType,
    Ix: IndexType,
{
    assert_eq!(g.node_count(), g.node_indices().count());
    assert_eq!(g.edge_count(), g.edge_indices().count());
    for edge in g.raw_edges() {
        assert!(
            g.find_edge(edge.source(), edge.target()).is_some(),
            "Edge not in graph! {:?} to {:?}",
            edge.source(),
            edge.target()
        );
    }
}

fn degree<'a, G>(g: G, node: G::NodeId) -> usize
where
    G: IntoNeighbors,
    G::NodeId: PartialEq,
{
    // self loops count twice
    let original_node = node.clone();
    let mut degree = 0;
    for v in g.neighbors(node) {
        degree += if v == original_node { 2 } else { 1 };
    }
    degree
}

// TODO: move to core
#[test]
fn filtered() {
    let mut g = Graph::new();
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
    println!("{:?}", g);

    let filt = NodeFiltered(&g, |n: NodeIndex| n != c && n != e);

    let mut dfs = DfsPostOrder::new(&filt, a);
    let mut po = Vec::new();
    while let Some(nx) = dfs.next(&filt) {
        println!("Next: {:?}", nx);
        po.push(nx);
    }
    assert_eq!(set(po), set(g.node_identifiers().filter(|n| (filt.1)(*n))));
}

// TODO: move to core
#[test]
fn filtered_edge_reverse() {
    use petgraph::visit::EdgeFiltered;
    #[derive(Eq, PartialEq)]
    enum E {
        A,
        B,
    }

    // Start with single node graph with loop
    let mut g = Graph::new();
    let a = g.add_node("A");
    g.add_edge(a, a, E::A);
    let ef_a = EdgeFiltered::from_fn(&g, |edge| *edge.weight() == E::A);
    let mut po = Vec::new();
    let mut dfs = Dfs::new(&ef_a, a);
    while let Some(next_n_ix) = dfs.next(&ef_a) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![a]));

    // Check in reverse
    let mut po = Vec::new();
    let mut dfs = Dfs::new(&Reversed(&ef_a), a);
    while let Some(next_n_ix) = dfs.next(&Reversed(&ef_a)) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![a]));

    let mut g = Graph::new();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");
    let f = g.add_node("F");
    let h = g.add_node("H");
    let i = g.add_node("I");
    let j = g.add_node("J");

    g.add_edge(a, b, E::A);
    g.add_edge(b, c, E::A);
    g.add_edge(c, d, E::B);
    g.add_edge(d, e, E::A);
    g.add_edge(e, f, E::A);
    g.add_edge(e, h, E::A);
    g.add_edge(e, i, E::A);
    g.add_edge(i, j, E::A);

    let ef_a = EdgeFiltered::from_fn(&g, |edge| *edge.weight() == E::A);
    let ef_b = EdgeFiltered::from_fn(&g, |edge| *edge.weight() == E::B);

    // DFS down from a, filtered by E::A.
    let mut po = Vec::new();
    let mut dfs = Dfs::new(&ef_a, a);
    while let Some(next_n_ix) = dfs.next(&ef_a) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![a, b, c]));

    // Reversed DFS from f, filtered by E::A.
    let mut dfs = Dfs::new(&Reversed(&ef_a), f);
    let mut po = Vec::new();
    while let Some(next_n_ix) = dfs.next(&Reversed(&ef_a)) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![d, e, f]));

    // Reversed DFS from j, filtered by E::A.
    let mut dfs = Dfs::new(&Reversed(&ef_a), j);
    let mut po = Vec::new();
    while let Some(next_n_ix) = dfs.next(&Reversed(&ef_a)) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![d, e, i, j]));

    // Reversed DFS from c, filtered by E::A.
    let mut dfs = Dfs::new(&Reversed(&ef_a), c);
    let mut po = Vec::new();
    while let Some(next_n_ix) = dfs.next(&Reversed(&ef_a)) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![a, b, c]));

    // Reversed DFS from c, filtered by E::B.
    let mut dfs = Dfs::new(&Reversed(&ef_b), c);
    let mut po = Vec::new();
    while let Some(next_n_ix) = dfs.next(&Reversed(&ef_b)) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![c]));

    // Reversed DFS from d, filtered by E::B.
    let mut dfs = Dfs::new(&Reversed(&ef_b), d);
    let mut po = Vec::new();
    while let Some(next_n_ix) = dfs.next(&Reversed(&ef_b)) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![c, d]));

    // Now let's test the same graph but undirected

    let mut g = Graph::new_undirected();
    let a = g.add_node("A");
    let b = g.add_node("B");
    let c = g.add_node("C");
    let d = g.add_node("D");
    let e = g.add_node("E");
    let f = g.add_node("F");
    let h = g.add_node("H");
    let i = g.add_node("I");
    let j = g.add_node("J");

    g.add_edge(a, b, E::A);
    g.add_edge(b, c, E::A);
    g.add_edge(c, d, E::B);
    g.add_edge(d, e, E::A);
    g.add_edge(e, f, E::A);
    g.add_edge(e, h, E::A);
    g.add_edge(e, i, E::A);
    g.add_edge(i, j, E::A);

    let ef_a = EdgeFiltered::from_fn(&g, |edge| *edge.weight() == E::A);
    let ef_b = EdgeFiltered::from_fn(&g, |edge| *edge.weight() == E::B);
    let mut po = Vec::new();
    let mut dfs = Dfs::new(&Reversed(&ef_b), d);
    while let Some(next_n_ix) = dfs.next(&Reversed(&ef_b)) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![c, d]));

    let mut po = Vec::new();
    let mut dfs = Dfs::new(&Reversed(&ef_a), h);
    while let Some(next_n_ix) = dfs.next(&Reversed(&ef_a)) {
        po.push(next_n_ix);
    }
    assert_eq!(set(po), set(vec![d, e, f, h, i, j]));
}

// TODO: move to core
#[test]
fn dfs_visit() {
    use petgraph::visit::{depth_first_search, Control, DfsEvent::*, Time, VisitMap, Visitable};
    let gr: Graph<(), ()> = Graph::from_edges(&[
        (0, 5),
        (0, 2),
        (0, 3),
        (0, 1),
        (1, 3),
        (2, 3),
        (2, 4),
        (4, 0),
        (4, 5),
    ]);

    let invalid_time = Time(!0);
    let mut discover_time = vec![invalid_time; gr.node_count()];
    let mut finish_time = vec![invalid_time; gr.node_count()];
    let mut has_tree_edge = gr.visit_map();
    let mut edges = HashSet::new();
    depth_first_search(&gr, Some(n(0)), |evt| {
        println!("Event: {:?}", evt);
        match evt {
            Discover(n, t) => discover_time[n.index()] = t,
            Finish(n, t) => finish_time[n.index()] = t,
            TreeEdge(u, v) => {
                // v is an ancestor of u
                assert!(has_tree_edge.visit(v), "Two tree edges to {:?}!", v);
                assert!(discover_time[v.index()] == invalid_time);
                assert!(discover_time[u.index()] != invalid_time);
                assert!(finish_time[u.index()] == invalid_time);
                edges.insert((u, v));
            }
            BackEdge(u, v) => {
                // u is an ancestor of v
                assert!(discover_time[v.index()] != invalid_time);
                assert!(finish_time[v.index()] == invalid_time);
                edges.insert((u, v));
            }
            CrossForwardEdge(u, v) => {
                edges.insert((u, v));
            }
        }
    });
    assert!(discover_time.iter().all(|x| *x != invalid_time));
    assert!(finish_time.iter().all(|x| *x != invalid_time));
    assert_eq!(edges.len(), gr.edge_count());
    assert_eq!(
        edges,
        set(gr.edge_references().map(|e| (e.source(), e.target())))
    );
    println!("{:?}", discover_time);
    println!("{:?}", finish_time);

    // find path from 0 to 4
    let mut predecessor = vec![NodeIndex::end(); gr.node_count()];
    let start = n(0);
    let goal = n(4);
    let ret = depth_first_search(&gr, Some(start), |event| {
        if let TreeEdge(u, v) = event {
            predecessor[v.index()] = u;
            if v == goal {
                return Control::Break(u);
            }
        }
        Control::Continue
    });
    // assert we did terminate early
    assert!(ret.break_value().is_some());
    assert!(predecessor.iter().any(|x| *x == NodeIndex::end()));

    let mut next = goal;
    let mut path = vec![next];
    while next != start {
        let pred = predecessor[next.index()];
        path.push(pred);
        next = pred;
    }
    path.reverse();
    assert_eq!(&path, &[n(0), n(2), n(4)]);

    // check that if we prune 2, we never see 4.
    let start = n(0);
    let prune = n(2);
    let nongoal = n(4);
    let ret = depth_first_search(&gr, Some(start), |event| {
        if let Discover(n, _) = event {
            if n == prune {
                return Control::Prune;
            }
        } else if let TreeEdge(u, v) = event {
            if v == nongoal {
                return Control::Break(u);
            }
        }
        Control::Continue
    });
    assert!(ret.break_value().is_none());
}

// TODO: move to core
#[test]
fn filtered_post_order() {
    use petgraph::visit::NodeFiltered;

    let mut gr: Graph<(), ()> =
        Graph::from_edges(&[(0, 2), (1, 2), (0, 3), (1, 4), (2, 4), (4, 5), (3, 5)]);
    // map reachable nodes
    let mut dfs = Dfs::new(&gr, n(0));
    while let Some(_) = dfs.next(&gr) {}

    let map = dfs.discovered;
    gr.add_edge(n(0), n(1), ());
    let mut po = Vec::new();
    let mut dfs = DfsPostOrder::new(&gr, n(0));
    let f = NodeFiltered(&gr, map);
    while let Some(n) = dfs.next(&f) {
        po.push(n);
    }
    assert!(!po.contains(&n(1)));
}

// TODO: move to core
#[test]
fn filter_elements() {
    use petgraph::data::{
        Element::{Edge, Node},
        ElementIterator, FromElements,
    };
    let elements = vec![
        Node { weight: "A" },
        Node { weight: "B" },
        Node { weight: "C" },
        Node { weight: "D" },
        Node { weight: "E" },
        Node { weight: "F" },
        Edge {
            source: 0,
            target: 1,
            weight: 7,
        },
        Edge {
            source: 2,
            target: 0,
            weight: 9,
        },
        Edge {
            source: 0,
            target: 3,
            weight: 14,
        },
        Edge {
            source: 1,
            target: 2,
            weight: 10,
        },
        Edge {
            source: 3,
            target: 2,
            weight: 2,
        },
        Edge {
            source: 3,
            target: 4,
            weight: 9,
        },
        Edge {
            source: 1,
            target: 5,
            weight: 15,
        },
        Edge {
            source: 2,
            target: 5,
            weight: 11,
        },
        Edge {
            source: 4,
            target: 5,
            weight: 6,
        },
    ];
    let mut g = DiGraph::<_, _>::from_elements(elements.iter().cloned());
    println!("{:#?}", g);
    assert!(g.contains_edge(n(1), n(5)));
    let g2 =
        DiGraph::<_, _>::from_elements(elements.iter().cloned().filter_elements(|elt| match elt {
            Node { ref weight } if **weight == "B" => false,
            _ => true,
        }));
    println!("{:#?}", g2);
    g.remove_node(n(1));
    assert!(is_isomorphic_matching(
        &g,
        &g2,
        PartialEq::eq,
        PartialEq::eq
    ));
}

// TODO: move to algo
#[test]
fn test_edge_filtered() {
    use petgraph::{
        algo::connected_components,
        visit::{EdgeFiltered, IntoEdgeReferences},
    };

    let gr = UnGraph::<(), _>::from_edges(&[
        // cycle
        (0, 1, 7),
        (1, 2, 9),
        (2, 1, 14),
        // cycle
        (3, 4, 10),
        (4, 5, 2),
        (5, 3, 9),
        // cross edges
        (0, 3, -1),
        (1, 4, -2),
        (2, 5, -3),
    ]);
    assert_eq!(connected_components(&gr), 1);
    let positive_edges = EdgeFiltered::from_fn(&gr, |edge| *edge.weight() >= 0);
    assert_eq!(positive_edges.edge_references().count(), 6);
    assert!(
        positive_edges
            .edge_references()
            .all(|edge| *edge.weight() >= 0)
    );
    assert_eq!(connected_components(&positive_edges), 2);

    let mut dfs = DfsPostOrder::new(&positive_edges, n(0));
    while let Some(_) = dfs.next(&positive_edges) {}

    let n = n::<u32>;
    for node in &[n(0), n(1), n(2)] {
        assert!(dfs.discovered.is_visited(node));
    }
    for node in &[n(3), n(4), n(5)] {
        assert!(!dfs.discovered.is_visited(node));
    }
}
