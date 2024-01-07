use petgraph::algo::minimum_cut;
use petgraph::prelude::*;
use petgraph::Graph;

#[test]
fn minimum_cut_test1() {
    let mut graph: Graph<(), u32, Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());

    graph.extend_with_edges(&[
        (a, b, 7),
        (b, c, 2),
        (c, d, 8),
        (d, a, 3),
        (e, f, 6),
        
        (f, g, 1),
        (g, h, 5),
        (h, e, 4),
       
    ]);
    let be = graph.add_edge(b, e, 1);
    let ch = graph.add_edge(c, h, 3);
    // a --7-- b --1-- e --6-- f
    // |       |       |       |
    // 3       2       4       1
    // |       |       |       |
    // d --8-- c---3-- h --5-- g

    let (edges, weight) = minimum_cut(&graph, |e| *e.weight());
    assert_eq!(edges.len(), 2);
    assert!(edges[0] == be && edges[1] == ch || edges[1] == be && edges[0] == ch);
    assert_eq!(weight, 4);
}


#[test]
fn minimum_cut_one_vertex() {
    let mut graph: Graph<(), u32, Undirected> = Graph::new_undirected();
    graph.add_node(());

    let (cut, weight) = minimum_cut(&graph, |e| *e.weight());
    assert_eq!(cut.len(), 0);
    assert_eq!(weight, 0);
}


#[test]
fn minimum_cut_two_vertices() {
    let mut graph: Graph<(), u32, Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());

    graph.extend_with_edges(&[
        (a, b, 1),
        (a, b, 3),
    ]);

    let (cut, weight) = minimum_cut(&graph, |e| *e.weight());
    assert_eq!(cut.len(), 2);
    assert_eq!(weight, 4);
}

#[test]
fn minimum_cut_disconnected() {
    let mut graph: Graph<(), u32, Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());

    graph.extend_with_edges(&[
        (a, b, 1),
        (b, c, 1),
        (c, a, 1),
        (d, e, 1),
        (e, f, 6),
        (f, d, 1),
    ]);

    let (cut, weight) = minimum_cut(&graph, |e| *e.weight());
    assert_eq!(cut.len(), 0);
    assert_eq!(weight, 0);
}

#[test]
fn minimum_cut_two_cliques() {
    let mut graph: Graph<(), u32, Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());

    let g = graph.add_node(());
    let h = graph.add_node(());
    let i = graph.add_node(());
    let j = graph.add_node(());
    let k = graph.add_node(());
    let l = graph.add_node(());

    let clique1 = vec!(a, b, c, d, e, f);
    let clique2 = vec!(g, h, i, j, k, l);

    for idx in 0 .. clique1.len() - 1 {
        for idx2 in idx+1 .. clique1.len() {
            graph.add_edge(clique1[idx], clique1[idx2], 1);
            graph.add_edge(clique2[idx], clique2[idx2], 1);
        }
    }
    graph.extend_with_edges(&[
        (a, g, 1),
        (b, h, 1),
        (c, i, 1),
    ]);

    let (cut, weight) = minimum_cut(&graph, |e| *e.weight());
    assert_eq!(cut.len(), 3);
    assert_eq!(weight, 3);
}
