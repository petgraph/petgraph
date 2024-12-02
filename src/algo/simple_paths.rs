use std::{
    hash::Hash,
    iter::{from_fn, FromIterator},
};

use indexmap::IndexSet;
use std::collections::HashMap;

use crate::visit::{EdgeRef, GraphProp, IntoEdges};
use crate::visit::{IntoNeighborsDirected, NodeCount};
use crate::Directed;
use crate::Direction::Outgoing;

/// Returns an iterator that produces all simple paths from `from` node to `to`, which contains at least `min_intermediate_nodes` nodes
/// and at most `max_intermediate_nodes`, if given, or limited by the graph's order otherwise. The simple path is a path without repetitions.
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
/// let ways = algo::all_simple_paths::<Vec<_>, _>(&graph, a, d, 0, None)
///   .collect::<Vec<_>>();
///
/// assert_eq!(4, ways.len());
/// ```
pub fn all_simple_paths<TargetColl, G>(
    graph: G,
    from: G::NodeId,
    to: G::NodeId,
    min_intermediate_nodes: usize,
    max_intermediate_nodes: Option<usize>,
) -> impl Iterator<Item = TargetColl>
where
    G: NodeCount,
    G: IntoNeighborsDirected,
    G::NodeId: Eq + Hash,
    TargetColl: FromIterator<G::NodeId>,
{
    // how many nodes are allowed in simple path up to target node
    // it is min/max allowed path length minus one, because it is more appropriate when implementing lookahead
    // than constantly add 1 to length of current path
    let max_length = if let Some(l) = max_intermediate_nodes {
        l + 1
    } else {
        graph.node_count() - 1
    };

    let min_length = min_intermediate_nodes + 1;

    // list of visited nodes
    let mut visited: IndexSet<G::NodeId> = IndexSet::from_iter(Some(from));
    // list of childs of currently exploring path nodes,
    // last elem is list of childs of last visited node
    let mut stack = vec![graph.neighbors_directed(from, Outgoing)];

    from_fn(move || {
        while let Some(children) = stack.last_mut() {
            if let Some(child) = children.next() {
                if visited.len() < max_length {
                    if child == to {
                        if visited.len() >= min_length {
                            let path = visited
                                .iter()
                                .cloned()
                                .chain(Some(to))
                                .collect::<TargetColl>();
                            return Some(path);
                        }
                    } else if !visited.contains(&child) {
                        visited.insert(child);
                        stack.push(graph.neighbors_directed(child, Outgoing));
                    }
                } else {
                    if (child == to || children.any(|v| v == to)) && visited.len() >= min_length {
                        let path = visited
                            .iter()
                            .cloned()
                            .chain(Some(to))
                            .collect::<TargetColl>();
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

type Path<T> = Vec<T>;

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
pub fn all_simple_paths_from<G>(
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
    use std::{collections::HashMap, collections::HashSet, iter::FromIterator};

    use itertools::assert_equal;

    use crate::dot::Dot;
    use crate::graph::{DiGraph, EdgeIndex, Graph, NodeIndex};
    use crate::visit::NodeFiltered;

    use super::all_simple_paths;
    use super::all_simple_paths_from;

    #[test]
    fn test_all_simple_paths() {
        let graph = DiGraph::<i32, i32, _>::from_edges(&[
            (0, 1),
            (0, 2),
            (0, 3),
            (1, 2),
            (1, 3),
            (2, 3),
            (2, 4),
            (3, 2),
            (3, 4),
            (4, 2),
            (4, 5),
            (5, 2),
            (5, 3),
        ]);

        let expexted_simple_paths_0_to_5 = vec![
            vec![0usize, 1, 2, 3, 4, 5],
            vec![0, 1, 2, 4, 5],
            vec![0, 1, 3, 2, 4, 5],
            vec![0, 1, 3, 4, 5],
            vec![0, 2, 3, 4, 5],
            vec![0, 2, 4, 5],
            vec![0, 3, 2, 4, 5],
            vec![0, 3, 4, 5],
        ];

        println!("{}", Dot::new(&graph));
        let actual_simple_paths_0_to_5: HashSet<Vec<_>> =
            all_simple_paths(&graph, 0u32.into(), 5u32.into(), 0, None)
                .map(|v: Vec<_>| v.into_iter().map(|i| i.index()).collect())
                .collect();
        assert_eq!(actual_simple_paths_0_to_5.len(), 8);
        assert_eq!(
            HashSet::from_iter(expexted_simple_paths_0_to_5),
            actual_simple_paths_0_to_5
        );
    }

    #[test]
    fn test_one_simple_path() {
        let graph = DiGraph::<i32, i32, _>::from_edges(&[(0, 1), (2, 1)]);

        let expexted_simple_paths_0_to_1 = &[vec![0usize, 1]];
        println!("{}", Dot::new(&graph));
        let actual_simple_paths_0_to_1: Vec<Vec<_>> =
            all_simple_paths(&graph, 0u32.into(), 1u32.into(), 0, None)
                .map(|v: Vec<_>| v.into_iter().map(|i| i.index()).collect())
                .collect();

        assert_eq!(actual_simple_paths_0_to_1.len(), 1);
        assert_equal(expexted_simple_paths_0_to_1, &actual_simple_paths_0_to_1);
    }

    #[test]
    fn test_no_simple_paths() {
        let graph = DiGraph::<i32, i32, _>::from_edges(&[(0, 1), (2, 1)]);

        println!("{}", Dot::new(&graph));
        let actual_simple_paths_0_to_2: Vec<Vec<_>> =
            all_simple_paths(&graph, 0u32.into(), 2u32.into(), 0, None)
                .map(|v: Vec<_>| v.into_iter().map(|i| i.index()).collect())
                .collect();

        assert_eq!(actual_simple_paths_0_to_2.len(), 0);
    }

    #[test]
    fn test_all_simple_paths_from() {
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

            let result_full = all_simple_paths_from(&gr, n0);

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
            let result_subset = all_simple_paths_from(&subset_graph, n4);
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

            let result = all_simple_paths_from(&gr, n0);
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

            let mut result = all_simple_paths_from(&gr, n0);

            assert_eq!(result[&n0], vec![vec![]]);
            assert_eq!(result[&n1], vec![vec![e1], vec![e0]]);

            result.get_mut(&n2).unwrap().sort_unstable();

            let mut expected_n0_to_n2 =
                vec![vec![e0, e2], vec![e1, e2], vec![e0, e3], vec![e1, e3]];
            expected_n0_to_n2.sort_unstable();
            assert_eq!(result[&n2], expected_n0_to_n2);
        }
    }
}
