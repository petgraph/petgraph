use std::iter::once;

use petgraph_core::{
    deprecated::visit::{depth_first_search, Control, DfsEvent, Time},
    id::{IndexMapper, LinearGraphId},
};
use petgraph_dino::{DiDinoGraph, NodeId};

#[test]
fn simple() {
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();

    graph.insert_edge("A → B", a, b);
    graph.insert_edge("B → C", b, c);
    graph.insert_edge("C → A", c, a);

    // TODO: think about moving this to an iterator instead
    depth_first_search(&graph, Some(a), |event| match event {
        DfsEvent::Discover(node, time) => {
            match node {
                _ if node == a => assert_eq!(time, Time(0)),
                _ if node == b => assert_eq!(time, Time(1)),
                _ if node == c => assert_eq!(time, Time(2)),
                _ => panic!("Unexpected node: {node}"),
            };
        }
        DfsEvent::TreeEdge(start, end) => {
            match (start, end) {
                (start, end) if start == a && end == b => {}
                (start, end) if start == b && end == c => {}
                (start, end) => panic!("Unexpected edge: {start} → {end}"),
            };
        }
        DfsEvent::BackEdge(start, end) => {
            match (start, end) {
                (start, end) if start == c && end == a => {}
                (start, end) => panic!("Unexpected edge: {start} → {end}"),
            };
        }
        DfsEvent::CrossForwardEdge(..) => panic!("Unexpected event: {event:?}"),
        DfsEvent::Finish(node, time) => {
            match node {
                _ if node == a => assert_eq!(time, Time(5)),
                _ if node == b => assert_eq!(time, Time(4)),
                _ if node == c => assert_eq!(time, Time(3)),
                _ => panic!("Unexpected node: {node}"),
            };
        }
    });
}

#[test]
fn terminate_early() {
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();

    graph.insert_edge("A → B", a, b);
    graph.insert_edge("B → C", b, c);
    graph.insert_edge("C → A", c, a);

    let mut mapper = NodeId::index_mapper(graph.storage());

    let mut predecessor = vec![None; graph.num_nodes()];
    let control = depth_first_search(&graph, once(a), |event| {
        if let DfsEvent::TreeEdge(start, end) = event {
            predecessor[mapper.index(end)] = Some(start);
            if end == b {
                return Control::Break(start);
            }
        }

        Control::Continue
    });

    assert!(matches!(control, Control::Break(start) if start == a));
    assert_eq!(predecessor, vec![None, Some(a), None]);
}

#[test]
fn cross_forward_edge() {
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();

    graph.insert_edge("A → B", a, b);
    graph.insert_edge("B → C", b, c);
    graph.insert_edge("A → C", a, c);

    depth_first_search(&graph, once(a), |event| {
        if let DfsEvent::CrossForwardEdge(start, end) = event {
            match (start, end) {
                (start, end) if start == b && end == c => {}
                (start, end) => panic!("Unexpected edge: {start} → {end}"),
            };
        }
    });
}

#[test]
fn prune() {
    let mut graph = DiDinoGraph::new();

    let a = graph.insert_node("A").id();
    let b = graph.insert_node("B").id();
    let c = graph.insert_node("C").id();

    graph.insert_edge("A → B", a, b);
    graph.insert_edge("B → C", b, c);

    depth_first_search(&graph, once(a), |event| {
        if let DfsEvent::Discover(node, _) = event {
            if node == b {
                return Control::<()>::Prune;
            }
        }

        if let DfsEvent::TreeEdge(start, end) = event {
            assert_ne!(end, c, "Unexpected edge: {start:?} → {end:?}");
        }

        Control::Continue
    });
}

