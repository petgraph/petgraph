use std::fmt::Debug;
use std::hash::Hash;
use std::collections::{VecDeque, HashMap, HashSet};
use std::cmp;
use std::cmp::Ord;
use std::ops::{Sub, Add};
use num::Zero;
use crate::graph::{NodeIndex, DiGraph, EdgeIndex};
use crate::Graph;
use crate::visit::{VisitMap, GraphRef, Visitable, IntoNodeReferences};
use crate::visit::{NodeCount, NodeIndexable, NodeRef, GraphBase, IntoEdges, EdgeRef};

/// \[Generic\] [Edmonds-Karp algorithm](https://en.wikipedia.org/wiki/Edmonds%E2%80%93Karp_algorithm)
///
/// Computes the max flow in the graph.
/// Edge weights are assumed to be nonnegative.
/// 
/// # Arguments
/// * `graph`: graph with nonnegative edge weights.
/// * `start`: graph node where the flow starts.
/// * `end`: graph node where the flow ends.
///
/// # Returns
/// * Max flow from `start` to `end`.
/// 
/// Running time is O(|V||E|^2), where |V| is the number of vertices and |E| is the number of edges.
/// Uses O(|E|) space.
/// 
/// Dinic's algorithm solves this problem in O(|V|^2|E|).
/// TODO: Do not remove edges from the graph

pub fn edmonds_karp<G, V, E, N, NR, ER>(
    original_graph: G, 
    start: N,
    end: N,
) -> E
where
    V: Clone + Debug,
    E: Zero + Ord + Copy + Sub<Output = E> + Add<Output = E> + Debug,
    G: GraphBase<NodeId = N> + IntoEdges<EdgeRef = ER> + IntoNodeReferences<NodeRef = NR>,
    NR: NodeRef<NodeId = N, Weight = V>,
    ER: EdgeRef<NodeId = N, Weight = E>,
    N: Hash + Eq + Debug,
    E: Clone + Zero + PartialOrd,
    V: Clone,
{
    // Start by making a directed version of the original graph using BFS.
    // The graph must be an adjacency list in order to run BFS in O(|E|) time.
    let (mut graph, new_start, new_end) = copy_graph_directed(original_graph, start, end).unwrap();

    // For every edge, store the index of its reversed edge.
    // This part could be made more efficient.
    let edges = graph.edge_references();
    let mut reversed_edge = HashMap::new();
    for edge in edges {
        if !reversed_edge.contains_key(&edge.id()) {
            let reverse = graph.find_edge(edge.target(), edge.source()).expect("Edge should be in graph");
            reversed_edge.insert(edge.id(), reverse);
            reversed_edge.insert(reverse, edge.id());
        }
    }

    let mut max_flow = E::zero();
    
    // This loop will run O(|V||E|) times. Each iteration takes O(|E|) time.
    loop {
        let path = BfsPath::shortest_path(&graph, new_start, new_end);
        if path.is_empty() {
            break;
        }
        let path_flow = min_weight(&graph, &path);
        max_flow = max_flow + path_flow;

        for edge in path.into_iter() {
            let weight = &mut graph[edge];
            *weight = *weight - path_flow;
            let reverse_id = reversed_edge[&edge];
            let reversed_weight = &mut graph[reverse_id];
            *reversed_weight = *reversed_weight + path_flow;
        }
    }
    max_flow
}

// Finds the minimum edge weight along the path.
// TODO: use PartialOrd so edges with float weights can be compared.
fn min_weight<V, E>(graph: &Graph<V, E>, path: &Vec<EdgeIndex>) -> E 
where
    E: Zero + Ord + Copy,
{
    if path.is_empty() {
        return E::zero();
    }
    let mut weight = graph[path[0]];
    for edge in path.iter().skip(1) {
        weight = cmp::min(weight, graph[*edge]);
    }
    return weight;
}

