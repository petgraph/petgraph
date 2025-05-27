extern crate alloc;

use alloc::collections::VecDeque;
use petgraph::algo::maximum_cardinality_search;
use petgraph::algo::{is_chordal, is_perfect_elimination_ordering};
use petgraph::graph::UnGraph;
use petgraph::visit::IntoNodeIdentifiers;
use petgraph::{Graph, Undirected};

#[allow(dead_code)]
/// Checks if a given undirected graph is chordal.
///
/// For each node, perform a BFS to find all simple cycles of length >= 4.
/// Thus, the algorithm is very inefficient in comparison to the
/// [is_chordal] function from the [petgraph] crate and easily
/// runs out of memory for larger graphs.
pub(crate) fn is_chordal_ref<N, E>(graph: &UnGraph<N, E>) -> bool {
    if graph.node_count() < 4 {
        return true; // A graph with less than 4 nodes is trivially chordal
    }

    let node_indices: Vec<_> = graph.node_identifiers().collect();
    for &start in &node_indices {
        let mut queue = VecDeque::new();
        queue.push_back((vec![start], start));
        while let Some((path, last)) = queue.pop_front() {
            for neighbor in graph.neighbors(last) {
                if path.len() > 2 && neighbor == start {
                    // Found a cycle
                    if path.len() >= 4 {
                        // Check for chords
                        let mut is_chordless = true;
                        for i in 0..path.len() {
                            for j in (i + 2)..path.len() {
                                if i == 0 && j == path.len() - 1 {
                                    continue; // skip adjacent in cycle
                                }
                                if graph.contains_edge(path[i], path[j]) {
                                    is_chordless = false;
                                    break;
                                }
                            }
                            if !is_chordless {
                                break;
                            }
                        }
                        if is_chordless {
                            return false;
                        }
                    }
                } else if !path.contains(&neighbor) {
                    let mut new_path = path.clone();
                    new_path.push(neighbor);
                    queue.push_back((new_path, neighbor));
                }
            }
        }
    }
    true
}

#[test]
fn is_chordal_chordal_graph() {
    // Graph is chordal:
    //   --- b ---
    //  |    |    |
    //  |    |    |
    //  |    a -- c
    //  |    |    |
    //  |    |    |
    //   --- d ---

    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b, ()),
        (a, c, ()),
        (a, d, ()),
        (b, c, ()),
        (b, d, ()),
        (c, d, ()),
    ]);

    assert!(is_chordal(&graph));
}

#[test]
fn is_chordal_non_chordal_graph() {
    // Graph is not chordal
    // (see the cycle a-b-e-d-a):
    //   --- b ---
    //  |    |    |
    //  |    |    |
    //  e    a -- c
    //  |    |    |
    //  |    |    |
    //   --- d ---

    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b, ()),
        (a, c, ()),
        (a, d, ()),
        (b, c, ()),
        (b, d, ()),
        (c, d, ()),
    ]);

    let e = graph.add_node(());
    graph.remove_edge(graph.find_edge(b, d).unwrap());
    graph.extend_with_edges([(b, e, ()), (e, d, ())]);

    assert!(!is_chordal(&graph));
}

#[test]
fn is_chordal_classic_chordal_graph() {
    // Example taken from Wikipedia: https://en.wikipedia.org/wiki/Chordal_graph#/media/File:Tree_decomposition.svg
    // Graph is chordal:
    // a --- b --- f
    // |   / | \   |
    // |  /  |  \  |
    // | /   |   \ |
    // c     |     g
    // | \   |   / |
    // |  \  |  /  |
    // |   \ | /   |
    // d --- e --- h

    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();

    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());

    graph.extend_with_edges([
        (a, b, ()),
        (a, c, ()),
        (b, c, ()),
        (b, f, ()),
        (b, g, ()),
        (b, e, ()),
        (c, d, ()),
        (c, e, ()),
        (d, e, ()),
        (e, g, ()),
        (e, h, ()),
        (f, g, ()),
        (g, h, ()),
    ]);

    assert!(is_chordal(&graph));
}

#[test]
fn is_chordal_ref_chordal_graph() {
    // Graph is chordal:
    //   --- b ---
    //  |    |    |
    //  |    |    |
    //  |    a -- c
    //  |    |    |
    //  |    |    |
    //   --- d ---

    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b, ()),
        (a, c, ()),
        (a, d, ()),
        (b, c, ()),
        (b, d, ()),
        (c, d, ()),
    ]);

    assert!(is_chordal_ref(&graph));
}

