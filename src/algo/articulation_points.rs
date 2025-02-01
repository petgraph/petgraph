use crate::visit;
use crate::visit::{EdgeRef, IntoEdges, IntoNodeReferences, NodeIndexable, NodeRef};
use std::cmp::min;
use std::collections::{HashMap, HashSet};
use std::hash::Hash;

/// \[Generic\] Find articulation points in a graph using [Tarjan's algorithm](https://en.wikipedia.org/wiki/Tarjan%27s_strongly_connected_components_algorithm).
///
/// Compute the articulation points of a graph (Nodes, which would increase the number of connected components in the graph.
///
/// # Arguments
/// * `graph`: A directed graph
///
/// # Returns
/// * `HashSet`: Hashset of the node ids which correspond to the articulation points of the graph.
///
/// # Examples
/// ```rust
/// use petgraph::{
///     algo::articulation_points,
///     graph::{NodeIndex, UnGraph},
///     algo::articulation_points::articulation_points,
/// };
///
/// fn main() {
///     let mut gr = UnGraph::<&str, ()>::new_undirected();
///     let a = gr.add_node("A");
///     let b = gr.add_node("B");
///     let c = gr.add_node("C");
///
///     gr.add_edge(a, b, ());
///     gr.add_edge(b, c, ());
///
///     let articulation_points: Vec<&str> = articulation_points(&gr)
///         .into_iter()
///         .map(|node_idx| gr[node_idx])
///         .collect();
///
///     // Articulation Points: ["B"]
///     println!("Articulation Points: {:?}", articulation_points);
/// }
/// ```
pub fn articulation_points<G>(g: G) -> HashSet<G::NodeId>
where
    G: IntoNodeReferences + IntoEdges + NodeIndexable + visit::GraphProp,
    G::NodeWeight: Clone,
    G::EdgeWeight: Clone + PartialOrd,
    G::NodeId: Eq + Hash,
{
    let graph_size = g.node_references().size_hint().0;

    let mut visited = vec![false; graph_size];
    let mut disc = vec![usize::MAX; graph_size];
    let mut low = vec![usize::MAX; graph_size];
    let mut parent = vec![usize::MAX; graph_size];
    let mut ap = HashSet::with_capacity(graph_size);
    let mut time = 0;

    for node in g.node_references() {
        let node_id = g.to_index(node.id());
        if !visited[node_id] {
            _dfs(
                &g,
                node_id,
                &mut visited,
                &mut parent,
                &mut low,
                &mut disc,
                &mut ap,
                &mut time,
            );
        }
    }

    ap.into_iter().map(|id| g.from_index(id)).collect()
}

/// Small helper enum that defines the various splitup recursion steps of Tarjan's algorithm.
enum RecursionStep {
    BaseStep(usize),
    ProcessChildStep(usize, usize),
    NoBackEdgeConditionCheck(usize, usize),
    RootMoreThanTwoChildrenCheck(usize),
}

/// Helper that performs the required DFS in an iterative manner.
fn _dfs<G>(
    g: &G,
    target_node: usize,
    visited: &mut Vec<bool>,
    parent: &mut Vec<usize>,
    low: &mut Vec<usize>,
    disc: &mut Vec<usize>,
    ap: &mut HashSet<usize>,
    time: &mut usize,
) where
    G: IntoEdges + NodeIndexable,
{
    let mut stack: Vec<RecursionStep> = vec![RecursionStep::BaseStep(target_node)];
    let mut children_count: HashMap<usize, usize> = HashMap::new();

    while let Some(recursionStep) = stack.pop() {
        match recursionStep {
            RecursionStep::BaseStep(current_node) => {
                visited[current_node] = true;
                disc[current_node] = *time;
                low[current_node] = *time;
                *time += 1;

                stack.push(RecursionStep::RootMoreThanTwoChildrenCheck(current_node));
                for edge in g.edges(g.from_index(current_node)) {
                    let child_node = g.to_index(edge.target());
                    stack.push(RecursionStep::ProcessChildStep(current_node, child_node));
                }
            }
            RecursionStep::ProcessChildStep(current_node, child_node) => {
                if !visited[child_node] {
                    parent[child_node] = current_node;
                    *children_count.entry(current_node).or_insert(0) += 1;

                    stack.push(RecursionStep::NoBackEdgeConditionCheck(
                        current_node,
                        child_node,
                    ));
                    stack.push(RecursionStep::BaseStep(child_node));
                } else if child_node != parent[current_node] {
                    low[current_node] = min(low[current_node], disc[child_node]);
                }
            }
            RecursionStep::NoBackEdgeConditionCheck(current_node, child_node) => {
                low[current_node] = min(low[current_node], low[child_node]);

                if parent[current_node] != usize::MAX && low[child_node] >= disc[current_node] {
                    ap.insert(current_node);
                }
            }

            RecursionStep::RootMoreThanTwoChildrenCheck(current_node) => {
                let child_count = children_count.get(&current_node).cloned().unwrap_or(0);
                if parent[current_node] == usize::MAX && child_count > 1 {
                    ap.insert(current_node);
                }
            }
        }
    }
}