/// Creates a copy of original_graph and stores it as a directed adjacency list.
/// If n -> n' is an edge, it also adds the edge n' -> n but with weight 0.
/// Also takes start and end and gives corresponding nodes in the new graph.
pub fn copy_graph_directed<G, V, E, N, NR, ER>(
    original_graph: G,
    start: N,
    end: N
) -> Result<(DiGraph<V, E>, NodeIndex, NodeIndex), String>
where
    G: GraphBase<NodeId = N> + IntoEdges<EdgeRef = ER> + IntoNodeReferences<NodeRef = NR>,
    NR: NodeRef<NodeId = N, Weight = V>,
    ER: EdgeRef<NodeId = N, Weight = E>,
    N: Hash + Eq + Debug,
    E: Clone + Zero + PartialOrd,
    V: Clone,
{
    let mut graph_copy: DiGraph<V, E> = Graph::default();
    // Ids of new nodes
    let mut new_node_ids = Vec::new();
    // All nodes in the graph
    let node_references: Vec<_> = original_graph.node_references().collect();

    let mut start_opt = None;
    let mut end_opt = None;
    // Add all nodes into graph_copy and keep track of their new index
    for node in node_references.iter() {
        let id = graph_copy.add_node(node.weight().clone());
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

            let weight = edge_ref.weight().clone();
            if weight < E::zero() {
                return Err("Nonnegative edgeweights expected for Edmonds-Karp.".to_owned());
            }
            graph_copy.add_edge(new_node_ids[start_index], new_node_ids[end_index], weight);
        }
    }

    for (index1, index2) in extra_edges {
        graph_copy.add_edge(new_node_ids[index1], new_node_ids[index2], E::zero());
    }
    Ok((graph_copy, new_start, new_end))
}

/// Same as crate::visit::Bfs but uses Bfs to compute the shortest path in an unweighted graph.
#[derive(Clone)]
pub struct BfsPath<N, VM> {
    /// The queue of nodes to visit
    pub stack: VecDeque<N>,
    /// The map of discovered nodes
    pub discovered: VM, 
}

impl<N, VM> Default for BfsPath<N, VM>
where
    VM: Default,
{
    fn default() -> Self {
        BfsPath {
            stack: VecDeque::new(),
            discovered: VM::default(),
        }
    }
}

