extern crate petgraph;

use petgraph::prelude::*;
use petgraph::visit::Walker;

use petgraph::algo::{dijkstra, astar};

#[test]
fn test_dijkstra() {
    let mut g = Graph::new_undirected();
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
    for no in Bfs::new(&g, a).iter(&g) {
        println!("Visit {:?} = {:?}", no, g.node_weight(no));
    }

    let scores = dijkstra(&g, a, None, |e| *e.weight());
    let mut scores: Vec<_> = scores.into_iter().map(|(n, s)| (g[n], s)).collect();
    scores.sort();
    assert_eq!(scores,
       vec![("A", 0), ("B", 7), ("C", 9), ("D", 11), ("E", 20), ("F", 20)]);

    let scores = dijkstra(&g, a, Some(c), |e| *e.weight());
    assert_eq!(scores[&c], 9);
}

#[test]
fn test_astar_null_heuristic() {
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

    let path = astar(&g, a, |finish| finish == e, |e| *e.weight(), |_| 0);
    assert_eq!(path, Some((23, vec![a, d, e])));

    // check against dijkstra
    let dijkstra_run = dijkstra(&g, a, Some(e), |e| *e.weight());
    assert_eq!(dijkstra_run[&e], 23);

    let path = astar(&g, e, |finish| finish == b, |e| *e.weight(), |_| 0);
    assert_eq!(path, None);
}

#[test]
fn test_astar_manhattan_heuristic() {
    let mut g = Graph::new();
    let a = g.add_node((0., 0.));
    let b = g.add_node((2., 0.));
    let c = g.add_node((1., 1.));
    let d = g.add_node((0., 2.));
    let e = g.add_node((3., 3.));
    let f = g.add_node((4., 2.));
    let _ = g.add_node((5., 5.)); // no path to node
    g.add_edge(a, b, 2.);
    g.add_edge(a, d, 4.);
    g.add_edge(b, c, 1.);
    g.add_edge(b, f, 7.);
    g.add_edge(c, e, 5.);
    g.add_edge(e, f, 1.);
    g.add_edge(d, e, 1.);

    let heuristic_for = |f: NodeIndex| {
        let g = &g;
        move |node: NodeIndex| -> f32 {
            let (x1, y1): (f32, f32) = g[node];
            let (x2, y2): (f32, f32) = g[f];

            (x2 - x1).abs() + (y2 - y1).abs()
        }
    };
    let path = astar(&g, a, |finish| finish == f, |e| *e.weight(), heuristic_for(f));

    assert_eq!(path, Some((6., vec![a, d, e, f])));

    // check against dijkstra
    let dijkstra_run = dijkstra(&g, a, None, |e| *e.weight());

    for end in g.node_indices() {
        let astar_path = astar(&g, a, |finish| finish == end, |e| *e.weight(),
                               heuristic_for(end));
        assert_eq!(dijkstra_run.get(&end).cloned(),
                   astar_path.map(|t| t.0));
    }
}
