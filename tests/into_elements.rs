extern crate petgraph;

use petgraph::*;
use petgraph::stable_graph::StableGraph;
use petgraph::into_elements::{IntoElements, rebuild};
use petgraph::data::Element;

#[test]
fn graph_into_elements() {
    let g = {
        let mut g = Graph::new();
        let n1 = g.add_node(1);
        let n2 = g.add_node(2);
        let n3 = g.add_node(3);
        g.add_edge(n1, n2, 'a');
        g.add_edge(n2, n3, 'b');
        g.add_edge(n1, n3, 'c');
        g.remove_node(n2);
        g
    };

    let elements = IntoElements(&g);
    let mut iter = elements.into_iter();
    assert_eq!(iter.next(), Some(Element::Node { weight: 1 }));
    assert_eq!(iter.next(), Some(Element::Node { weight: 3 }));
    assert_eq!(iter.next(),
               Some(Element::Edge {
                        source: 0,
                        target: 1,
                        weight: 'c',
                    }));

}

#[test]
fn stable_graph_into_elements() {
    let g = {
        let mut g = StableGraph::new();
        let n1 = g.add_node(1);
        let n2 = g.add_node(2);
        let n3 = g.add_node(3);
        g.add_edge(n1, n2, 'a');
        g.add_edge(n2, n3, 'b');
        g.add_edge(n1, n3, 'c');
        g.remove_node(n2);
        g
    };

    let elements = IntoElements(&g);
    let mut iter = elements.into_iter();
    assert_eq!(iter.next(), Some(Element::Node { weight: 1 }));
    assert_eq!(iter.next(), Some(Element::Node { weight: 3 }));
    assert_eq!(iter.next(),
               Some(Element::Edge {
                        source: 0,
                        target: 1,
                        weight: 'c',
                    }));

}

#[test]
fn rebuild_stable_graph_as_graph() {
    let g: StableGraph<u8, _> = {
        let mut g = StableGraph::new();
        let n1 = g.add_node(1);
        let n2 = g.add_node(2);
        let n3 = g.add_node(3);
        g.add_edge(n1, n2, 'a');
        g.add_edge(n2, n3, 'b');
        g.add_edge(n1, n3, 'c');
        g.remove_node(n2);
        g
    };
    let stable: StableGraph<_, _> = rebuild(&g);
    assert!(IntoElements(&stable).into_iter().eq(IntoElements(&g)));
    let gg: Graph<_, _> = rebuild(&stable);
    assert!(IntoElements(&stable).into_iter().eq(IntoElements(&gg)));

}
