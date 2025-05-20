#[cfg(feature = "stable_graph")]
#[cfg(test)]
use petgraph::{
    graph::{NodeIndex, UnGraph},
    Graph, Undirected,
};

#[cfg(feature = "stable_graph")]
#[cfg(test)]
fn b01_example() -> (UnGraph<(), i32>, Vec<NodeIndex>) {
    // Implementing b01 case from Vienna test set B
    let mut graph = Graph::<(), i32, Undirected>::new_undirected();
    let _n0 = graph.add_node(());
    let _n1 = graph.add_node(());
    let _n2 = graph.add_node(());
    let _n3 = graph.add_node(());
    let _n4 = graph.add_node(());
    let _n5 = graph.add_node(());
    let _n6 = graph.add_node(());
    let _n7 = graph.add_node(());
    let _n8 = graph.add_node(());
    let _n9 = graph.add_node(());
    let _n10 = graph.add_node(());

    let _n11 = graph.add_node(());
    let n12 = graph.add_node(());
    let _n13 = graph.add_node(());
    let _n14 = graph.add_node(());
    let _n15 = graph.add_node(());
    let _n16 = graph.add_node(());
    let _n17 = graph.add_node(());
    let _n18 = graph.add_node(());
    let _n19 = graph.add_node(());

    let _n20 = graph.add_node(());
    let _n21 = graph.add_node(());
    let n22 = graph.add_node(());
    let _n23 = graph.add_node(());
    let n24 = graph.add_node(());
    let _n25 = graph.add_node(());
    let _n26 = graph.add_node(());
    let n27 = graph.add_node(());
    let _n28 = graph.add_node(());
    let _n29 = graph.add_node(());

    let _n30 = graph.add_node(());
    let _n31 = graph.add_node(());
    let _n32 = graph.add_node(());
    let _n33 = graph.add_node(());
    let n34 = graph.add_node(());
    let n35 = graph.add_node(());
    let _n36 = graph.add_node(());
    let n37 = graph.add_node(());
    let _n38 = graph.add_node(());
    let _n39 = graph.add_node(());

    let _n40 = graph.add_node(());
    let _n41 = graph.add_node(());
    let _n42 = graph.add_node(());
    let _n43 = graph.add_node(());
    let _n44 = graph.add_node(());
    let _n45 = graph.add_node(());
    let _n46 = graph.add_node(());
    let _n47 = graph.add_node(());
    let n48 = graph.add_node(());
    let n49 = graph.add_node(());

    let _n50 = graph.add_node(());

    graph.extend_with_edges([
        (2, 8, 8),
        (2, 21, 7),
        (2, 32, 2),
        (4, 5, 8),
        (7, 29, 7),
        (11, 3, 7),
        (14, 31, 9),
        (17, 6, 7),
        (17, 42, 6),
        (18, 19, 2),
        (18, 28, 1),
        (18, 43, 1),
        (19, 2, 5),
        (20, 7, 3),
        (20, 14, 7),
        (20, 16, 8),
        (20, 27, 2),
        (20, 38, 8),
        (20, 40, 10),
        (20, 48, 2),
        (21, 12, 7),
        (21, 17, 5),
        (21, 18, 10),
        (22, 10, 6),
        (22, 20, 2),
        (22, 21, 2),
        (22, 40, 8),
        (22, 43, 7),
        (25, 34, 4),
        (27, 34, 4),
        (28, 5, 8),
        (28, 24, 5),
        (29, 9, 5),
        (29, 33, 7),
        (30, 5, 4),
        (30, 15, 1),
        (30, 16, 2),
        (33, 35, 3),
        (34, 20, 10),
        (34, 30, 2),
        (36, 2, 8),
        (36, 4, 6),
        (36, 11, 9),
        (36, 39, 7),
        (36, 49, 9),
        (36, 50, 10),
        (40, 15, 10),
        (40, 23, 3),
        (41, 1, 5),
        (41, 22, 8),
        (41, 25, 5),
        (41, 36, 2),
        (41, 44, 7),
        (41, 47, 7),
        (42, 6, 9),
        (42, 46, 10),
        (44, 24, 8),
        (44, 39, 3),
        (45, 26, 6),
        (45, 28, 1),
        (47, 37, 3),
        (47, 45, 10),
        (50, 13, 1),
    ]);

    let terminals = vec![n48, n49, n22, n35, n27, n12, n37, n34, n24];
    (graph, terminals)
}

