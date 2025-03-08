use petgraph::algo::label_propagation;
use petgraph::prelude::{Graph, NodeIndex};

#[test]
fn test_label_propagation() {
    let mut graph = Graph::<Option<&str>, ()>::new();
    graph.add_node(Some("A"));
    graph.add_node(None); // missing label
    graph.add_node(Some("A"));
    graph.add_node(Some("B"));
    graph.add_node(Some("C"));
    graph.add_node(Some("B"));
    graph.add_node(None); // missing label
    graph.extend_with_edges([(1, 0), (1, 3), (1, 5), (6, 0), (6, 4)]);
    //    (1, None) -- (0, A) -- (6, None) -- (4, C)
    //       / \
    //  (3, B) (5, B)        (2, A)
    let labels = vec![Some("A"), Some("B"), Some("C")];
    assert_eq!(
        std::collections::HashMap::from([
            (NodeIndex::new(1), Some("B")),
            (NodeIndex::new(6), Some("C"))
        ]),
        label_propagation(&graph, &labels, 1, 10)
    );
}
