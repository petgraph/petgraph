use petgraph::{
    algo::{min_spanning_tree, min_spanning_tree_prim},
    dot::Dot,
    graph::UnGraph,
    Graph, Undirected,
};

#[test]
fn mst_kruskal() {
    use petgraph::data::FromElements;

    let mut gr = Graph::<_, _>::new();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    gr.add_edge(a, b, 7.);
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

    // add a disjoint part
    let h = gr.add_node("H");
    let i = gr.add_node("I");
    let j = gr.add_node("J");
    gr.add_edge(h, i, 1.);
    gr.add_edge(h, j, 3.);
    gr.add_edge(i, j, 1.);

    println!("{}", Dot::new(&gr));

    let mst = UnGraph::from_elements(min_spanning_tree(&gr));

    println!("{}", Dot::new(&mst));
    println!("{:?}", Dot::new(&mst));
    println!("MST is:\n{:#?}", mst);
    assert!(mst.node_count() == gr.node_count());
    // |E| = |N| - 2  because there are two disconnected components.
    assert!(mst.edge_count() == gr.node_count() - 2);

    // check the exact edges are there
    assert!(mst.find_edge(a, b).is_some());
    assert!(mst.find_edge(a, d).is_some());
    assert!(mst.find_edge(b, e).is_some());
    assert!(mst.find_edge(e, c).is_some());
    assert!(mst.find_edge(e, g).is_some());
    assert!(mst.find_edge(d, f).is_some());

    assert!(mst.find_edge(h, i).is_some());
    assert!(mst.find_edge(i, j).is_some());

    assert!(mst.find_edge(d, b).is_none());
    assert!(mst.find_edge(b, c).is_none());
}

#[test]
fn mst_prim() {
    use petgraph::data::FromElements;

    let mut gr = UnGraph::<_, _>::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let c = gr.add_node("C");
    let d = gr.add_node("D");
    let e = gr.add_node("E");
    let f = gr.add_node("F");
    let g = gr.add_node("G");
    gr.add_edge(b, a, 7.);
    gr.add_edge(d, a, 5.);
    gr.add_edge(d, b, 9.);
    gr.add_edge(b, c, 8.);
    gr.add_edge(b, e, 7.);
    gr.add_edge(c, e, 5.);
    gr.add_edge(d, e, 15.);
    gr.add_edge(d, f, 6.);
    gr.add_edge(f, e, 8.);
    gr.add_edge(f, g, 11.);
    gr.add_edge(e, g, 9.);

    println!("{}", Dot::new(&gr));

    let mst = UnGraph::from_elements(min_spanning_tree_prim(&gr));

    println!("{}", Dot::new(&mst));
    println!("{:?}", Dot::new(&mst));
    println!("MST is:\n{:#?}", mst);

    assert!(mst.node_count() == gr.node_count());
    assert!(mst.edge_count() == gr.node_count() - 1);

    // check the exact edges are there
    assert!(mst.find_edge(a, d).is_some());
    assert!(mst.find_edge(a, b).is_some());
    assert!(mst.find_edge(d, f).is_some());
    assert!(mst.find_edge(b, e).is_some());
    assert!(mst.find_edge(e, c).is_some());
    assert!(mst.find_edge(e, g).is_some());
}

#[test]
fn mst_prim_trivial_graph() {
    use petgraph::data::FromElements;

    let mut gr = UnGraph::<_, _>::new_undirected();
    let a = gr.add_node("A");
    let b = gr.add_node("B");
    let a_b_weight = 7.;
    gr.add_edge(a, b, a_b_weight);

    println!("{}", Dot::new(&gr));

    let mst = UnGraph::from_elements(min_spanning_tree_prim(&gr));

    println!("{}", Dot::new(&mst));
    println!("{:?}", Dot::new(&mst));
    println!("MST is:\n{:#?}", mst);

    assert!(mst.node_count() == gr.node_count());
    assert!(mst.edge_count() == gr.node_count() - 1);

    assert!(mst.find_edge(a, b).is_some());
    let edge_weight = *mst.edge_weight(mst.find_edge(a, b).unwrap()).unwrap();
    assert_eq!(edge_weight, a_b_weight);
}

#[test]
fn mst_prim_graph_without_edges() {
    use petgraph::data::FromElements;

    let mut gr = UnGraph::<_, _>::new_undirected();
    gr.add_node("A");
    gr.add_node("B");
    gr.add_node("C");
    gr.add_node("D");
    gr.add_node("E");
    gr.add_node("F");
    gr.add_node("G");

    let mst: Graph<&str, usize, Undirected> = UnGraph::from_elements(min_spanning_tree_prim(&gr));

    assert!(mst.node_count() == gr.node_count());
    assert!(mst.edge_count() == 0);
}

#[test]
fn mst_prim_empty_graph() {
    use petgraph::data::FromElements;

    let gr = UnGraph::new_undirected();

    let mst: Graph<&str, usize, Undirected> = UnGraph::from_elements(min_spanning_tree_prim(&gr));

    assert!(mst.node_count() == 0);
    assert!(mst.edge_count() == 0);
}
