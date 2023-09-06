use core::iter::once;

use hashbrown::HashSet;
use petgraph_core::{
    attributes::NoValue,
    edge::{marker::Directed, DetachedEdge, Direction},
    node::{DetachedNode, Node, NodeMut},
    storage::GraphStorage,
};

use crate::{DinoGraph, DinosaurStorage, EdgeId, NodeId};

#[test]
fn empty() {
    let graph = DinoGraph::<(), (), Directed>::new();

    assert_eq!(graph.num_nodes(), 0);
    assert_eq!(graph.num_edges(), 0);

    assert_eq!(graph.nodes().count(), 0);
    assert_eq!(graph.edges().count(), 0);
}

#[test]
fn insert_node() {
    let mut graph = DinoGraph::<u8, (), Directed>::new();

    let node = graph.insert_node(2u8).unwrap();

    assert_eq!(node.weight(), &2u8);

    assert_eq!(graph.num_nodes(), 1);
    assert_eq!(graph.num_edges(), 0);

    assert_eq!(graph.nodes().count(), 1);
    assert_eq!(graph.edges().count(), 0);
}

#[test]
fn insert_edge() {
    let mut graph = DinoGraph::<(), u8, Directed>::new();

    let node = graph.insert_node(()).unwrap();
    let node = *node.id();

    let edge = graph.insert_edge(2u8, &node, &node).unwrap();

    assert_eq!(edge.weight(), &2u8);

    assert_eq!(graph.num_nodes(), 1);
    assert_eq!(graph.num_edges(), 1);

    assert_eq!(graph.nodes().count(), 1);
    assert_eq!(graph.edges().count(), 1);
}

#[test]
fn next_node_id_pure() {
    let mut storage = DinosaurStorage::<(), (), Directed>::new();

    let a = storage.next_node_id(NoValue::new());
    let b = storage.next_node_id(NoValue::new());

    assert_eq!(a, b);

    let node = storage.insert_node(a, ()).unwrap();
    let node = *node.id();

    assert_eq!(node, a);

    let c = storage.next_node_id(NoValue::new());

    assert_ne!(a, c);
}

#[test]
fn next_edge_id_pure() {
    let mut storage = DinosaurStorage::<(), (), Directed>::new();

    let node = storage
        .insert_node(storage.next_node_id(NoValue::new()), ())
        .unwrap();
    let node = *node.id();

    let a = storage.next_edge_id(NoValue::new());
    let b = storage.next_edge_id(NoValue::new());

    assert_eq!(a, b);

    let edge = storage.insert_edge(a, (), &node, &node).unwrap();
    let edge = *edge.id();

    assert_eq!(edge, a);

    let c = storage.next_edge_id(NoValue::new());

    assert_ne!(a, c);
}

#[test]
fn remove_node() {
    let mut graph = DinoGraph::<u8, (), Directed>::new();

    let node = graph.insert_node(2u8).unwrap();
    let node = *node.id();

    assert_eq!(graph.remove_node(&node), Some(DetachedNode::new(node, 2u8)));

    assert_eq!(graph.num_nodes(), 0);
    assert_eq!(graph.num_edges(), 0);

    assert_eq!(graph.nodes().count(), 0);
    assert_eq!(graph.edges().count(), 0);
}

#[test]
fn remove_edge() {
    let mut graph = DinoGraph::<(), u8, Directed>::new();

    let node = graph.insert_node(()).unwrap();
    let node = *node.id();

    let edge = graph.insert_edge(2u8, &node, &node).unwrap();
    let edge = *edge.id();

    assert_eq!(
        graph.remove_edge(&edge),
        Some(DetachedEdge::new(edge, 2u8, node, node))
    );

    assert_eq!(graph.num_nodes(), 1);
    assert_eq!(graph.num_edges(), 0);

    assert_eq!(graph.nodes().count(), 1);
    assert_eq!(graph.edges().count(), 0);

    assert_eq!(graph.connections(&node).count(), 0);
    assert_eq!(graph.neighbours(&node).count(), 0);

    assert_eq!(
        graph
            .connections_directed(&node, Direction::Incoming)
            .count(),
        0
    );
    assert_eq!(
        graph
            .connections_directed(&node, Direction::Outgoing)
            .count(),
        0
    );

    assert_eq!(
        graph
            .neighbours_directed(&node, Direction::Incoming)
            .count(),
        0
    );
    assert_eq!(
        graph
            .neighbours_directed(&node, Direction::Outgoing)
            .count(),
        0
    );
}

