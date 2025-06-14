use crate::algo::vec;
use crate::algo::Vec;
use crate::alloc::collections::vec_deque::VecDeque;
use crate::alloc::string::String;
use crate::alloc::string::ToString;
use crate::visit::{GraphProp, IntoNeighbors, NodeCount, NodeIndexable, Visitable, GetAdjacencyMatrix};
use core::hash::Hash;
use hashbrown::HashMap;
use fixedbitset::FixedBitSet;

/// \[Generic\] [Wave Function Collapse Coloring algorithm][1] to properly color a non-weighted undirected graph.
///
/// This is a constraint satisfaction algorithm that assigns colors to vertices such that
/// no adjacent vertices share the same color. The algorithm uses entropy-based heuristics
/// to determine the order of vertex coloring and constraint propagation to ensure consistency.
///
/// The graph **must** be undirected. It should not contain loops.
///
/// # Arguments
/// * `graph`: undirected graph without loops.
///
/// # Returns
/// Returns a [`Result`] containing:
/// * [`struct@std::collections::HashMap`] that associates to each `NodeId` its color (1-based numbering).
/// * [`String`]: error message if the graph cannot be colored or is directed.
///
/// # Complexity
/// * Time complexity: **O(|V|² × (|V| + |E|))** in worst case, **O(|V| × (|V| + |E|))** in best case.
/// * Auxiliary space: **O(|V|²)**.
///
/// where **|V|** is the number of nodes and **|E|** is the number of edges.
///
/// [1]: https://arxiv.org/pdf/2108.09329
///
/// # Example
/// ```rust
/// use petgraph::{Graph, Undirected};
/// use std::collections::HashMap;
/// use petgraph::algo::wfc_coloring;
///
/// let mut graph = Graph::<(), (), Undirected>::new_undirected();
/// let a = graph.add_node(());
/// let b = graph.add_node(());
/// let c = graph.add_node(());
///
/// graph.extend_with_edges(&[
///     (a, b),
///     (b, c),
///     (c, a),
/// ]);
///
/// // a ----- b
/// // \      /
/// //  \    /
/// //   \  /
/// //    c
///
/// let coloring = match wfc_coloring(&graph) {
///     Ok(coloring) => coloring,
///    Err(e) => panic!("Error: {}", e),
/// };
/// // [color-1] a ----- b [color-2]
/// //           \      /
/// //            \   /
/// //              c [color-3]
///
/// assert_ne!(coloring[&a], coloring[&b]); // Adjacent vertices have different colors
/// assert_ne!(coloring[&b], coloring[&c]); // Adjacent vertices have different colors
/// assert_ne!(coloring[&c], coloring[&a]); // Adjacent vertices have different colors
/// ```
pub fn wfc_coloring<G>(graph: G) -> Result<HashMap<G::NodeId, usize>, String>
where
    G: IntoNeighbors + NodeCount + NodeIndexable + Visitable + GraphProp + GetAdjacencyMatrix,
    G::NodeId: Eq + Hash + Copy,
{
    if graph.is_directed() {
        return Err("Graph must be undirected".into());
    }

    let node_count = graph.node_count();

    // Get adjacency matrix from the graph
    let adjacency_matrix = graph.adjacency_matrix();

    // Calculate maximum degree for color count
    let mut max_degree = 0;
    for i in 0..node_count {
        let node = graph.from_index(i);
        let degree = (0..node_count)
            .filter(|&j| {
                let neighbor = graph.from_index(j);
                graph.is_adjacent(&adjacency_matrix, node, neighbor)
            })
            .count();
        max_degree = max_degree.max(degree);
    }
    let colors = max_degree + 1;

    // Create and run WFC state
    let mut wfc_state = WfcState::new(node_count, colors, adjacency_matrix, &graph);
    let result = wfc_state.run(&graph)?;

    // Convert result to hashmap
    let mut color_map = HashMap::with_capacity(node_count);
    for i in 0..node_count {
        let node = graph.from_index(i);
        color_map.insert(node, (result[i] as usize) + 1); // Convert to 1-based colors
    }

    Ok(color_map)
}

