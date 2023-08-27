use petgraph::{algo::tarjan_scc, algorithms::shortest_paths::find_negative_cycle};
use petgraph_core::visit::{Dfs, VisitMap};
use petgraph_csr::{Csr, NodeIndex};

fn n(i: usize) -> NodeIndex {
    NodeIndex::from_usize(i)
}

fn assert_f64_slice_eq(a: &[f64], b: &[f64]) {
    assert_eq!(a.len(), b.len());

    for (a, b) in a.iter().zip(b) {
        assert!((a - b).abs() < f64::EPSILON);
    }
}

#[test]
fn dfs() {
    let mut matrix: Csr = Csr::from_sorted_edges(&[
        (n(0), n(1)),
        (n(0), n(2)),
        (n(1), n(0)),
        (n(1), n(1)),
        (n(1), n(3)),
        (n(2), n(2)),
        // disconnected subgraph
        (n(4), n(4)),
        (n(4), n(5)),
    ])
    .unwrap();

    let mut dfs = Dfs::new(&matrix, n(0));
    while dfs.next(&matrix).is_some() {}

    for i in 0..matrix.node_count() - 2 {
        assert!(dfs.discovered.is_visited(&i), "visited {}", i)
    }

    assert!(!dfs.discovered[4]);
    assert!(!dfs.discovered[5]);

    matrix.add_edge(1, 4, ());

    dfs.reset(&matrix);
    dfs.move_to(NodeIndex::from_usize(0));

    while dfs.next(&matrix).is_some() {}

    for i in 0..matrix.node_count() {
        assert!(dfs.discovered[i], "visited {}", i)
    }
}

#[test]
fn tarjan() {
    let matrix: Csr = Csr::from_sorted_edges(&[
        (n(0), n(1)),
        (n(0), n(2)),
        (n(1), n(0)),
        (n(1), n(1)),
        (n(1), n(3)),
        (n(2), n(2)),
        (n(2), n(4)),
        (n(4), n(4)),
        (n(4), n(5)),
        (n(5), n(2)),
    ])
    .unwrap();

    let scc = tarjan_scc(&matrix);

    assert_eq!(scc, vec![
        vec![n(3)], //
        vec![n(5), n(4), n(2)],
        vec![n(1), n(0)]
    ]);
}

#[test]
fn bellman_ford() {
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

    let result = petgraph::algorithms::shortest_paths::bellman_ford(&matrix, n(0)).unwrap();

    let answer = [0., 0.5, 1.5, 1.5];
    assert_f64_slice_eq(&answer, &result.distances[..4]);

    assert!(result.distances[4..].iter().all(|&x| f64::is_infinite(x)));
}

#[test]
fn bellman_ford_neg_cycle() {
    let matrix: Csr<(), _> = Csr::from_sorted_edges(&[
        (n(0), n(1), 0.5),
        (n(0), n(2), 2.),
        (n(1), n(0), 1.),
        (n(1), n(1), -1.),
        (n(1), n(2), 1.),
        (n(1), n(3), 1.),
        (n(2), n(3), 3.),
    ])
    .unwrap();

    let result = petgraph::algorithms::shortest_paths::bellman_ford(&matrix, n(0));

    assert!(result.is_err());
}

#[test]
fn find_negative_cycle_self_loop() {
    let matrix: Csr<(), _> = Csr::from_sorted_edges(&[
        (n(0), n(1), 0.5),
        (n(0), n(2), 2.),
        (n(1), n(0), 1.),
        (n(1), n(1), -1.),
        (n(1), n(2), 1.),
        (n(1), n(3), 1.),
        (n(2), n(3), 3.),
    ])
    .unwrap();
    let result = find_negative_cycle(&matrix, n(0));

    assert_eq!(result, Some(vec![n(1)]));
}

#[test]
fn find_no_negative_cycle() {
    let matrix: Csr<(), _> = Csr::from_sorted_edges(&[
        (n(0), n(1), 0.5),
        (n(0), n(2), 2.),
        (n(1), n(0), 1.),
        (n(1), n(2), 1.),
        (n(1), n(3), 1.),
        (n(2), n(3), 3.),
    ])
    .unwrap();

    let result = find_negative_cycle(&matrix, n(0));
    assert_eq!(result, None);
}

#[test]
fn find_complex_negative_cycle() {
    let matrix: Csr<(), _> = Csr::from_sorted_edges(&[
        (n(0), n(1), 1.),
        (n(0), n(2), 1.),
        (n(0), n(3), 1.),
        (n(1), n(3), 1.),
        (n(2), n(1), 1.),
        (n(3), n(2), -3.),
    ])
    .unwrap();
    let result = find_negative_cycle(&matrix, n(0));
    assert_eq!(result, Some(vec![n(1), n(3), n(2)]));
}

#[test]
fn find_isolated_negative_cycle() {
    let matrix: Csr<(), _> = Csr::from_sorted_edges(&[(n(0), n(0), -1.)]).unwrap();
    let result = find_negative_cycle(&matrix, n(0));
    assert_eq!(result, Some(vec![n(0)]));
}