#[test]
fn clear() {
    let mut graph = DinoGraph::<u8, u8, Directed>::new();

    let node = graph.insert_node(2u8).unwrap();
    let node = *node.id();

    graph.insert_edge(2u8, &node, &node).unwrap();

    graph.clear().unwrap();

    assert_eq!(graph.num_nodes(), 0);
    assert_eq!(graph.num_edges(), 0);

    assert_eq!(graph.nodes().count(), 0);
    assert_eq!(graph.edges().count(), 0);
}

#[test]
fn node() {
    let mut graph = DinoGraph::<u8, (), Directed>::new();

    let node = graph.insert_node(2u8).unwrap();
    let node = *node.id();

    assert_eq!(
        graph.node(&node),
        Some(Node::new(graph.storage(), &node, &2u8))
    );
}

#[test]
fn node_mut() {
    let mut graph = DinoGraph::<u8, (), Directed>::new();

    let node = graph.insert_node(2u8).unwrap();
    let node = *node.id();

    assert_eq!(graph.node_mut(&node), Some(NodeMut::new(&node, &mut 2u8)));

    let mut node = graph.node_mut(&node).unwrap();
    *node.weight_mut() = 3u8;
    let node = *node.id();

    assert_eq!(
        graph.node(&node),
        Some(Node::new(graph.storage(), &node, &3u8))
    );
}

#[test]
fn contains_node() {
    let mut graph = DinoGraph::<u8, (), Directed>::new();

    let node = graph.insert_node(2u8).unwrap();
    let node = *node.id();

    assert!(graph.storage().contains_node(&node));
}

#[test]
fn edge() {
    let mut graph = DinoGraph::<(), u8, Directed>::new();

    let node = graph.insert_node(()).unwrap();
    let node = *node.id();

    let edge = graph.insert_edge(2u8, &node, &node).unwrap();
    let edge = *edge.id();

    assert_eq!(
        graph.edge(&edge),
        Some(petgraph_core::edge::Edge::new(
            graph.storage(),
            &edge,
            &2u8,
            &node,
            &node
        ))
    );
}

#[test]
fn edge_mut() {
    let mut graph = DinoGraph::<(), u8, Directed>::new();

    let node = graph.insert_node(()).unwrap();
    let node = *node.id();

    let edge = graph.insert_edge(2u8, &node, &node).unwrap();
    let edge = *edge.id();

    assert_eq!(
        graph.edge_mut(&edge),
        Some(petgraph_core::edge::EdgeMut::new(
            &edge, &mut 2u8, &node, &node
        ))
    );

    let mut edge = graph.edge_mut(&edge).unwrap();
    *edge.weight_mut() = 3u8;
    let edge = *edge.id();

    assert_eq!(
        graph.edge(&edge),
        Some(petgraph_core::edge::Edge::new(
            graph.storage(),
            &edge,
            &3u8,
            &node,
            &node
        ))
    );
}

#[test]
fn contains_edge() {
    let mut graph = DinoGraph::<(), u8, Directed>::new();

    let node = graph.insert_node(()).unwrap();
    let node = *node.id();

    let edge = graph.insert_edge(2u8, &node, &node).unwrap();
    let edge = *edge.id();

    assert!(graph.storage().contains_edge(&edge));
}

struct SimpleGraph {
    graph: DinoGraph<u8, u8, Directed>,

    a: NodeId,
    b: NodeId,
    c: NodeId,
    d: NodeId,

    ab: EdgeId,
    ba: EdgeId,
    bc: EdgeId,
    ac: EdgeId,
    ca: EdgeId,
}

impl SimpleGraph {
    fn create() -> Self {
        let mut graph = DinoGraph::new();

        let a = graph.insert_node(1u8).unwrap();
        let a = *a.id();

        let b = graph.insert_node(2u8).unwrap();
        let b = *b.id();

        let c = graph.insert_node(3u8).unwrap();
        let c = *c.id();

        let d = graph.insert_node(8u8).unwrap();
        let d = *d.id();

        let ab = graph.insert_edge(4u8, &a, &b).unwrap();
        let ab = *ab.id();

        let ba = graph.insert_edge(5u8, &b, &a).unwrap();
        let ba = *ba.id();

        let bc = graph.insert_edge(6u8, &b, &c).unwrap();
        let bc = *bc.id();

        let ac = graph.insert_edge(7u8, &a, &c).unwrap();
        let ac = *ac.id();

        let ca = graph.insert_edge(8u8, &c, &a).unwrap();
        let ca = *ca.id();

        Self {
            graph,
            a,
            b,
            c,
            d,
            ab,
            ba,
            bc,
            ac,
            ca,
        }
    }
}