#[test]
fn is_chordal_ref_non_chordal_graph() {
    // Graph is not chordal
    // (see the cycle a-b-e-d-a):
    //   --- b ---
    //  |    |    |
    //  |    |    |
    //  e    a -- c
    //  |    |    |
    //  |    |    |
    //   --- d ---

    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b, ()),
        (a, c, ()),
        (a, d, ()),
        (b, c, ()),
        (b, d, ()),
        (c, d, ()),
    ]);

    let e = graph.add_node(());
    graph.remove_edge(graph.find_edge(b, d).unwrap());
    graph.extend_with_edges([(b, e, ()), (e, d, ())]);

    assert!(!is_chordal_ref(&graph));
}

#[test]
fn is_chordal_ref_classic_chordal_graph() {
    // Example taken from Wikipedia: https://en.wikipedia.org/wiki/Chordal_graph#/media/File:Tree_decomposition.svg
    // Graph is chordal:
    // a --- b --- f
    // |   / | \   |
    // |  /  |  \  |
    // | /   |   \ |
    // c     |     g
    // | \   |   / |
    // |  \  |  /  |
    // |   \ | /   |
    // d --- e --- h

    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();

    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());

    graph.extend_with_edges([
        (a, b, ()),
        (a, c, ()),
        (b, c, ()),
        (b, f, ()),
        (b, g, ()),
        (b, e, ()),
        (c, d, ()),
        (c, e, ()),
        (d, e, ()),
        (e, g, ()),
        (e, h, ()),
        (f, g, ()),
        (g, h, ()),
    ]);

    assert!(is_chordal_ref(&graph));
}

#[test]
fn maximum_cardinality_search_chordal_graph() {
    // Graph is chordal:
    //   --- b ---
    //  |    |    |
    //  |    |    |
    //  |    a -- c
    //  |    |    |
    //  |    |    |
    //   --- d ---

    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b, ()),
        (a, c, ()),
        (a, d, ()),
        (b, c, ()),
        (b, d, ()),
        (c, d, ()),
    ]);

    let elimination_ordering = maximum_cardinality_search(&graph);
    assert_eq!(elimination_ordering.len(), graph.node_count());
    assert_eq!(elimination_ordering, vec![d, c, b, a]);
    assert!(is_perfect_elimination_ordering(
        &graph,
        &elimination_ordering
    ));
}

#[test]
fn maximum_cardinality_search_non_chordal_graph() {
    // Graph is not chordal anymore
    // (see the cycle a-b-e-d-a):
    //   --- b ---
    //  |    |    |
    //  |    |    |
    //  e    a -- c
    //  |    |    |
    //  |    |    |
    //   --- d ---

    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();
    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());

    graph.extend_with_edges([
        (a, b, ()),
        (a, c, ()),
        (a, d, ()),
        (b, c, ()),
        (b, d, ()),
        (c, d, ()),
    ]);

    let e = graph.add_node(());
    graph.remove_edge(graph.find_edge(b, d).unwrap());
    graph.extend_with_edges([(b, e, ()), (e, d, ())]);

    let elimination_ordering = maximum_cardinality_search(&graph);
    assert_eq!(elimination_ordering.len(), graph.node_count());
    assert_eq!(elimination_ordering, vec![e, d, c, b, a]);
    assert!(!is_perfect_elimination_ordering(
        &graph,
        &elimination_ordering
    ));
}

#[test]
fn maximum_cardinality_search_classic_chordal_graph() {
    // Example taken from Wikipedia: https://en.wikipedia.org/wiki/Chordal_graph#/media/File:Tree_decomposition.svg
    // Graph is chordal:
    // a --- b --- f
    // |   / | \   |
    // |  /  |  \  |
    // | /   |   \ |
    // c     |     g
    // | \   |   / |
    // |  \  |  /  |
    // |   \ | /   |
    // d --- e --- h

    let mut graph: Graph<(), (), Undirected> = Graph::new_undirected();

    let a = graph.add_node(());
    let b = graph.add_node(());
    let c = graph.add_node(());
    let d = graph.add_node(());
    let e = graph.add_node(());
    let f = graph.add_node(());
    let g = graph.add_node(());
    let h = graph.add_node(());

    graph.extend_with_edges([
        (a, b, ()),
        (a, c, ()),
        (b, c, ()),
        (b, f, ()),
        (b, g, ()),
        (b, e, ()),
        (c, d, ()),
        (c, e, ()),
        (d, e, ()),
        (e, g, ()),
        (e, h, ()),
        (f, g, ()),
        (g, h, ()),
    ]);

    let elimination_ordering = maximum_cardinality_search(&graph);
    assert_eq!(elimination_ordering.len(), graph.node_count());
    assert_eq!(elimination_ordering, vec![h, f, g, d, e, c, b, a]);
    assert!(is_perfect_elimination_ordering(
        &graph,
        &elimination_ordering
    ));
}
