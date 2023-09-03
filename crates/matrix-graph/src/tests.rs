use alloc::{vec, vec::Vec};

use petgraph_core::{
    edge::{Directed, Direction},
    visit::{IntoEdgeReferences, IntoNodeIdentifiers},
};

use crate::{DiMatrix, MatrixGraph, NodeIndex, NotZero, UnMatrix};

#[test]
fn new() {
    let graph = MatrixGraph::<i32, i32>::new();

    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn default() {
    let graph = MatrixGraph::<i32, i32>::default();

    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn with_capacity() {
    let graph = MatrixGraph::<i32, i32>::with_capacity(10);

    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn node_indexing() {
    let mut graph: MatrixGraph<char, ()> = MatrixGraph::new();
    let a = graph.add_node('a');
    let b = graph.add_node('b');

    assert_eq!(graph.node_count(), 2);
    assert_eq!(graph.edge_count(), 0);

    assert_eq!(graph[a], 'a');
    assert_eq!(graph[b], 'b');
}

#[test]
fn remove_node() {
    let mut graph: MatrixGraph<char, ()> = MatrixGraph::new();
    let a = graph.add_node('a');

    graph.remove_node(a);

    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);
}

#[test]
fn add_edge() {
    let mut graph = MatrixGraph::new();

    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');

    graph.add_edge(a, b, ());
    graph.add_edge(b, c, ());

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 2);
}

/// Adds an edge that triggers a second extension of the matrix.
/// From #425
#[test]
fn add_edge_with_extension() {
    let mut graph = DiMatrix::<u8, ()>::new();

    let _n0 = graph.add_node(0);
    let n1 = graph.add_node(1);
    let n2 = graph.add_node(2);
    let n3 = graph.add_node(3);
    let n4 = graph.add_node(4);
    let _n5 = graph.add_node(5);

    graph.add_edge(n2, n1, ());
    graph.add_edge(n2, n3, ());
    graph.add_edge(n2, n4, ());

    assert_eq!(graph.node_count(), 6);
    assert_eq!(graph.edge_count(), 3);
    assert!(graph.has_edge(n2, n1));
    assert!(graph.has_edge(n2, n3));
    assert!(graph.has_edge(n2, n4));
}

#[test]
fn matrix_resize() {
    let mut graph = DiMatrix::<u8, ()>::with_capacity(3);
    let n0 = graph.add_node(0);
    let n1 = graph.add_node(1);
    let n2 = graph.add_node(2);
    let n3 = graph.add_node(3);

    graph.add_edge(n1, n0, ());
    graph.add_edge(n1, n1, ());
    // Triggers a resize from capacity 3 to 4
    graph.add_edge(n2, n3, ());

    assert_eq!(graph.node_count(), 4);
    assert_eq!(graph.edge_count(), 3);
    assert!(graph.has_edge(n1, n0));
    assert!(graph.has_edge(n1, n1));
    assert!(graph.has_edge(n2, n3));
}

#[test]
fn add_edge_with_weights() {
    let mut graph = MatrixGraph::new();
    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');

    graph.add_edge(a, b, true);
    graph.add_edge(b, c, false);

    assert_eq!(*graph.edge_weight(a, b), true);
    assert_eq!(*graph.edge_weight(b, c), false);
}

#[test]
fn add_edge_with_weights_undirected() {
    let mut graph = MatrixGraph::new_undirected();
    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');
    let d = graph.add_node('d');

    graph.add_edge(a, b, "ab");
    graph.add_edge(a, a, "aa");
    graph.add_edge(b, c, "bc");
    graph.add_edge(d, d, "dd");

    assert_eq!(*graph.edge_weight(a, b), "ab");
    assert_eq!(*graph.edge_weight(b, c), "bc");
}

/// Shorthand for `.collect::<Vec<_>>()`
trait IntoVec<T> {
    fn into_vec(self) -> Vec<T>;
}

impl<It, T> IntoVec<T> for It
where
    It: Iterator<Item = T>,
{
    fn into_vec(self) -> Vec<T> {
        self.collect()
    }
}

#[test]
fn clear() {
    let mut graph = MatrixGraph::new();
    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');
    assert_eq!(graph.node_count(), 3);

    graph.add_edge(a, b, ());
    graph.add_edge(b, c, ());
    graph.add_edge(c, a, ());
    assert_eq!(graph.edge_count(), 3);

    graph.clear();

    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);

    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');
    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 0);

    assert_eq!(
        graph.neighbors_directed(a, Direction::Incoming).into_vec(),
        vec![]
    );
    assert_eq!(
        graph.neighbors_directed(b, Direction::Incoming).into_vec(),
        vec![]
    );
    assert_eq!(
        graph.neighbors_directed(c, Direction::Incoming).into_vec(),
        vec![]
    );

    assert_eq!(
        graph.neighbors_directed(a, Direction::Outgoing).into_vec(),
        vec![]
    );
    assert_eq!(
        graph.neighbors_directed(b, Direction::Outgoing).into_vec(),
        vec![]
    );
    assert_eq!(
        graph.neighbors_directed(c, Direction::Outgoing).into_vec(),
        vec![]
    );
}