#[test]
fn find_undirected_edges() {
    let SimpleGraph {
        graph,
        a,
        b,
        ab,
        ba,
        ..
    } = SimpleGraph::create();

    assert_eq!(
        graph
            .find_undirected_edges(&a, &b)
            .map(petgraph_core::edge::Edge::detach)
            .collect::<HashSet<_>>(),
        [
            DetachedEdge::new(ab, 4u8, a, b),
            DetachedEdge::new(ba, 5u8, b, a),
        ]
        .into_iter()
        .collect::<HashSet<_>>()
    );
}

// TODO: find_undirected_edges_mut?

#[test]
fn node_connections() {
    let SimpleGraph {
        graph,
        a,
        b,
        c,
        ab,
        ba,
        ac,
        ca,
        ..
    } = SimpleGraph::create();

    assert_eq!(
        graph
            .connections(&a)
            .map(petgraph_core::edge::Edge::detach)
            .collect::<HashSet<_>>(),
        [
            DetachedEdge::new(ab, 4u8, a, b),
            DetachedEdge::new(ba, 5u8, b, a),
            DetachedEdge::new(ac, 7u8, a, c),
            DetachedEdge::new(ca, 8u8, c, a),
        ]
        .into_iter()
        .collect::<HashSet<_>>()
    );
}

#[test]
fn node_connections_mut() {
    let SimpleGraph {
        mut graph,
        a,
        b,
        c,
        ab,
        ba,
        ac,
        ca,
        bc,
        ..
    } = SimpleGraph::create();
    for mut connection in graph.connections_mut(&a) {
        *connection.weight_mut() += 1;
    }

    assert_eq!(
        graph
            .connections(&a)
            .map(petgraph_core::edge::Edge::detach)
            .collect::<HashSet<_>>(),
        [
            DetachedEdge::new(ab, 5u8, a, b),
            DetachedEdge::new(ba, 6u8, b, a),
            DetachedEdge::new(ac, 8u8, a, c),
            DetachedEdge::new(ca, 9u8, c, a),
        ]
        .into_iter()
        .collect::<HashSet<_>>()
    );

    assert_eq!(
        graph.edge(&bc),
        Some(petgraph_core::edge::Edge::new(
            graph.storage(),
            &bc,
            &6u8,
            &b,
            &c
        ))
    );
}

#[test]
fn node_neighbours() {
    let SimpleGraph {
        graph, a, b, c, d, ..
    } = SimpleGraph::create();

    assert_eq!(
        graph
            .neighbours(&a)
            .map(Node::detach)
            .collect::<HashSet<_>>(),
        [DetachedNode::new(b, 2u8), DetachedNode::new(c, 3u8)]
            .into_iter()
            .collect::<HashSet<_>>()
    );

    assert_eq!(
        graph
            .neighbours(&b)
            .map(Node::detach)
            .collect::<HashSet<_>>(),
        [DetachedNode::new(a, 1u8), DetachedNode::new(c, 3u8)]
            .into_iter()
            .collect::<HashSet<_>>()
    );

    assert_eq!(
        graph
            .neighbours(&c)
            .map(Node::detach)
            .collect::<HashSet<_>>(),
        [DetachedNode::new(a, 1u8), DetachedNode::new(b, 2u8)]
            .into_iter()
            .collect::<HashSet<_>>()
    );

    assert_eq!(
        graph
            .neighbours(&d)
            .map(Node::detach)
            .collect::<HashSet<_>>(),
        HashSet::new()
    );
}

#[test]
fn node_neighbours_mut() {
    let SimpleGraph {
        mut graph,
        a,
        b,
        c,
        d,
        ..
    } = SimpleGraph::create();

    for mut neighbour in graph.neighbours_mut(&a) {
        *neighbour.weight_mut() += 1;
    }

    assert_eq!(
        graph
            .neighbours(&a)
            .map(Node::detach)
            .collect::<HashSet<_>>(),
        [DetachedNode::new(b, 3u8), DetachedNode::new(c, 4u8)]
            .into_iter()
            .collect::<HashSet<_>>()
    );

    assert_eq!(
        graph
            .neighbours(&b)
            .map(Node::detach)
            .collect::<HashSet<_>>(),
        [DetachedNode::new(a, 1u8), DetachedNode::new(c, 4u8)]
            .into_iter()
            .collect::<HashSet<_>>()
    );

    assert_eq!(
        graph
            .neighbours(&c)
            .map(Node::detach)
            .collect::<HashSet<_>>(),
        [DetachedNode::new(a, 1u8), DetachedNode::new(b, 3u8)]
            .into_iter()
            .collect::<HashSet<_>>()
    );

    assert_eq!(
        graph
            .neighbours(&d)
            .map(Node::detach)
            .collect::<HashSet<_>>(),
        HashSet::new()
    );
}

