use std::{
    collections::HashMap,
    hash::Hash,
    iter::{from_fn, FromIterator},
};

use crate::visit::{
    EdgeCount, EdgeRef, GraphProp, IntoEdges, IntoEdgesDirected, IntoNeighborsDirected, NodeCount,
};
use crate::Directed;
use crate::Direction::Outgoing;

type Path<T> = Vec<T>;

/// Returns an iterator that produces all simple paths on a directed graph `G(V, E)`,
/// with each path specified as an (ordered) list of nodes from node `from` to node `to`.
/// Each path contains at least `min_edges_traversed` steps (default 1) and at most
/// `max_edges_traversed` (default is the order of `G(V, E)`).
///
/// Panics if either `min_edges_traversed` or `max_edges_traversed` is zero or
/// `min_edges_traversed > max_edges_traversed` is specified.
///
/// # Note:
///  * a simple path is a path without repetition,
///  * the number of nodes in a path is the number of steps taken + 1.
///  * no provision exists to limit/pre-check the size of the iterable, meaning
///    that if `G(V, E)` contains numerous parallel edges (for example) it may
///    take consideralbe time to iterate through all paths and/or materializing
///    all paths may lead to memory issues.
///
/// This algorithm is adapted from <https://networkx.github.io/documentation/stable/reference/algorithms/generated/networkx.algorithms.simple_paths.all_simple_paths.html>.
///
/// # Example
/// ```
/// use petgraph::{algo, prelude::*};
///
/// let mut graph = DiGraph::<&str, i32>::new();
///
/// let a = graph.add_node("a");
/// let b = graph.add_node("b");
/// let c = graph.add_node("c");
/// let d = graph.add_node("d");
///
/// graph.extend_with_edges(&[(a, b, 1), (b, c, 1), (c, d, 1), (a, b, 1), (b, d, 1)]);
///
/// let ways = algo::all_simple_node_paths(&graph, a, d, None, None)
///   .collect::<Vec<_>>();
///
/// assert_eq!(4, ways.len());
/// ```
pub fn all_simple_node_paths<G>(
    graph: G,
    from: G::NodeId,
    to: G::NodeId,
    min_edges_traversed: Option<usize>,
    max_edges_traversed: Option<usize>,
) -> impl Iterator<Item = Path<G::NodeId>>
where
    G: NodeCount + IntoNeighborsDirected,
    G::NodeId: Eq,
{
    let min_length = min_edges_traversed.unwrap_or(1);
    let max_length = max_edges_traversed.unwrap_or(graph.node_count() - 1);
    assert!(min_length.gt(&0), "paths must contain at least one edge");
    assert!(
        max_length.ge(&min_length),
        "impossible `min_length > max_length` condition specified"
    );

    let mut visited: Path<G::NodeId> = Path::from_iter(Some(from));
    let mut stack = vec![graph.neighbors_directed(from, Outgoing)];

    from_fn(move || {
        while let Some(children) = stack.last_mut() {
            if let Some(child) = children.next() {
                if visited.len() < max_length {
                    if child == to {
                        if visited.len() >= min_length {
                            let path = visited.iter().cloned().chain(Some(to)).collect::<Path<_>>();
                            return Some(path);
                        }
                    } else if !visited.contains(&child) {
                        visited.push(child);
                        stack.push(graph.neighbors_directed(child, Outgoing));
                    }
                } else {
                    if (child.eq(&to) || children.any(|v| v.eq(&to)))
                        && visited.len().ge(&min_length)
                    {
                        let path = visited.iter().cloned().chain(Some(to)).collect::<Path<_>>();
                        return Some(path);
                    }
                    stack.pop();
                    visited.pop();
                }
            } else {
                stack.pop();
                visited.pop();
            }
        }
        None
    })
}