#[test]
fn clear_undirected() {
    let mut graph = MatrixGraph::new_undirected();
    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');

    assert_eq!(graph.node_count(), 3);

    graph.add_edge(a, b, ());
    graph.add_edge(b, c, ());
    graph.add_edge(c, a, ());

    assert_eq!(graph.edge_count(), 3);

    graph.clear();

    assert_eq!(graph.node_count(), 0);
    assert_eq!(graph.edge_count(), 0);

    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');

    assert_eq!(graph.node_count(), 3);
    assert_eq!(graph.edge_count(), 0);

    assert_eq!(graph.neighbors(a).into_vec(), vec![]);
    assert_eq!(graph.neighbors(b).into_vec(), vec![]);
    assert_eq!(graph.neighbors(c).into_vec(), vec![]);
}

/// Helper trait for always sorting before testing.
trait IntoSortedVec<T> {
    fn into_sorted_vec(self) -> Vec<T>;
}

impl<It, T> IntoSortedVec<T> for It
where
    It: Iterator<Item = T>,
    T: Ord,
{
    fn into_sorted_vec(self) -> Vec<T> {
        let mut v: Vec<T> = self.collect();
        v.sort();
        v
    }
}

/// Helper macro for always sorting before testing.
macro_rules! sorted_vec {
        ($($x:expr),*) => {
            {
                let mut v = vec![$($x,)*];
                v.sort();
                v
            }
        }
    }

#[test]
fn neighbours() {
    let mut graph = MatrixGraph::new();
    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');

    graph.add_edge(a, b, ());
    graph.add_edge(a, c, ());

    let a_neighbors = graph.neighbors(a).into_sorted_vec();
    assert_eq!(a_neighbors, sorted_vec![b, c]);

    let b_neighbors = graph.neighbors(b).into_sorted_vec();
    assert_eq!(b_neighbors, vec![]);

    let c_neighbors = graph.neighbors(c).into_sorted_vec();
    assert_eq!(c_neighbors, vec![]);
}

#[test]
fn neighbours_undirected() {
    let mut graph = MatrixGraph::new_undirected();
    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');

    graph.add_edge(a, b, ());
    graph.add_edge(a, c, ());

    let a_neighbors = graph.neighbors(a).into_sorted_vec();
    assert_eq!(a_neighbors, sorted_vec![b, c]);

    let b_neighbors = graph.neighbors(b).into_sorted_vec();
    assert_eq!(b_neighbors, sorted_vec![a]);

    let c_neighbors = graph.neighbors(c).into_sorted_vec();
    assert_eq!(c_neighbors, sorted_vec![a]);
}

#[test]
fn remove_node_and_edges() {
    let mut graph = MatrixGraph::new();
    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');

    graph.add_edge(a, b, ());
    graph.add_edge(b, c, ());
    graph.add_edge(c, a, ());

    // removing b should break the `a -> b` and `b -> c` edges
    graph.remove_node(b);

    assert_eq!(graph.node_count(), 2);

    let a_neighbors = graph.neighbors(a).into_sorted_vec();
    assert_eq!(a_neighbors, vec![]);

    let c_neighbors = graph.neighbors(c).into_sorted_vec();
    assert_eq!(c_neighbors, vec![a]);
}

