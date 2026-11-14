// TODO: rework tests to be more encompassing and use test utils!

// #[test]
// fn next_node_id_pure() {
//     let mut storage = DinoGraph::<(), (), Directed>::new();

//     let a = storage.next_node_id();
//     let b = storage.next_node_id();

//     assert_eq!(a, b);

//     let node = storage.insert_node(a, ()).unwrap();
//     let node = node.id();

//     assert_eq!(node, a);

//     let c = storage.next_node_id();

//     assert_ne!(a, c);
// }

// #[test]
// fn next_edge_id_pure() {
//     let mut storage = DinoGraph::<(), (), Directed>::new();

//     let node = storage.insert_node(storage.next_node_id(), ()).unwrap();
//     let node = node.id();

//     let a = storage.next_edge_id();
//     let b = storage.next_edge_id();

//     assert_eq!(a, b);

//     let edge = storage.insert_edge(a, (), node, node).unwrap();
//     let edge = edge.id();

//     assert_eq!(edge, a);

//     let c = storage.next_edge_id();

//     assert_ne!(a, c);
// }

// #[test]
// fn remove_node() {
//     let mut graph = DinoGraph::<u8, (), Directed>::new();

//     let node = graph.try_insert_node(2u8).unwrap();
//     let node = node.id();

//     assert_eq!(graph.remove_node(node), Some(2u8));

//     assert_eq!(graph.num_nodes(), 0);
//     assert_eq!(graph.num_edges(), 0);

//     assert_eq!(graph.nodes().count(), 0);
//     assert_eq!(graph.edges().count(), 0);
// }

// #[test]
// fn remove_edge() {
//     let mut graph = DinoGraph::<(), u8, Directed>::new();

//     let node = graph.try_insert_node(()).unwrap();
//     let node = node.id();

//     let edge = graph.try_insert_edge(2u8, node, node).unwrap();
//     let edge = edge.id();

//     assert_eq!(graph.remove_edge(edge), Some(2u8));

//     assert_eq!(graph.num_nodes(), 1);
//     assert_eq!(graph.num_edges(), 0);

//     assert_eq!(graph.nodes().count(), 1);
//     assert_eq!(graph.edges().count(), 0);

//     assert_eq!(graph.connections(node).count(), 0);
//     assert_eq!(graph.neighbours(node).count(), 0);

//     assert_eq!(
//         graph
//             .connections_directed(node, Direction::Incoming)
//             .count(),
//         0
//     );
//     assert_eq!(
//         graph
//             .connections_directed(node, Direction::Outgoing)
//             .count(),
//         0
//     );

//     assert_eq!(
//         graph.neighbours_directed(node, Direction::Incoming).count(),
//         0
//     );
//     assert_eq!(
//         graph.neighbours_directed(node, Direction::Outgoing).count(),
//         0
//     );
// }

// #[test]
// fn clear() {
//     let mut graph = DinoGraph::<u8, u8, Directed>::new();

//     let node = graph.try_insert_node(2u8).unwrap();
//     let node = node.id();

//     graph.try_insert_edge(2u8, node, node).unwrap();

//     graph.clear();

//     assert_eq!(graph.num_nodes(), 0);
//     assert_eq!(graph.num_edges(), 0);

//     assert_eq!(graph.nodes().count(), 0);
//     assert_eq!(graph.edges().count(), 0);
// }
