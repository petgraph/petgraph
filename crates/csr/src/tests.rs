use petgraph_core::{
    edge::Undirected,
    visit::{EdgeRef, IntoEdgeReferences, IntoNodeReferences},
};

use crate::{Csr, EdgesNotSorted, NodeIndex};

fn n(index: usize) -> NodeIndex {
    NodeIndex::from_usize(index)
}

#[test]
fn sanity() {
    let mut matrix: Csr = Csr::with_nodes(3);
    matrix.add_edge(0, 0, ());
    matrix.add_edge(1, 2, ());
    matrix.add_edge(2, 2, ());
    matrix.add_edge(0, 2, ());
    matrix.add_edge(1, 0, ());
    matrix.add_edge(1, 1, ());

    assert_eq!(&matrix.column, &[n(0), n(2), n(0), n(1), n(2), n(2)]);
    assert_eq!(&matrix.row, &[0, 2, 5, 6]);

    let added = matrix.add_edge(NodeIndex::new(1), NodeIndex::new(2), ());

    assert!(!added);
    assert_eq!(&matrix.column, &[n(0), n(2), n(0), n(1), n(2), n(2)]);
    assert_eq!(&matrix.row, &[0, 2, 5, 6]);

    assert_eq!(matrix.neighbors_slice(NodeIndex::new(1)), &[
        n(0),
        n(1),
        n(2)
    ]);

    assert_eq!(matrix.node_count(), 3);
    assert_eq!(matrix.edge_count(), 6);
}

#[test]
fn undirected() {
    /*
       [ 1 . 1
         . . 1
         1 1 1 ]
    */

    let mut matrix: Csr<(), (), Undirected> = Csr::with_nodes(3);
    matrix.add_edge(0, 0, ());
    matrix.add_edge(0, 2, ());
    matrix.add_edge(1, 2, ());
    matrix.add_edge(2, 2, ());

    assert_eq!(&matrix.column, &[n(0), n(2), n(2), n(0), n(1), n(2)]);
    assert_eq!(&matrix.row, &[0, 2, 3, 6]);

    assert_eq!(matrix.node_count(), 3);
    assert_eq!(matrix.edge_count(), 4);
}

#[test]
fn error_not_sorted() {
    // not sorted in source
    let result: Result<Csr, _> = Csr::from_sorted_edges(&[
        (n(0), n(1)), //
        (n(1), n(0)),
        (n(0), n(2)),
    ]);

    let error = result.unwrap_err();

    assert_eq!(error, EdgesNotSorted {
        first_error: (0, 2)
    });
}

#[test]
fn error_not_sorted_more_complex() {
    // not sorted in target
    let result: Result<Csr, _> = Csr::from_sorted_edges(&[
        (n(0), n(1)), //
        (n(1), n(0)),
        (n(1), n(2)),
        (n(1), n(1)),
    ]);

    let error = result.unwrap_err();

    assert_eq!(error, EdgesNotSorted {
        first_error: (1, 1)
    });
}

#[test]
fn from_sorted_edges() {
    let m: Csr = Csr::from_sorted_edges(&[
        (n(0), n(1)),
        (n(0), n(2)),
        (n(1), n(0)),
        (n(1), n(1)),
        (n(2), n(2)),
        (n(2), n(4)),
    ])
    .unwrap();

    assert_eq!(m.neighbors_slice(NodeIndex::new(0)), &[n(1), n(2)]);
    assert_eq!(m.neighbors_slice(NodeIndex::new(1)), &[n(0), n(1)]);
    assert_eq!(m.neighbors_slice(NodeIndex::new(2)), &[n(2), n(4)]);

    assert_eq!(m.node_count(), 5);
    assert_eq!(m.edge_count(), 6);
}

#[test]
fn edge_references() {
    let matrix: Csr<(), _> = Csr::from_sorted_edges(&[
        (n(0), n(1), 0.5),
        (n(0), n(2), 2.),
        (n(1), n(0), 1.),
        (n(1), n(1), 1.),
        (n(1), n(2), 1.),
        (n(1), n(3), 1.),
        (n(2), n(3), 3.),
        (n(4), n(5), 1.),
        (n(5), n(7), 2.),
        (n(6), n(7), 1.),
        (n(7), n(8), 3.),
    ])
    .unwrap();

    let mut copy = Vec::new();
    for e in matrix.edge_references() {
        copy.push((e.source(), e.target(), *e.weight()));
    }

    let copied_matrix: Csr<(), _> = Csr::from_sorted_edges(&copy).unwrap();

    assert_eq!(&matrix.row, &copied_matrix.row);
    assert_eq!(&matrix.column, &copied_matrix.column);
    assert_eq!(&matrix.edges, &copied_matrix.edges);
}

#[test]
fn add_node() {
    let mut matrix: Csr = Csr::new();
    let a = matrix.add_node(());
    let b = matrix.add_node(());
    let c = matrix.add_node(());

    assert!(matrix.add_edge(a, b, ()));
    assert!(matrix.add_edge(b, c, ()));
    assert!(matrix.add_edge(c, a, ()));

    assert_eq!(matrix.node_count(), 3);

    assert_eq!(matrix.neighbors_slice(a), &[b]);
    assert_eq!(matrix.neighbors_slice(b), &[c]);
    assert_eq!(matrix.neighbors_slice(c), &[a]);

    assert_eq!(matrix.edge_count(), 3);
}

#[test]
fn add_node_with_existing_edges() {
    let mut matrix: Csr = Csr::new();
    let a = matrix.add_node(());
    let b = matrix.add_node(());

    assert!(matrix.add_edge(a, b, ()));

    let c = matrix.add_node(());

    assert_eq!(matrix.node_count(), 3);

    assert_eq!(matrix.neighbors_slice(a), &[b]);
    assert_eq!(matrix.neighbors_slice(b), &[]);
    assert_eq!(matrix.neighbors_slice(c), &[]);

    assert_eq!(matrix.edge_count(), 1);
}

#[test]
fn node_references() {
    let mut matrix: Csr<u32> = Csr::new();
    matrix.add_node(42);
    matrix.add_node(3);
    matrix.add_node(44);

    let mut refs = matrix.node_references();

    assert_eq!(refs.next(), Some((n(0), &42)));
    assert_eq!(refs.next(), Some((n(1), &3)));
    assert_eq!(refs.next(), Some((n(2), &44)));
    assert_eq!(refs.next(), None);
}