#[test]
fn remove_node_and_edges_undirected() {
    let mut graph = UnMatrix::new_undirected();
    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');

    graph.add_edge(a, b, ());
    graph.add_edge(b, c, ());
    graph.add_edge(c, a, ());

    // removing a should break the `a - b` and `a - c` edges
    graph.remove_node(a);

    assert_eq!(graph.node_count(), 2);

    let b_neighbors = graph.neighbors(b).into_sorted_vec();
    assert_eq!(b_neighbors, vec![c]);

    let c_neighbors = graph.neighbors(c).into_sorted_vec();
    assert_eq!(c_neighbors, vec![b]);
}

#[test]
fn node_identifiers() {
    let mut graph = MatrixGraph::new();

    let a = graph.add_node('a');
    let b = graph.add_node('b');
    let c = graph.add_node('c');
    let d = graph.add_node('c');

    graph.add_edge(a, b, ());
    graph.add_edge(a, c, ());

    let node_ids = graph.node_identifiers().into_sorted_vec();
    assert_eq!(node_ids, sorted_vec![a, b, c, d]);
}

#[test]
fn edges_directed() {
    let graph: MatrixGraph<char, bool> = MatrixGraph::from_edges(&[
        (NodeIndex::new(0), NodeIndex::new(5)),
        (NodeIndex::new(0), NodeIndex::new(2)),
        (NodeIndex::new(0), NodeIndex::new(3)),
        (NodeIndex::new(0), NodeIndex::new(1)),
        (NodeIndex::new(1), NodeIndex::new(3)),
        (NodeIndex::new(2), NodeIndex::new(3)),
        (NodeIndex::new(2), NodeIndex::new(4)),
        (NodeIndex::new(4), NodeIndex::new(0)),
        (NodeIndex::new(6), NodeIndex::new(6)),
    ]);

    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(0), Direction::Outgoing)
            .count(),
        4
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(1), Direction::Outgoing)
            .count(),
        1
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(2), Direction::Outgoing)
            .count(),
        2
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(3), Direction::Outgoing)
            .count(),
        0
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(4), Direction::Outgoing)
            .count(),
        1
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(5), Direction::Outgoing)
            .count(),
        0
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(6), Direction::Outgoing)
            .count(),
        1
    );

    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(0), Direction::Incoming)
            .count(),
        1
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(1), Direction::Incoming)
            .count(),
        1
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(2), Direction::Incoming)
            .count(),
        1
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(3), Direction::Incoming)
            .count(),
        3
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(4), Direction::Incoming)
            .count(),
        1
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(5), Direction::Incoming)
            .count(),
        1
    );
    assert_eq!(
        graph
            .edges_directed(NodeIndex::from_usize(6), Direction::Incoming)
            .count(),
        1
    );
}

#[test]
fn edges_undirected() {
    let graph: UnMatrix<char, bool> = UnMatrix::from_edges(&[
        (NodeIndex::new(0), NodeIndex::new(5)),
        (NodeIndex::new(0), NodeIndex::new(2)),
        (NodeIndex::new(0), NodeIndex::new(3)),
        (NodeIndex::new(0), NodeIndex::new(1)),
        (NodeIndex::new(1), NodeIndex::new(3)),
        (NodeIndex::new(2), NodeIndex::new(3)),
        (NodeIndex::new(2), NodeIndex::new(4)),
        (NodeIndex::new(4), NodeIndex::new(0)),
        (NodeIndex::new(6), NodeIndex::new(6)),
    ]);

    assert_eq!(graph.edges(NodeIndex::new(0)).count(), 5);
    assert_eq!(graph.edges(NodeIndex::new(1)).count(), 2);
    assert_eq!(graph.edges(NodeIndex::new(2)).count(), 3);
    assert_eq!(graph.edges(NodeIndex::new(3)).count(), 3);
    assert_eq!(graph.edges(NodeIndex::new(4)).count(), 2);
    assert_eq!(graph.edges(NodeIndex::new(5)).count(), 1);
    assert_eq!(graph.edges(NodeIndex::new(6)).count(), 1);
}

#[test]
fn edges_of_absent_node_is_empty_iterator() {
    let graph: MatrixGraph<char, bool> = MatrixGraph::new();

    assert_eq!(graph.edges(NodeIndex::new(0)).count(), 0);
}