/// Returns an iterator that produces all simple paths on a directed graph `G(V, E)`,
/// with each path specified as an (ordered) list of edges from node `from` to node `to`.
/// Each path contains at least `min_edges_traversed` steps (default 1) and at most
/// `max_edges_traversed` (default is order of `G(V, E)`).
///
/// Panics if either `min_edges_traversed` or `max_edges_traversed` is zero or
/// `min_edges_traversed > max_edges_traversed` is specified.
///
/// # Note:
///  * a simple path is a path without repetition,
///  * no provision exists to limit/pre-check the size of the iterable, meaning
///    that if `G(V, E)` contains numerous parallel edges (for example) it may
///    take consideralbe time to iterate through all paths and/or materializing
///    all paths may lead to memory issues.
///
/// # Example
/// ```
/// use petgraph::{algo, prelude::*};
///
/// let mut graph = DiGraph::<&str, i32>::new();
///
/// let a = graph.add_node("a");
/// let b = graph.add_node("b");
/// let c = graph.add_node("c");
/// let d = graph.add_node("d");
///
/// graph.extend_with_edges(&[(a, b, 1), (b, c, 1), (c, d, 1), (a, b, 1), (b, d, 1)]);
///
/// let ways = algo::all_simple_edge_paths(&graph, a, d, None, None)
///   .collect::<Vec<_>>();
///
/// assert_eq!(4, ways.len());
/// ```
pub fn all_simple_edge_paths<G>(
    graph: G,
    from: G::NodeId,
    to: G::NodeId,
    min_edges_traversed: Option<usize>,
    max_edges_traversed: Option<usize>,
) -> impl Iterator<Item = Path<G::EdgeId>>
where
    G: EdgeCount + IntoEdgesDirected,
    G::NodeId: Eq,
    G::EdgeId: Eq,
{
    let min_length = min_edges_traversed.unwrap_or(1);
    let max_length = max_edges_traversed.unwrap_or(graph.edge_count());
    assert!(min_length.gt(&0), "paths must contain at least one edge");
    assert!(
        max_length.ge(&min_length),
        "impossible `min_length > max_length` condition specified"
    );

    let mut visited: Path<G::EdgeId> = Path::new();
    let mut stack = vec![graph.edges_directed(from, Outgoing)];

    from_fn(move || {
        visited.pop();
        while let Some(children) = stack.last_mut() {
            if let Some(child) = children.next() {
                if visited.len().lt(&max_length) {
                    if child.target().eq(&to) {
                        if visited.len().ge(&min_length) {
                            visited.push(child.id());
                            let path = visited.to_vec();
                            return Some(path);
                        }
                    } else if !visited.contains(&child.id()) {
                        visited.push(child.id());
                        stack.push(graph.edges_directed(child.target(), Outgoing));
                    }
                } else {
                    stack.pop();
                    visited.pop();
                }
            } else {
                stack.pop();
                visited.pop();
            }
        }
        None
    })
}

/// Compute all simple paths in a directed graph from a given starting node to
/// all reachable nodes.
///
/// For this algorithm, paths are sequences of edges. All simple paths here
/// means that in any given path, an edge or a vertex can not be repeated.
///
/// starting_node: Explore paths from this node
///
/// If there are multiple edges between the same nodes, these count as different
/// paths and will both appear in the output. With many such options, it's can
/// lead to a "combinatorial explosion" of paths.
///
/// The returned map will contain entries for all explored nodes including the
/// starting node.
pub fn all_simple_edge_paths_from<G>(
    graph: G,
    starting_node: G::NodeId,
) -> HashMap<G::NodeId, Vec<Path<G::EdgeId>>>
where
    G: IntoEdges + GraphProp<EdgeType = Directed>,
    G::NodeId: Eq + Hash,
    G::EdgeId: Eq + Hash,
{
    let mut current_paths = Vec::<Path<_>>::new();
    let mut result = HashMap::<G::NodeId, Vec<Path<_>>>::default();
    // Map edge id to edge target node id
    // This extra map needed when we don't have graph[edge_id] lookup
    let mut edge_targets = HashMap::new();

    for edge in graph.edges(starting_node) {
        edge_targets.entry(edge.id()).or_insert(edge.target());
        current_paths.push(vec![edge.id()]);
    }

    result.entry(starting_node).or_default().push(Vec::new());

    // Using a BFS order, explore all possible paths from the starting point
    let mut next_paths = Vec::new();
    while !current_paths.is_empty() {
        for path in current_paths.drain(..) {
            let path_end = *path.last().expect("non-empty path");
            let target = edge_targets[&path_end];

            // Given a path P to `target`
            // Skip if there is already an existing path to `target` that is a prefix of `P`.
            let paths_to_target = result.entry(target).or_default();
            if paths_to_target
                .iter()
                .any(|existing_path| path.starts_with(existing_path))
            {
                continue;
            }

            // Else add P to `target` and explore all paths P + E where E is an edge from target.
            for edge in graph.edges(target) {
                debug_assert!(!path.contains(&edge.id()));
                edge_targets.entry(edge.id()).or_insert(edge.target());
                let mut new_path = path.clone();
                new_path.push(edge.id());
                next_paths.push(new_path);
            }
            paths_to_target.push(path);
        }
        // current_paths now empty, swap with next list to explore
        std::mem::swap(&mut current_paths, &mut next_paths);
    }

    result
}

