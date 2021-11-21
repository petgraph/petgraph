use std::fmt::Debug;
use std::hash::Hash;
use std::collections::{HashMap, HashSet};
use crate::graph::{NodeIndex, DiGraph, EdgeIndex};
use crate::Graph;
use crate::visit::{IntoNodeReferences, Bfs};
use crate::visit::{NodeRef, GraphBase, IntoEdges, EdgeRef};
use crate::algo::OrderableMeasure;

/// \[Generic\] [Edmonds-Karp algorithm][link]
/// 
/// Computes the max flow in the graph.
/// Edge weights are assumed to be nonnegative.
/// 
/// # Arguments
/// * `graph`
/// * `start`: graph node where the flow starts.
/// * `end`: graph node where the flow ends.
/// * `edge_cost`: nonnegative edge weights for the graph. 
/// (Note if an edge_cost has value NaN, it allows arbitrarily large flow.)
///
/// # Returns
/// * Max flow from `start` to `end`.
/// 
/// Running time is O(|V||E|^2), where |V| is the number of vertices and |E| is
/// the number of edges.
/// Uses O(|E|) space.
/// 
/// Dinic's algorithm solves this problem in O(|V|^2|E|).
/// 
/// [link]: https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm
/// 
/// # Example
/// ```rust
/// use petgraph::algo::edmonds_karp;
/// use petgraph::Graph;
/// let mut graph = Graph::new();
/// let v0 = graph.add_node(0);
/// let v1 = graph.add_node(1);
/// let v2 = graph.add_node(2);
/// let v3 = graph.add_node(3);
/// graph.extend_with_edges(&[
///     (v1, v2, 3), (v1, v3, 1), (v2, v3, 3),
///     (v2, v0, 1), (v3, v0, 3)
/// ]);
/// 
/// // Graph represented with edge weights
/// //
/// //       3
/// //   v1 --> v2
/// //   |    / |
/// // 1 |  3/  | 1 
/// //   v  L   V
/// //   v3 --> v0
/// //       3
/// 
/// let max_flow = edmonds_karp(&graph, v1, v0, |e| *e.weight());
/// assert_eq!(max_flow, 4);
/// ```
/// TODO: PartialOrd versus Ord

pub fn edmonds_karp<G, V, E, N, NR, ER, F>(
    original_graph: G, 
    start: N,
    end: N,
    edge_cost: F,
) -> E
where
    E: OrderableMeasure,
    G: GraphBase<NodeId = N>,
    G: IntoEdges<EdgeRef = ER> + IntoNodeReferences<NodeRef = NR>,
    NR: NodeRef<NodeId = N, Weight = V>,
    ER: EdgeRef<NodeId = N, Weight = E>,
    N: Hash + Eq + Debug,
    F: Fn(G::EdgeRef) -> E,
{
    // Start by making a directed version of the original graph using BFS.
    // The graph must be an adjacency list in order to run BFS in O(|E|) time.
    let (mut graph, new_start, new_end) = copy_graph_directed(
        original_graph,
        start,
        end,
        edge_cost
    ).unwrap();

    // For every edge, store the index of its reversed edge.
    // This part could be made more efficient.
    let edges = graph.edge_references();
    let mut reversed_edge = HashMap::new();
    for edge in edges {
        if !reversed_edge.contains_key(&edge.id()) {
            let reverse = graph.find_edge(edge.target(), edge.source())
                .expect("Edge should be in graph");
            reversed_edge.insert(edge.id(), reverse);
            reversed_edge.insert(reverse, edge.id());
        }
    }

    // Default is the value 0.
    let mut max_flow = E::default(); 
    
    // This loop will run O(|V||E|) times. Each iteration takes O(|E|) time.
    loop {
        let path = Bfs::shortest_path(&graph, new_start, new_end);
        if path.is_empty() {
            break;
        }
        let path_flow = min_weight(&graph, &path);
        max_flow = max_flow + path_flow.clone();

        for edge in path.into_iter() {
            let weight = &mut graph[edge];
            *weight = weight.clone() - path_flow.clone();
            let reverse_id = reversed_edge[&edge];
            let reversed_weight = &mut graph[reverse_id];
            *reversed_weight = reversed_weight.clone() + path_flow.clone();
        }
    }
    max_flow
}

/// Finds the minimum edge weight along the path.
fn min_weight<V, E>(graph: &Graph<V, E>, path: &Vec<EdgeIndex>) -> E 
where
    E: OrderableMeasure,
{
    if path.is_empty() {
        return E::default();
    }
    let mut weight = &graph[path[0]];
    for edge in path.iter().skip(1) {
        weight = weight.min(&graph[*edge]);
    }
    return weight.clone();
}

/// Creates a copy of original_graph and stores it as a directed adjacency list.
/// 
/// If n -> n' is an edge, it also adds the edge n' -> n but with weight 0.
/// Also takes start and end and gives corresponding nodes in the new graph.
fn copy_graph_directed<G, V, E, N, NR, ER, F>(
    original_graph: G,
    start: N,
    end: N,
    edge_cost: F
) -> Result<(DiGraph<u8, E>, NodeIndex, NodeIndex), String>
where
    G: GraphBase<NodeId = N>,
    G: IntoEdges<EdgeRef = ER> + IntoNodeReferences<NodeRef = NR>,
    NR: NodeRef<NodeId = N, Weight = V>,
    ER: EdgeRef<NodeId = N, Weight = E>,
    N: Hash + Eq + Debug,
    E: OrderableMeasure,
    F: Fn(G::EdgeRef) -> E,
{
    let mut graph_copy: DiGraph<_, E> = Graph::default();
    // Ids of new nodes
    let mut new_node_ids = Vec::new();
    // All nodes in the graph
    let node_references: Vec<_> = original_graph.node_references().collect();

    let mut start_opt = None;
    let mut end_opt = None;
    // Add all nodes into graph_copy and keep track of their new index
    for node in node_references.iter() {
        let id = graph_copy.add_node(0);
        new_node_ids.push(id);
        if node.id() == start {
            start_opt = Some(id);
        }
        if node.id() == end {
            end_opt = Some(id);
        }
    }

    if start_opt == None || end_opt == None {
        return Err("Start or end not found".to_owned());
    }
    let new_start = start_opt.unwrap();
    let new_end = end_opt.unwrap();

    // Store the index of a node in the vector node_references
    let index_map: HashMap<_, _> = node_references
        .iter()
        .enumerate()
        .map(|(index, node)| (node.id(), index))
        .collect();
    
    // Extra edges to add to graph_copy
    let mut extra_edges = HashSet::new();
    
    for start_ref in node_references {
        let edges = original_graph.edges(start_ref.id());
        for edge_ref in edges {
            let start_index = index_map[&start_ref.id()];
            let end_index = index_map[&edge_ref.target()];
            
            // We need to add the reversed edge if its not already there.
            let option = extra_edges.remove(&(end_index, start_index));
            if !option {
                extra_edges.insert((end_index, start_index));
            }

            let weight = edge_cost(edge_ref);
            if weight < E::default() {
                return Err("Nonnegative edgeweights expected for Edmonds-Karp.".to_owned());
            }
            graph_copy.add_edge(new_node_ids[start_index], new_node_ids[end_index], weight);
        }
    }

    for (index1, index2) in extra_edges {
        graph_copy.add_edge(new_node_ids[index1], new_node_ids[index2], E::default());
    }
    Ok((graph_copy, new_start, new_end))
}