#[test]
fn neighbours_of_absent_node_is_empty_iterator() {
    let graph: MatrixGraph<char, bool> = MatrixGraph::new();

    assert_eq!(graph.neighbors(NodeIndex::new(0)).count(), 0);
}

#[test]
fn edge_references() {
    let graph: MatrixGraph<char, bool> = MatrixGraph::from_edges(&[
        (NodeIndex::new(0), NodeIndex::new(5)),
        (NodeIndex::new(0), NodeIndex::new(2)),
        (NodeIndex::new(0), NodeIndex::new(3)),
        (NodeIndex::new(0), NodeIndex::new(1)),
        (NodeIndex::new(1), NodeIndex::new(3)),
        (NodeIndex::new(2), NodeIndex::new(3)),
        (NodeIndex::new(2), NodeIndex::new(4)),
        (NodeIndex::new(4), NodeIndex::new(0)),
        (NodeIndex::new(6), NodeIndex::new(6)),
    ]);

    assert_eq!(graph.edge_references().count(), 9);
}

#[test]
fn edge_references_undirected() {
    let graph: UnMatrix<char, bool> = UnMatrix::from_edges(&[
        (NodeIndex::new(0), NodeIndex::new(5)),
        (NodeIndex::new(0), NodeIndex::new(2)),
        (NodeIndex::new(0), NodeIndex::new(3)),
        (NodeIndex::new(0), NodeIndex::new(1)),
        (NodeIndex::new(1), NodeIndex::new(3)),
        (NodeIndex::new(2), NodeIndex::new(3)),
        (NodeIndex::new(2), NodeIndex::new(4)),
        (NodeIndex::new(4), NodeIndex::new(0)),
        (NodeIndex::new(6), NodeIndex::new(6)),
    ]);

    assert_eq!(graph.edge_references().count(), 9);
}

#[test]
fn id_storage() {
    use super::IdStorage;

    let mut storage: IdStorage<char> = IdStorage::with_capacity(0);
    let a = storage.add('a');
    let b = storage.add('b');
    let c = storage.add('c');

    assert!(a < b && b < c);

    // list IDs
    assert_eq!(storage.iter_ids().into_vec(), vec![a, b, c]);

    storage.remove(b);

    // re-use of IDs
    let bb = storage.add('B');
    assert_eq!(b, bb);

    // list IDs
    assert_eq!(storage.iter_ids().into_vec(), vec![a, b, c]);
}

#[test]
fn not_zero() {
    let mut g: MatrixGraph<(), i32, Directed, NotZero<i32>> = MatrixGraph::default();

    let a = g.add_node(());
    let b = g.add_node(());

    assert!(!g.has_edge(a, b));
    assert_eq!(g.edge_count(), 0);

    g.add_edge(a, b, 12);

    assert!(g.has_edge(a, b));
    assert_eq!(g.edge_count(), 1);
    assert_eq!(g.edge_weight(a, b), &12);

    g.remove_edge(a, b);

    assert!(!g.has_edge(a, b));
    assert_eq!(g.edge_count(), 0);
}

#[test]
#[cfg(feature = "std")]
fn not_zero_asserted() {
    let mut g: MatrixGraph<(), i32, Directed, NotZero<i32>> = MatrixGraph::default();

    let a = g.add_node(());
    let b = g.add_node(());

    let panic = std::panic::catch_unwind(move || {
        g.add_edge(a, b, 0);
    });

    assert!(panic.is_err());
}

#[test]
fn not_zero_float() {
    let mut graph: MatrixGraph<(), f32, Directed, NotZero<f32>> = MatrixGraph::default();

    let a = graph.add_node(());
    let b = graph.add_node(());

    assert!(!graph.has_edge(a, b));
    assert_eq!(graph.edge_count(), 0);

    graph.add_edge(a, b, 12.);

    assert!(graph.has_edge(a, b));
    assert_eq!(graph.edge_count(), 1);
    assert_eq!(graph.edge_weight(a, b), &12.);

    graph.remove_edge(a, b);

    assert!(!graph.has_edge(a, b));
    assert_eq!(graph.edge_count(), 0);
}