#[cfg(test)]
mod test {
    use std::{
        collections::{HashMap, HashSet},
        panic,
    };

    use crate::dot::Dot;
    use crate::graph::{EdgeIndex, Graph, NodeIndex};
    use crate::visit::NodeFiltered;

    use super::all_simple_edge_paths;
    use super::all_simple_edge_paths_from;
    use super::all_simple_node_paths;

    /// test on a common graph that has all the features one could expect to test
    fn build_test_graph<'a>() -> Graph<&'a str, &'a str> {
        let mut graph: Graph<&str, &str> = Graph::new();
        let n0 = graph.add_node("0");
        let n1 = graph.add_node("1");
        let n2 = graph.add_node("2");
        let n3 = graph.add_node("3");
        let n4 = graph.add_node("4");
        let n5 = graph.add_node("5");

        graph.add_edge(n0, n1, "A0");
        graph.add_edge(n0, n1, "A1");
        graph.add_edge(n0, n1, "A2");
        graph.add_edge(n0, n2, "B");
        graph.add_edge(n0, n3, "C");
        graph.add_edge(n1, n2, "D");
        graph.add_edge(n1, n3, "E");
        graph.add_edge(n2, n3, "F");
        graph.add_edge(n2, n4, "G");
        graph.add_edge(n3, n2, "H");
        graph.add_edge(n3, n4, "I");
        graph.add_edge(n4, n2, "J");
        graph.add_edge(n4, n5, "K");
        graph.add_edge(n5, n2, "L");
        graph.add_edge(n5, n3, "M");