#[cfg(feature = "stable_graph")]
#[cfg(test)]
fn b07_example() -> (UnGraph<(), i32>, Vec<NodeIndex>) {
    // Implementing b07 case from Vienna test set B
    let mut graph = Graph::<(), i32, Undirected>::new_undirected();
    let _n0 = graph.add_node(());
    let _n1 = graph.add_node(());
    let _n2 = graph.add_node(());
    let _n3 = graph.add_node(());
    let _n4 = graph.add_node(());
    let _n5 = graph.add_node(());
    let _n6 = graph.add_node(());
    let _n7 = graph.add_node(());
    let _n8 = graph.add_node(());
    let _n9 = graph.add_node(());
    let _n10 = graph.add_node(());

    let _n11 = graph.add_node(());
    let _n12 = graph.add_node(());
    let _n13 = graph.add_node(());
    let n14 = graph.add_node(());
    let _n15 = graph.add_node(());
    let _n16 = graph.add_node(());
    let _n17 = graph.add_node(());
    let _n18 = graph.add_node(());
    let _n19 = graph.add_node(());

    let _n20 = graph.add_node(());
    let _n21 = graph.add_node(());
    let _n22 = graph.add_node(());
    let n23 = graph.add_node(());
    let n24 = graph.add_node(());
    let _n25 = graph.add_node(());
    let n26 = graph.add_node(());
    let _n27 = graph.add_node(());
    let _n28 = graph.add_node(());
    let n29 = graph.add_node(());

    let _n30 = graph.add_node(());
    let _n31 = graph.add_node(());
    let _n32 = graph.add_node(());
    let _n33 = graph.add_node(());
    let _n34 = graph.add_node(());
    let _n35 = graph.add_node(());
    let n36 = graph.add_node(());
    let _n37 = graph.add_node(());
    let _n38 = graph.add_node(());
    let _n39 = graph.add_node(());

    let _n40 = graph.add_node(());
    let _n41 = graph.add_node(());
    let _n42 = graph.add_node(());
    let _n43 = graph.add_node(());
    let _n44 = graph.add_node(());
    let _n45 = graph.add_node(());
    let _n46 = graph.add_node(());
    let _n47 = graph.add_node(());
    let _n48 = graph.add_node(());
    let _n49 = graph.add_node(());

    let _n50 = graph.add_node(());
    let n51 = graph.add_node(());
    let n52 = graph.add_node(());
    let _n53 = graph.add_node(());
    let _n54 = graph.add_node(());
    let n55 = graph.add_node(());
    let _n56 = graph.add_node(());
    let _n57 = graph.add_node(());
    let _n58 = graph.add_node(());
    let n59 = graph.add_node(());

    let n60 = graph.add_node(());
    let _n61 = graph.add_node(());
    let _n62 = graph.add_node(());
    let n63 = graph.add_node(());
    let _n64 = graph.add_node(());
    let _n65 = graph.add_node(());
    let _n66 = graph.add_node(());
    let _n67 = graph.add_node(());
    let _n68 = graph.add_node(());
    let _n69 = graph.add_node(());

    let n70 = graph.add_node(());
    let _n71 = graph.add_node(());
    let _n72 = graph.add_node(());
    let _n73 = graph.add_node(());
    let _n74 = graph.add_node(());
    let _n75 = graph.add_node(());

    graph.extend_with_edges([
        (7, 17, 6),
        (7, 69, 1),
        (8, 25, 10),
        (8, 39, 1),
        (9, 70, 4),
        (15, 2, 3),
        (15, 20, 1),
        (18, 45, 2),
        (18, 74, 7),
        (19, 7, 9),
        (19, 64, 7),
        (22, 34, 5),
        (23, 9, 5),
        (25, 19, 7),
        (26, 72, 6),
        (27, 3, 10),
        (27, 36, 10),
        (27, 40, 3),
        (28, 6, 6),
        (28, 48, 7),
        (28, 63, 9),
        (29, 21, 4),
        (30, 29, 1),
        (30, 41, 8),
        (30, 62, 2),
        (32, 34, 3),
        (32, 74, 9),
        (33, 7, 10),
        (38, 7, 6),
        (38, 54, 8),
        (38, 60, 10),
        (38, 65, 2),
        (39, 2, 6),
        (40, 3, 10),
        (41, 1, 9),
        (41, 21, 7),
        (41, 23, 7),
        (42, 12, 2),
        (42, 30, 2),
        (42, 53, 3),
        (42, 56, 10),
        (42, 74, 1),
        (43, 4, 8),
        (43, 51, 9),
        (43, 54, 5),
        (43, 55, 6),
        (43, 71, 2),
        (44, 10, 1),
        (44, 26, 3),
        (44, 28, 9),
        (44, 36, 4),
        (44, 43, 8),
        (44, 46, 2),
        (44, 57, 2),
        (44, 68, 2),
        (45, 1, 6),
        (46, 49, 10),
        (46, 52, 2),
        (47, 27, 5),
        (47, 38, 5),
        (47, 41, 10),
        (48, 14, 2),
        (48, 22, 7),
        (50, 73, 1),
        (51, 8, 4),
        (51, 15, 7),
        (52, 16, 9),
        (53, 16, 10),
        (54, 66, 6),
        (56, 20, 6),
        (56, 75, 2),
        (57, 58, 9),
        (58, 5, 10),
        (59, 11, 9),
        (60, 18, 5),
        (60, 31, 5),
        (60, 35, 8),
        (60, 61, 1),
        (60, 67, 10),
        (61, 32, 9),
        (61, 37, 9),
        (63, 24, 10),
        (65, 16, 3),
        (65, 33, 2),
        (65, 42, 6),
        (65, 44, 4),
        (65, 50, 1),
        (65, 59, 1),
        (67, 20, 3),
        (67, 39, 6),
        (70, 13, 8),
        (71, 70, 4),
        (72, 9, 1),
        (74, 37, 4),
    ]);
    let terminals = vec![
        n55, n52, n60, n63, n24, n26, n70, n36, n29, n51, n23, n59, n14,
    ];
    (graph, terminals)
}