#[derive(Debug)]
struct WfcState<AdjMatrix> {
    nodes: usize,
    colors: usize,
    adjacency_matrix: AdjMatrix,
    available_colors: Vec<FixedBitSet>,
    entropy: Vec<usize>,
    output: Vec<isize>,
    affected_nodes: VecDeque<usize>,
    min_index: Option<usize>,
    finished: bool,
    restart_flag: bool,
}

impl<AdjMatrix> WfcState<AdjMatrix> {
    fn new<G>(nodes: usize, colors: usize, adjacency_matrix: AdjMatrix, _graph: &G) -> Self
    where
        G: GetAdjacencyMatrix<AdjMatrix = AdjMatrix>,
    {
        Self {
            nodes,
            colors,
            adjacency_matrix,
            available_colors: (0..nodes)
                .map(|_| {
                    let mut bitset = FixedBitSet::with_capacity(colors);
                    bitset.set_range(.., true);
                    bitset
                })
                .collect(),
            entropy: vec![colors; nodes],
            output: vec![-1; nodes],
            affected_nodes: VecDeque::new(),
            min_index: None,
            finished: false,
            restart_flag: false,
        }
    }

    fn restart_wfc(&mut self) {
        self.available_colors = (0..self.nodes)
            .map(|_| {
                let mut bitset = FixedBitSet::with_capacity(self.colors);
                bitset.set_range(.., true);
                bitset
            })
            .collect();
        self.entropy = vec![self.colors; self.nodes];
        self.output = vec![-1; self.nodes];
        self.affected_nodes.clear();
        self.min_index = None;
        self.finished = false;
        self.restart_flag = false;
    }

    fn find_lowest_entropy(&mut self) {
        let mut min_value = self.colors + 1;
        self.finished = true;
        self.min_index = None;

        for (index, &val) in self.entropy.iter().enumerate() {
            if val == usize::MAX {
                continue;
            }
            if val == 0 {
                self.restart_flag = true;
                self.restart_wfc();
                return;
            }
            if val < min_value {
                min_value = val;
                self.min_index = Some(index);
                self.finished = false;
            }
        }
    }

    fn collapse(&mut self, index: usize) -> Result<(), String> {
        if self.finished {
            return Ok(());
        }
        if self.entropy[index] == 0 {
            return Err("Impossible pattern".to_string());
        }

        self.entropy[index] = usize::MAX;
        self.affected_nodes.push_back(index);

        let color_index = self.available_colors[index]
            .ones()
            .next()
            .ok_or_else(|| "No available color".to_string())?;

        self.available_colors[index].clear();
        self.available_colors[index].set(color_index, true);
        self.output[index] = color_index as isize;

        Ok(())
    }

    fn propagate<G>(&mut self, graph: &G) -> Result<(), String>
    where
        G: NodeIndexable + GetAdjacencyMatrix<AdjMatrix = AdjMatrix>,
    {
        let mut visited = vec![false; self.nodes];

        while let Some(index) = self.affected_nodes.pop_front() {
            let color_index = self.available_colors[index]
                .ones()
                .next()
                .ok_or_else(|| "No available color during propagation".to_string())?;

            for node_index in 0..self.nodes {
                let node = graph.from_index(index);
                let neighbor = graph.from_index(node_index);

                if graph.is_adjacent(&self.adjacency_matrix, node, neighbor)
                    && self.entropy[node_index] != usize::MAX
                    && self.available_colors[node_index].contains(color_index)
                {
                    self.available_colors[node_index].set(color_index, false);
                    self.entropy[node_index] -= 1;

                    if self.entropy[node_index] == 0 {
                        return Err("Propagation error: no valid configuration".to_string());
                    }
                    if self.entropy[node_index] == 1 && !visited[node_index] {
                        visited[node_index] = true;
                        self.affected_nodes.push_back(node_index);
                    }
                }
            }
        }

        Ok(())
    }

    fn run<G>(&mut self, graph: &G) -> Result<Vec<isize>, String>
    where
        G: NodeIndexable + GetAdjacencyMatrix<AdjMatrix = AdjMatrix>,
    {
        while !self.finished {
            self.restart_flag = false;
            self.find_lowest_entropy();

            if let Some(index) = self.min_index {
                if self.restart_flag {
                    continue;
                }
                self.collapse(index)?;
                self.propagate(graph)?;
            } else {
                break;
            }
        }
        Ok(self.output.clone())
    }
}