        graph
    }

    #[test]
    fn test_all_simple_edge_paths_from() {
        fn string_paths_to_index_map(
            graph: &Graph<&str, &str>,
            paths: &[(&str, &[&str])],
        ) -> HashMap<NodeIndex, Vec<Vec<EdgeIndex>>> {
            let mut result = HashMap::<_, Vec<Vec<_>>>::new();
            for &(node_weight, edge_path) in paths.iter() {
                let node_id = graph
                    .node_indices()
                    .find(|ni| graph[*ni] == node_weight)
                    .unwrap();
                let edge_ids = edge_path
                    .iter()
                    .map(|&edge_weight| {
                        graph
                            .edge_indices()
                            .find(|ei| graph[*ei] == edge_weight)
                            .unwrap()
                    })
                    .collect::<Vec<_>>();
                result.entry(node_id).or_default().push(edge_ids);
            }
            result
        }

        {
            let mut gr = Graph::new();
            let n0 = gr.add_node("0");
            let n1 = gr.add_node("1");
            let n2 = gr.add_node("2");
            let n3 = gr.add_node("3");
            let n4 = gr.add_node("4");
            let n5 = gr.add_node("5");
            let n6 = gr.add_node("6");
            let n7 = gr.add_node("7");
            let n8 = gr.add_node("8");

            gr.add_edge(n0, n2, "A");
            gr.add_edge(n0, n1, "B");
            gr.add_edge(n2, n3, "D");
            gr.add_edge(n3, n6, "I");
            gr.add_edge(n3, n4, "E");
            gr.add_edge(n1, n4, "C");
            gr.add_edge(n4, n5, "F");
            gr.add_edge(n5, n6, "H");
            gr.add_edge(n6, n2, "J");
            gr.add_edge(n5, n7, "K");
            gr.add_edge(n6, n8, "L");
            gr.add_edge(n5, n3, "G");

            let paths_full: Vec<(&str, &[&str])> = vec![
                ("0", &[]),
                ("1", &["B"]),
                ("2", &["A"]),
                ("2", &["B", "C", "F", "H", "J"]),
                ("2", &["B", "C", "F", "G", "I", "J"]),
                ("3", &["A", "D"]),
                ("3", &["B", "C", "F", "G"]),
                ("3", &["B", "C", "F", "H", "J", "D"]),
                ("4", &["B", "C"]),
                ("4", &["A", "D", "E"]),
                ("5", &["B", "C", "F"]),
                ("5", &["A", "D", "E", "F"]),
                ("6", &["A", "D", "I"]),
                ("6", &["B", "C", "F", "H"]),
                ("6", &["B", "C", "F", "G", "I"]),
                ("6", &["A", "D", "E", "F", "H"]),
                ("7", &["B", "C", "F", "K"]),
                ("7", &["A", "D", "E", "F", "K"]),
                ("8", &["A", "D", "I", "L"]),
                ("8", &["B", "C", "F", "H", "L"]),
                ("8", &["B", "C", "F", "G", "I", "L"]),
                ("8", &["A", "D", "E", "F", "H", "L"]),
            ];

            let paths_full_map = string_paths_to_index_map(&gr, &paths_full);

            let result_full = all_simple_edge_paths_from(&gr, n0);

            assert_eq!(paths_full_map, result_full);

            let paths_subset: Vec<(_, &[&str])> = vec![
                ("2", &["F", "H", "J"]),
                ("2", &["F", "G", "I", "J"]),
                ("3", &["F", "G"]),
                ("3", &["F", "H", "J", "D"]),
                ("4", &[]),
                ("5", &["F"]),
                ("6", &["F", "H"]),
                ("6", &["F", "G", "I"]),
            ];
            let subset = [n2, n3, n4, n5, n6]; // strongly connected component as subset
            let paths_subset_map = string_paths_to_index_map(&gr, &paths_subset);

            let subset_graph = NodeFiltered(&gr, |node| subset.contains(&node));
            let result_subset = all_simple_edge_paths_from(&subset_graph, n4);
            assert_eq!(paths_subset_map, result_subset);
        }

        {
            // In this graph, find only the path A from 0 to 1.
            // Do not include path A, B from 0 to 1.
            let mut gr = Graph::new();
            let n0 = gr.add_node("0");
            let n1 = gr.add_node("1");

            let e0 = gr.add_edge(n0, n1, "A");
            gr.add_edge(n1, n1, "B");

            println!("{}", Dot::new(&gr));

            let result = all_simple_edge_paths_from(&gr, n0);
            assert_eq!(result[&n0], vec![vec![]]);
            assert_eq!(result[&n1], vec![vec![e0]]);
        }

        {
            // In this graph, there are 2 trails from 0 to 1 and 4 trails from 0 to 2.
            let mut gr = Graph::new();
            let n0 = gr.add_node("0");
            let n1 = gr.add_node("1");
            let n2 = gr.add_node("2");

            let e0 = gr.add_edge(n0, n1, "A");
            let e1 = gr.add_edge(n0, n1, "B");
            let e2 = gr.add_edge(n1, n2, "C");
            let e3 = gr.add_edge(n1, n2, "D");

            println!("{}", Dot::new(&gr));

            let mut result = all_simple_edge_paths_from(&gr, n0);

            assert_eq!(result[&n0], vec![vec![]]);
            assert_eq!(result[&n1], vec![vec![e1], vec![e0]]);

            result.get_mut(&n2).unwrap().sort_unstable();

            let mut expected_n0_to_n2 =
                vec![vec![e0, e2], vec![e1, e2], vec![e0, e3], vec![e1, e3]];
            expected_n0_to_n2.sort_unstable();
            assert_eq!(result[&n2], expected_n0_to_n2);
        }
    }
    #[test]
    fn test_all_simple_node_paths() {
        let graph = build_test_graph();
        let n0 = graph
            .node_indices()
            .find(|&idx| graph.node_weight(idx).eq(&Some(&"0")))
            .unwrap();
        let n5 = graph
            .node_indices()
            .find(|&idx| graph.node_weight(idx).eq(&Some(&"5")))
            .unwrap();

        // test node paths
        let expected_node_paths: HashSet<Vec<_>> = vec![
            vec!["0", "1", "2", "3", "4", "5"],
            vec!["0", "1", "2", "4", "5"],
            vec!["0", "1", "3", "2", "4", "5"],
            vec!["0", "1", "3", "4", "5"],
            vec!["0", "2", "3", "4", "5"],
            vec!["0", "2", "4", "5"],
            vec!["0", "3", "2", "4", "5"],
            vec!["0", "3", "4", "5"],
        ]
        .into_iter()
        .collect();

        let mut n_paths = 0;
        let actual: HashSet<Vec<_>> = all_simple_node_paths(&graph, n0, n5, None, None)
            .map(|path| {
                n_paths += 1;
                path.iter()
                    .map(|&node| *graph.node_weight(node).unwrap())
                    .collect()
            })
            .collect();

        assert_eq!(n_paths, 16);
        assert_eq!(actual.len(), 8);
        assert_eq!(expected_node_paths, actual);

        // test edge paths
        let expected_edge_paths: HashSet<Vec<_>> = vec![
            vec!["A2", "D", "G", "J", "F", "I", "K"],
            vec!["B", "G", "K"],
            vec!["B", "G", "J", "F", "I", "K"],
            vec!["A2", "D", "F", "I", "K"],
            vec!["A1", "E", "H", "F", "I", "J", "G", "K"],
            vec!["A2", "E", "H", "F", "I", "K"],
            vec!["A1", "E", "H", "G", "J", "F", "I", "K"],
            vec!["A1", "D", "G", "J", "F", "I", "K"],
            vec!["A2", "E", "I", "J", "G", "K"],
            vec!["A2", "E", "I", "J", "F", "H", "G", "K"],
            vec!["A0", "D", "G", "K"],
            vec!["A0", "E", "I", "J", "F", "H", "G", "K"],
            vec!["A2", "D", "G", "K"],
            vec!["A2", "D", "F", "H", "G", "K"],
            vec!["A0", "E", "H", "F", "I", "J", "G", "K"],
            vec!["B", "F", "I", "J", "G", "K"],
            vec!["A1", "D", "F", "I", "J", "G", "K"],
            vec!["A0", "E", "H", "F", "I", "K"],
            vec!["C", "H", "G", "J", "F", "I", "K"],
            vec!["A1", "D", "G", "K"],
            vec!["A2", "E", "H", "G", "J", "F", "I", "K"],
            vec!["A1", "E", "H", "G", "K"],
            vec!["A1", "E", "I", "J", "G", "K"],
            vec!["A0", "E", "I", "K"],
            vec!["A0", "D", "F", "I", "K"],
            vec!["C", "H", "F", "I", "K"],
            vec!["A1", "D", "F", "I", "K"],
            vec!["B", "F", "I", "K"],
            vec!["A1", "D", "F", "H", "G", "K"],
            vec!["A0", "E", "H", "G", "K"],
            vec!["A0", "E", "H", "G", "J", "F", "I", "K"],
            vec!["C", "H", "F", "I", "J", "G", "K"],
            vec!["A2", "E", "H", "G", "K"],
            vec!["A1", "E", "I", "K"],
            vec!["A2", "E", "H", "F", "I", "J", "G", "K"],
            vec!["A0", "D", "G", "J", "F", "I", "K"],
            vec!["C", "I", "K"],
            vec!["C", "I", "J", "F", "H", "G", "K"],
            vec!["A2", "D", "F", "I", "J", "G", "K"],
            vec!["A2", "E", "I", "K"],
            vec!["A1", "E", "I", "J", "F", "H", "G", "K"],
            vec!["C", "H", "G", "K"],
            vec!["B", "F", "H", "G", "K"],
            vec!["A0", "E", "I", "J", "G", "K"],
            vec!["A0", "D", "F", "I", "J", "G", "K"],
            vec!["A0", "D", "F", "H", "G", "K"],
            vec!["A1", "E", "H", "F", "I", "K"],
            vec!["C", "I", "J", "G", "K"],
        ]
        .into_iter()
        .collect();

        let mut n_paths = 0;
        let actual: HashSet<Vec<_>> = all_simple_edge_paths(&graph, n0, n5, None, None)
            .map(|path| {
                n_paths += 1;
                path.iter()
                    .map(|&edge| *graph.edge_weight(edge).unwrap())
                    .collect()
            })
            .collect();

        assert_eq!(n_paths, 48);
        assert_eq!(actual.len(), 48);
        assert_eq!(expected_edge_paths, actual);
    }

    #[test]
    fn test_one_simple_node_path() {
        let graph = build_test_graph();
        let n2 = graph
            .node_indices()
            .find(|&idx| graph.node_weight(idx).eq(&Some(&"2")))
            .unwrap();
        let n5 = graph
            .node_indices()
            .find(|&idx| graph.node_weight(idx).eq(&Some(&"5")))
            .unwrap();

        // node paths
        let expected_node_paths = [vec!["2", "3", "4", "5"]];
        let actual: Vec<Vec<_>> = all_simple_node_paths(&graph, n2, n5, Some(3), None)
            .map(|path| {
                path.iter()
                    .map(|&node| *graph.node_weight(node).unwrap())
                    .collect()
            })
            .collect();

        assert_eq!(actual.as_slice(), expected_node_paths);

        // edge paths
        let expected_edge_paths = [vec!["F", "H", "G", "K"]];
        let actual: Vec<Vec<_>> = all_simple_edge_paths(&graph, n2, n5, Some(3), Some(4))
            .map(|path| {
                path.iter()
                    .map(|&edge| *graph.edge_weight(edge).unwrap())
                    .collect()
            })
            .collect();

        assert_eq!(actual.as_slice(), expected_edge_paths);
        println!("ACTUAL: {:?}", actual);
    }

    #[test]
    fn test_no_simple_node_paths() {
        let graph = build_test_graph();
        let n0 = graph
            .node_indices()
            .find(|&idx| graph.node_weight(idx).eq(&Some(&"0")))
            .unwrap();
        let n5 = graph
            .node_indices()
            .find(|&idx| graph.node_weight(idx).eq(&Some(&"5")))
            .unwrap();

        let paths = all_simple_node_paths(&graph, n5, n0, None, None);

        assert_eq!(paths.count(), 0);
    }

    #[test]
    fn negative_path_tests() {
        let graph = build_test_graph();
        let n0 = graph
            .node_indices()
            .find(|&idx| graph.node_weight(idx).eq(&Some(&"0")))
            .unwrap();
        let n5 = graph
            .node_indices()
            .find(|&idx| graph.node_weight(idx).eq(&Some(&"5")))
            .unwrap();

        let result = panic::catch_unwind(|| {
            all_simple_node_paths(&graph, n0, n5, Some(0), None).count();
        });
        match result {
            Err(msg) => {
                if let Some(s) = msg.downcast_ref::<&str>() {
                    assert_eq!(*s, "paths must contain at least one edge");
                } else {
                    panic!("unexpected panic message");
                }
            }
            Ok(_) => panic!("expected a panic, but the code managed to run"),
        }

        let result = panic::catch_unwind(|| {
            all_simple_edge_paths(&graph, n0, n5, Some(0), None).count();
        });
        match result {
            Err(msg) => {
                if let Some(s) = msg.downcast_ref::<&str>() {
                    assert_eq!(*s, "paths must contain at least one edge");
                } else {
                    panic!("unexpected panic message");
                }
            }
            Ok(_) => panic!("expected a panic, but the code managed to run"),
        }

        let result = panic::catch_unwind(|| {
            all_simple_node_paths(&graph, n0, n5, Some(5), Some(1)).count();
        });
        match result {
            Err(msg) => {
                if let Some(s) = msg.downcast_ref::<&str>() {
                    assert_eq!(
                        *s,
                        "impossible `min_length > max_length` condition specified"
                    );
                } else {
                    panic!("unexpected panic message");
                }
            }
            Ok(_) => panic!("expected a panic, but the code managed to run"),
        }

        let result = panic::catch_unwind(|| {
            all_simple_edge_paths(&graph, n0, n5, Some(5), Some(1)).count();
        });
        match result {
            Err(msg) => {
                if let Some(s) = msg.downcast_ref::<&str>() {
                    assert_eq!(
                        *s,
                        "impossible `min_length > max_length` condition specified"
                    );
                } else {
                    panic!("unexpected panic message");
                }
            }
            Ok(_) => panic!("expected a panic, but the code managed to run"),
        }
    }
}