#[test]
fn external_nodes() {
    let SimpleGraph { graph, d, .. } = SimpleGraph::create();

    assert_eq!(
        graph.externals().map(Node::detach).collect::<HashSet<_>>(),
        once(DetachedNode::new(d, 8u8)).collect::<HashSet<_>>()
    );
}

#[test]
fn external_nodes_mut() {
    let SimpleGraph {
        mut graph,
        a,
        b,
        c,
        d,
        ..
    } = SimpleGraph::create();

    for mut external in graph.externals_mut() {
        *external.weight_mut() += 1;
    }

    assert_eq!(
        graph.externals().map(Node::detach).collect::<HashSet<_>>(),
        once(DetachedNode::new(d, 9u8)).collect::<HashSet<_>>()
    );

    assert_eq!(graph.node(&a), Some(Node::new(graph.storage(), &a, &1u8)));

    assert_eq!(graph.node(&b), Some(Node::new(graph.storage(), &b, &2u8)));

    assert_eq!(graph.node(&c), Some(Node::new(graph.storage(), &c, &3u8)));
}

#[test]
fn nodes() {
    let SimpleGraph {
        graph, a, b, c, d, ..
    } = SimpleGraph::create();

    assert_eq!(
        graph.nodes().map(Node::detach).collect::<HashSet<_>>(),
        [
            DetachedNode::new(a, 1u8),
            DetachedNode::new(b, 2u8),
            DetachedNode::new(c, 3u8),
            DetachedNode::new(d, 8u8),
        ]
        .into_iter()
        .collect::<HashSet<_>>()
    );
}

#[test]
fn nodes_mut() {
    let SimpleGraph {
        mut graph,
        a,
        b,
        c,
        d,
        ..
    } = SimpleGraph::create();

    for mut node in graph.nodes_mut() {
        *node.weight_mut() += 1;
    }

    assert_eq!(
        graph.nodes().map(Node::detach).collect::<HashSet<_>>(),
        [
            DetachedNode::new(a, 2u8),
            DetachedNode::new(b, 3u8),
            DetachedNode::new(c, 4u8),
            DetachedNode::new(d, 9u8),
        ]
        .into_iter()
        .collect::<HashSet<_>>()
    );
}

#[test]
fn edges() {
    let SimpleGraph {
        graph,
        a,
        b,
        c,
        ab,
        ba,
        bc,
        ac,
        ca,
        ..
    } = SimpleGraph::create();

    assert_eq!(
        graph
            .edges()
            .map(petgraph_core::edge::Edge::detach)
            .collect::<HashSet<_>>(),
        [
            DetachedEdge::new(ab, 4u8, a, b),
            DetachedEdge::new(ba, 5u8, b, a),
            DetachedEdge::new(bc, 6u8, b, c),
            DetachedEdge::new(ac, 7u8, a, c),
            DetachedEdge::new(ca, 8u8, c, a),
        ]
        .into_iter()
        .collect::<HashSet<_>>()
    );
}

#[test]
fn edges_mut() {
    let SimpleGraph {
        mut graph,
        a,
        b,
        c,
        ab,
        ba,
        bc,
        ac,
        ca,
        ..
    } = SimpleGraph::create();

    for mut edge in graph.edges_mut() {
        *edge.weight_mut() += 1;
    }

    assert_eq!(
        graph
            .edges()
            .map(petgraph_core::edge::Edge::detach)
            .collect::<HashSet<_>>(),
        [
            DetachedEdge::new(ab, 5u8, a, b),
            DetachedEdge::new(ba, 6u8, b, a),
            DetachedEdge::new(bc, 7u8, b, c),
            DetachedEdge::new(ac, 8u8, a, c),
            DetachedEdge::new(ca, 9u8, c, a),
        ]
        .into_iter()
        .collect::<HashSet<_>>()
    );
}
