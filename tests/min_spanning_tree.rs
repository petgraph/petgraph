use petgraph::{
    algo::{min_spanning_tree, min_spanning_tree_prim},
    dot::Dot,
    graph::{NodeIndex, UnGraph},
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

#[test]
fn mst_kruskal_test_cases() {
    use petgraph::data::FromElements;

    for (edges, expected_mst_edges) in TEST_CASES {
        let mut gr = UnGraph::new_undirected();
        gr.extend_with_edges(edges.to_vec());

        let mst: Graph<(), u32, Undirected, u32> = UnGraph::from_elements(min_spanning_tree(&gr));

        assert!(mst.node_count() == gr.node_count());
        assert!(mst.edge_count() == expected_mst_edges.len());

        for (source, target, _) in expected_mst_edges {
            let a = NodeIndex::new(*source as usize);
            let b = NodeIndex::new(*target as usize);
            assert!(mst.contains_edge(a, b));
        }
    }
}

#[test]
fn mst_prim_test_cases() {
    use petgraph::data::FromElements;

    for (edges, expected_mst_edges) in TEST_CASES {
        let mut gr = UnGraph::new_undirected();
        gr.extend_with_edges(edges.to_vec());

        let mst: Graph<(), u32, Undirected, u32> =
            UnGraph::from_elements(min_spanning_tree_prim(&gr));

        assert!(mst.node_count() == gr.node_count());
        assert!(mst.edge_count() == expected_mst_edges.len());

        for (source, target, _) in expected_mst_edges {
            let a = NodeIndex::new(*source as usize);
            let b = NodeIndex::new(*target as usize);
            assert!(mst.contains_edge(a, b));
        }
    }
}

// Test cases format: (graph order, graph edges, mst edges)
#[rustfmt::skip]
#[allow(clippy::type_complexity)]
const TEST_CASES: [(&[(u32, u32, u32)], &[(u32, u32, u32)]); 4] = [
    // Order 9
    (&[(0, 2, 107), (0, 3, 24), (0, 4, 47), (0, 5, 60), (0, 6, 98), (0, 7, 29), (0, 8, 20), (1, 2, 62), (1, 3, 115), (1, 6, 42), (1, 7, 117), (1, 8, 19), (2, 3, 39), (2, 4, 12), (2, 5, 27), (2, 7, 3), (2, 8, 66), (3, 4, 54), (3, 5, 129), (3, 6, 18), (3, 7, 137), (3, 8, 120), (4, 5, 9), (4, 6, 124), (4, 7, 103), (4, 8, 7), (5, 7, 77), (5, 8, 63), (6, 7, 130), (7, 8, 138)],
    &[(2, 7, 3), (4, 8, 7), (4, 5, 9), (2, 4, 12), (3, 6, 18), (1, 8, 19), (0, 8, 20), (0, 3, 24)]),
    // Order 10
    (&[(0, 2, 84), (0, 4, 5), (0, 5, 17), (0, 6, 97), (0, 7, 74), (0, 8, 16), (1, 3, 49), (1, 4, 28), (1, 8, 51), (1, 9, 137), (2, 3, 125), (2, 4, 87), (2, 6, 114), (2, 8, 131), (3, 4, 136), (3, 5, 43), (3, 8, 24), (3, 9, 112), (4, 5, 61), (4, 7, 99), (4, 8, 63), (4, 9, 108), (5, 6, 13), (5, 7, 9), (5, 8, 133), (6, 9, 147), (7, 8, 10), (7, 9, 88), (8, 9, 68)],
    &[(0, 4, 5), (5, 7, 9), (7, 8, 10), (5, 6, 13), (0, 8, 16), (3, 8, 24), (1, 4, 28), (8, 9, 68), (0, 2, 84)]),
    // Order 15
    (&[(0, 1, 124), (0, 3, 126), (0, 6, 84), (0, 7, 87), (0, 9, 93), (0, 12, 32), (1, 2, 51), (1, 4, 144), (1, 6, 36), (1, 8, 46), (1, 9, 8), (1, 13, 26), (2, 8, 111), (3, 4, 114), (3, 6, 98), (3, 8, 86), (3, 9, 73), (4, 5, 41), (4, 6, 7), (4, 8, 82), (4, 9, 48), (4, 10, 113), (4, 11, 54), (4, 12, 10), (5, 8, 60), (5, 13, 34), (6, 13, 85), (6, 14, 52), (7, 8, 74), (7, 12, 137), (8, 10, 118), (8, 12, 69), (8, 13, 133), (9, 12, 13), (10, 12, 65), (10, 14, 107), (11, 14, 102), (12, 13, 140), (12, 14, 11), (13, 14, 25)],
    &[(4, 6, 7), (1, 9, 8), (4, 12, 10), (12, 14, 11), (9, 12, 13), (13, 14, 25), (0, 12, 32), (5, 13, 34), (1, 8, 46), (1, 2, 51), (4, 11, 54), (10, 12, 65), (3, 9, 73), (7, 8, 74)]),
    // Order 20
    (&[(0, 2, 5), (0, 19, 73), (0, 12, 3), (1, 17, 145), (1, 18, 16), (1, 3, 125), (1, 5, 6), (1, 10, 76), (1, 11, 13), (1, 15, 12), (2, 7, 34), (2, 9, 118), (3, 12, 43), (3, 13, 146), (4, 7, 31), (5, 6, 62), (5, 8, 147), (6, 14, 66), (7, 16, 67), (7, 17, 48), (7, 10, 93), (7, 12, 113), (7, 14, 85), (8, 16, 40), (8, 18, 111), (9, 17, 102), (10, 16, 128), (10, 18, 120), (11, 17, 35), (11, 18, 88), (11, 13, 54), (11, 14, 36), (12, 16, 148), (13, 15, 75), (16, 17, 71), (16, 18, 10)],
    &[(0, 12, 3), (0, 2, 5), (1, 5, 6), (16, 18, 10), (1, 15, 12), (1, 11, 13), (1, 18, 16), (4, 7, 31), (2, 7, 34), (11, 17, 35), (11, 14, 36), (8, 16, 40), (3, 12, 43), (7, 17, 48), (11, 13, 54), (5, 6, 62), (0, 19, 73), (1, 10, 76), (9, 17, 102)]),
];
