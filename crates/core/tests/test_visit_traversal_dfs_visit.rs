use petgraph::Graph;
use petgraph_core::visit::{depth_first_search, Control, DfsEvent, Time};

#[test]
fn simple() {
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    graph.add_edge(a, b, "A → B");
    graph.add_edge(b, c, "B → C");
    graph.add_edge(c, a, "C → A");

    // TODO: think about moving this to an iterator instead
    depth_first_search(&graph, Some(a), |event| match event {
        DfsEvent::Discover(node, time) => {
            match node {
                _ if node == a => assert_eq!(time, Time(0)),
                _ if node == b => assert_eq!(time, Time(1)),
                _ if node == c => assert_eq!(time, Time(2)),
                _ => panic!("Unexpected node: {:?}", node),
            };
        }
        DfsEvent::TreeEdge(start, end) => {
            match (start, end) {
                (start, end) if start == a && end == b => {}
                (start, end) if start == b && end == c => {}
                (start, end) => panic!("Unexpected edge: {:?} → {:?}", start, end),
            };
        }
        DfsEvent::BackEdge(start, end) => {
            match (start, end) {
                (start, end) if start == c && end == a => {}
                (start, end) => panic!("Unexpected edge: {:?} → {:?}", start, end),
            };
        }
        DfsEvent::CrossForwardEdge(..) => panic!("Unexpected event: {:?}", event),
        DfsEvent::Finish(node, time) => {
            match node {
                _ if node == a => assert_eq!(time, Time(5)),
                _ if node == b => assert_eq!(time, Time(4)),
                _ if node == c => assert_eq!(time, Time(3)),
                _ => panic!("Unexpected node: {:?}", node),
            };
        }
    });
}

#[test]
fn terminate_early() {
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    graph.add_edge(a, b, "A → B");
    graph.add_edge(b, c, "B → C");
    graph.add_edge(c, a, "C → A");

    let mut predecessor = vec![None; graph.node_count()];
    let control = depth_first_search(&graph, Some(a), |event| {
        if let DfsEvent::TreeEdge(start, end) = event {
            predecessor[end.index()] = Some(start);
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
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    graph.add_edge(a, b, "A → B");
    graph.add_edge(b, c, "B → C");
    graph.add_edge(a, c, "A → C");

    depth_first_search(&graph, Some(a), |event| {
        if let DfsEvent::CrossForwardEdge(start, end) = event {
            match (start, end) {
                (start, end) if start == b && end == c => {}
                (start, end) => panic!("Unexpected edge: {:?} → {:?}", start, end),
            };
        }
    });
}

#[test]
fn prune() {
    let mut graph = Graph::new();

    let a = graph.add_node("A");
    let b = graph.add_node("B");
    let c = graph.add_node("C");

    graph.add_edge(a, b, "A → B");
    graph.add_edge(b, c, "B → C");

    depth_first_search(&graph, Some(a), |event| {
        if let DfsEvent::Discover(node, _) = event {
            if node == b {
                return Control::<()>::Prune;
            }
        }

        if let DfsEvent::TreeEdge(start, end) = event {
            assert!(end != c, "Unexpected edge: {start:?} → {end:?}");
        }

        Control::Continue
    });
}