// TODO: disabled for now because `proptest` isn't ported yet
// #[cfg(not(miri))]
// proptest! {
//     #[test]
//     fn discover_all_directed(graph in any::<Graph<(), (), Directed, u8>>()) {
//         let mut discovered = vec![false; graph.node_count()];
//
//         depth_first_search(&graph, graph.node_indices(), |event| {
//             if let DfsEvent::Discover(node, _) = event {
//                 discovered[node.index()] = true;
//             }
//         });
//
//         assert!(discovered.iter().all(|&discovered| discovered));
//     }
//
//     #[test]
//     fn discover_all_undirected(graph in any::<Graph<(), (), Undirected, u8>>()) {
//         let mut discovered = vec![false; graph.node_count()];
//
//         depth_first_search(&graph, graph.node_indices(), |event| {
//             if let DfsEvent::Discover(node, _) = event {
//                 discovered[node.index()] = true;
//             }
//         });
//
//         assert!(discovered.iter().all(|&discovered| discovered));
//     }
//
//     #[test]
//     fn finish_all_directed(graph in any::<Graph<(), (), Directed, u8>>()) {
//         let mut finished = vec![false; graph.node_count()];
//
//         depth_first_search(&graph, graph.node_indices(), |event| {
//             if let DfsEvent::Finish(node, _) = event {
//                 finished[node.index()] = true;
//             }
//         });
//
//         assert!(finished.iter().all(|&finished| finished));
//     }
//
//     #[test]
//     fn finish_all_undirected(graph in any::<Graph<(), (), Undirected, u8>>()) {
//         let mut finished = vec![false; graph.node_count()];
//
//         depth_first_search(&graph, graph.node_indices(), |event| {
//             if let DfsEvent::Finish(node, _) = event {
//                 finished[node.index()] = true;
//             }
//         });
//
//         assert!(finished.iter().all(|&finished| finished));
//     }
//
//     #[test]
//     fn tree_edges_directed(graph in any::<Graph<(), (), Directed, u8>>()) {
//         let mut discovered = vec![false; graph.node_count()];
//         let mut finished = vec![false; graph.node_count()];
//
//         depth_first_search(&graph, graph.node_indices(), |event| {
//             if let DfsEvent::Discover(node, _) = event {
//                 discovered[node.index()] = true;
//             }
//
//             if let DfsEvent::Finish(node, _) = event {
//                 finished[node.index()] = true;
//             }
//
//             // end is the ancestor of start, we have already discovered start,
//             // but haven't discovered end
//             if let DfsEvent::TreeEdge(start, end) = event {
//                 assert!(graph.find_edge(start, end).is_some());
//
//                 assert!(discovered[start.index()]);
//                 assert!(!discovered[end.index()]);
//
//                 assert!(!finished[start.index()]);
//             }
//         });
//     }
//
//     #[test]
//     fn tree_edges_undirected(graph in any::<Graph<(), (), Undirected, u8>>()) {
//         let mut discovered = vec![false; graph.node_count()];
//         let mut finished = vec![false; graph.node_count()];
//
//         depth_first_search(&graph, graph.node_indices(), |event| {
//             if let DfsEvent::Discover(node, _) = event {
//                 discovered[node.index()] = true;
//             }
//
//             if let DfsEvent::Finish(node, _) = event {
//                 finished[node.index()] = true;
//             }
//
//             // end is the ancestor of start, we have already discovered start,
//             // but haven't discovered end
//             if let DfsEvent::TreeEdge(start, end) = event {
//                 assert!(graph.find_edge(start, end).is_some());
//
//                 assert!(discovered[start.index()]);
//                 assert!(!discovered[end.index()]);
//
//                 assert!(!finished[start.index()]);
//             }
//         });
//     }
//
//     #[test]
//     fn edges_directed(graph in any::<Graph<(), (), Directed, u8>>()) {
//         let mut edges = HashSet::new();
//
//         depth_first_search(&graph, graph.node_indices(), |event| {
//             if let DfsEvent::TreeEdge(start, end) = event {
//                 edges.insert((start, end));
//             }
//
//             if let DfsEvent::BackEdge(start, end) = event {
//                 edges.insert((start, end));
//             }
//
//             if let DfsEvent::CrossForwardEdge(start, end) = event {
//                 edges.insert((start, end));
//             }
//         });
//
//         let expected: HashSet<_> = graph.edge_references().map(|e| (e.source(),
// e.target())).collect();         assert_eq!(edges, expected);
//     }
//
//     #[test]
//     fn edges_undirected(graph in any::<Graph<(), (), Undirected, u8>>()) {
//         let mut edges = HashSet::new();
//
//         depth_first_search(&graph, graph.node_indices(), |event| {
//             if let DfsEvent::TreeEdge(start, end) = event {
//                 edges.insert((start, end));
//             }
//
//             if let DfsEvent::BackEdge(start, end) = event {
//                 edges.insert((start, end));
//             }
//
//             if let DfsEvent::CrossForwardEdge(start, end) = event {
//                 edges.insert((start, end));
//             }
//         });
//
//         for edge in edges {
//             assert!(graph.find_edge(edge.0, edge.1).is_some() || graph.find_edge(edge.1,
// edge.0).is_some());         }
//     }
// }