#[cfg(feature = "stable_graph")]
#[cfg(test)]
fn example_kou_paper() -> (UnGraph<(), usize>, Vec<NodeIndex>) {
    let mut graph = Graph::<(), usize, Undirected>::new_undirected();
    // Add nodes
    let nodes: Vec<_> = (0..9).map(|_| graph.add_node(())).collect();

    let edges = vec![
        (0, 1, 20),
        (0, 8, 1),
        (1, 2, 8),
        (1, 5, 1),
        (2, 4, 2),
        (2, 3, 9),
        (3, 4, 2),
        (4, 8, 1),
        (4, 4, 1),
        (5, 6, 1),
        (6, 7, 0), // Note: In the Kou Paper this is 0.5, but the nodes are not present in terminals, so we approximate this with 0
        (7, 8, 0), // Note: In the Kou Paper this is 0.5, but the nodes are not present in terminals, so we approximate this with 0
    ];

    // Add edges to the graph
    for (u, v, weight) in edges {
        graph.add_edge(nodes[u], nodes[v], weight);
    }

    (graph, vec![nodes[0], nodes[1], nodes[2], nodes[3]])
}

#[cfg(feature = "stable_graph")]
#[cfg(test)]
mod test {
    use crate::{b01_example, b07_example, example_kou_paper};
    use petgraph::algo::{connected_components, steiner_tree};
    use petgraph::graph::UnGraph;

    #[test]
    fn b01_vienna_test() {
        let (graph, terminals) = b01_example();
        let st = steiner_tree(&graph, &terminals);
        let weights = st.edge_weights().cloned().sum::<i32>();
        let steiner_tree_nodes: Vec<_> = st.node_indices().collect();
        assert!(terminals.iter().all(|&t| steiner_tree_nodes.contains(&t)));
        assert!(st.edge_count() == st.node_count() - 1);
        assert_eq!(connected_components(&UnGraph::from(st)), 1);
        assert_eq!(weights, 82);
    }

    #[test]
    fn b07_vienna_test() {
        let (graph, terminals) = b07_example();
        let st = steiner_tree(&graph, &terminals);

        let weights = st.edge_weights().cloned().sum::<i32>();
        let steiner_tree_nodes: Vec<_> = st.node_indices().collect();
        assert!(terminals.iter().all(|&t| steiner_tree_nodes.contains(&t)));
        assert!(st.edge_count() == st.node_count() - 1);
        assert_eq!(connected_components(&UnGraph::from(st)), 1);
        assert_eq!(weights, 111);
    }

    #[test]
    fn example_kous_paper() {
        let (graph, terminals) = example_kou_paper();
        let st = steiner_tree(&graph, &terminals);

        let weights = st.edge_weights().cloned().sum::<usize>();
        let steiner_tree_nodes: Vec<_> = st.node_indices().collect();
        assert!(terminals.iter().all(|&t| steiner_tree_nodes.contains(&t)));
        assert!(st.edge_count() == st.node_count() - 1);
        assert_eq!(connected_components(&UnGraph::from(st)), 1);
        assert_eq!(weights, 8);
    }
}