impl<N, VM> BfsPath<N, VM>
where
    N: Copy + PartialEq,
    VM: VisitMap<N>,
{
    /// Create a new **Bfs**, using the graph's visitor map, and put **start**
    /// in the stack of nodes to visit.
    fn new<G>(graph: G, start: N) -> Self
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM>,
    {
        let mut discovered = graph.visit_map();
        discovered.visit(start);
        let mut stack = VecDeque::new();
        stack.push_front(start);
        BfsPath { stack, discovered }
    }

    /// Returns a shortest path from start to end ignoring edge weights.
    /// The path is a vector of EdgeRef.
    /// Returns an empty vector is no path exists.
    /// Ignore edges with zero weight.
    /// TODO: implement with partial equals.
    pub fn shortest_path<G, E, I, ER>(graph: G, start: N, end: N) -> Vec<I>
    where
        G: GraphRef + Visitable<NodeId = N, Map = VM> + NodeCount,
        G: IntoEdges<EdgeRef = ER> + NodeIndexable,
        ER: EdgeRef<NodeId = N, EdgeId = I, Weight = E>,
        E: Zero + Eq,
        N: Debug,
        I: Copy,
    {
        // For every Node N in G, stores the EdgeRef that first goes to N
        let mut predecessor: Vec<Option<_>> = vec![None; graph.node_count()];
        let mut path = Vec::new();
        let mut bfs = BfsPath::new(graph, start);

        while let Some(node) = bfs.stack.pop_front() {
            if node == end {
                break;
            }
            for edge in graph.edges(node) {
                if *edge.weight() != E::zero() {
                    let succ = edge.target();
                    if bfs.discovered.visit(succ) {
                        bfs.stack.push_back(succ);
                        predecessor[graph.to_index(succ)] = Some(edge);
                    }
                }
            }
        } 

        let mut next = end;
        while let Some(edge) = predecessor[graph.to_index(next)] {
            path.push(edge.id());
            let node = edge.source();
            if node == start {
                break;
            }
            next = node;
        }
        path.reverse();
        path
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_max_flow_unweighted() {
        let mut graph = Graph::<_, u32>::new();
        let v0 = graph.add_node(0);
        let v1 = graph.add_node(1);
        let v2 = graph.add_node(2);
        let v3 = graph.add_node(3);
        let v4 = graph.add_node(4);

        graph.extend_with_edges(&[
            (v0, v1, 1), (v0, v2, 1),
            (v2, v3, 1), (v3, v4, 1), (v2, v4, 1),
        ]);
        // 0 ---> 1
        // |      
        // v
        // 2 ---> 4
        // |     7
        // v   /
        // 3
        assert_eq!(1, edmonds_karp(&graph, v0, v4));

        graph.add_edge(v1, v4, 1);
        assert_eq!(2, edmonds_karp(&graph, v0, v4));

        graph.add_edge(v0, v3, 1);
        assert_eq!(3, edmonds_karp(&graph, v0, v4));

        graph.clear();
        graph.extend_with_edges(&[
            (v0, v1, 1), (v0, v2, 1), (v1, v4, 1),
            (v2, v3, 1), (v4, v3, 1), (v2, v4, 1),
        ]);
        assert_eq!(2, edmonds_karp(&graph, v0, v4));
    }

    #[test]
    fn test_max_flow_weighted() {
        let mut graph = Graph::<_, u32>::new();
        let v0 = graph.add_node(0);
        let v1 = graph.add_node(1);
        let v2 = graph.add_node(2);
        let v3 = graph.add_node(3);
        graph.extend_with_edges(&[
            (v1, v2, 3), (v1, v3, 1), (v2, v3, 3),
            (v2, v0, 1), (v3, v0, 3)
        ]);
        let max_flow = edmonds_karp(&graph, v1, v0);
        assert_eq!(4, max_flow);

        let mut graph = Graph::<_, u32>::new();
        let a1 = graph.add_node(0);
        let b1 = graph.add_node(0);
        let b2 = graph.add_node(0);
        let b3 = graph.add_node(0);
        let c1 = graph.add_node(0);
        let c2 = graph.add_node(0);
        let c3 = graph.add_node(0);
        let d1 = graph.add_node(0);
        graph.extend_with_edges(&[
            (a1, b1, 6), (a1, b2, 1), (a1, b3, 1),
            (b1, c1, 6), (b1, c2, 6),
            (b2, c1, 1), (b2, c3, 1),
            (b3, c2, 1), (b3, c3, 1),
            (c1, d1, 1), (c2, d1, 4), (c3, d1, 3)
        ]);
        let max_flow = edmonds_karp(&graph, a1, d1);
        assert_eq!(7, max_flow);

        let mut graph = Graph::<_, u32>::new();
        let a1 = graph.add_node(0);
        let b1 = graph.add_node(0);
        let b2 = graph.add_node(0);
        let b3 = graph.add_node(0);
        let c1 = graph.add_node(0);
        let c2 = graph.add_node(0);
        let d1 = graph.add_node(0);
        graph.extend_with_edges(&[
            (a1, b1, 20), (a1, b2, 40), (a1, b3, 5),
            (b1, b2, 5), (b2, b3, 5),
            (b1, c1, 20), (b2, c1, 25), (b2, c2, 15), (b3, c2, 10),
            (c1, c2, 5),
            (c1, d1, 40), (c2, d1, 30),
        ]);
        let max_flow = edmonds_karp(&graph, a1, d1);
        assert_eq!(65, max_flow);
    }
}