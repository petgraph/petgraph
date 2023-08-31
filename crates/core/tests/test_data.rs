#![cfg(feature = "alloc")]

use petgraph_core::deprecated::data::{Element, ElementIterator};

#[test]
fn filter_elements_node() {
    let elements = [
        Element::Node { weight: "A" },
        Element::Node { weight: "B" },
        Element::Node { weight: "C" },
        Element::Edge {
            source: 0,
            target: 1,
            weight: 7,
        },
        Element::Edge {
            source: 2,
            target: 0,
            weight: 9,
        },
    ];

    let filtered = elements
        .into_iter()
        .filter_elements(|element| match element {
            Element::Node { weight } => *weight == "A" || *weight == "B",
            Element::Edge { .. } => true,
        });

    // because we remove the node `C`, the edge `2 → 0` is also removed
    assert_eq!(filtered.collect::<Vec<_>>(), [
        Element::Node { weight: "A" },
        Element::Node { weight: "B" },
        Element::Edge {
            source: 0,
            target: 1,
            weight: 7,
        },
    ]);
}

#[test]
fn filter_elements_edge() {
    let elements = [
        Element::Node { weight: "A" },
        Element::Node { weight: "B" },
        Element::Node { weight: "C" },
        Element::Edge {
            source: 0,
            target: 1,
            weight: 7,
        },
        Element::Edge {
            source: 2,
            target: 0,
            weight: 9,
        },
    ];

    let filtered = elements
        .into_iter()
        .filter_elements(|element| match element {
            Element::Node { .. } => true,
            Element::Edge { weight, .. } => *weight >= 8,
        });

    assert_eq!(filtered.collect::<Vec<_>>(), [
        Element::Node { weight: "A" },
        Element::Node { weight: "B" },
        Element::Node { weight: "C" },
        Element::Edge {
            source: 2,
            target: 0,
            weight: 9,
        },
    ]);
}
