use crate::algo::vec;
use crate::algo::Vec;
use crate::alloc::collections::vec_deque::VecDeque;
use crate::alloc::string::String;
use crate::alloc::string::ToString;
use crate::visit::{GraphProp, IntoNeighbors, NodeCount, NodeIndexable, Visitable};
use core::hash::Hash;
use hashbrown::HashMap;

/// \[Generic\] [Wave Function Collapse algorithm][1] to properly color a non-weighted undirected graph.
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
    G: IntoNeighbors + NodeCount + NodeIndexable + Visitable + GraphProp,
    G::NodeId: Eq + Hash + Copy,
{
    if graph.is_directed() {
        return Err("Graph must be undirected".into());
    }

    let node_count = graph.node_count();

    // Convert graph to adjacency matrix
    let mut connections = vec![vec![false; node_count]; node_count];
    for i in 0..node_count {
        let node = graph.from_index(i);
        for neighbor in graph.neighbors(node) {
            let j = graph.to_index(neighbor);
            connections[i][j] = true;
            connections[j][i] = true;
        }
    }

    // Calculate maximum degree for color count
    let max_degree = connections
        .iter()
        .map(|row| row.iter().filter(|&&x| x).count())
        .max()
        .unwrap_or(0);
    let colors = max_degree + 1;

    // Create and run WFC state
    let mut wfc_state = WfcState::new(node_count, colors, connections);
    let result = wfc_state.run()?;

    // Convert result to hashmap
    let mut color_map = HashMap::new();
    for i in 0..node_count {
        let node = graph.from_index(i);
        color_map.insert(node, (result[i] as usize) + 1); // Convert to 1-based colors
    }

    Ok(color_map)
}

#[derive(Debug)]
struct WfcState {
    nodes: usize,
    colors: usize,
    connections: Vec<Vec<bool>>,
    available_colors: Vec<Vec<bool>>,
    entropy: Vec<usize>,
    output: Vec<isize>,
    affected_nodes: VecDeque<usize>,
    min_index: Option<usize>,
    finished: bool,
    restart_flag: bool,
}

impl WfcState {
    fn new(nodes: usize, colors: usize, connections: Vec<Vec<bool>>) -> Self {
        Self {
            nodes,
            colors,
            connections,
            available_colors: vec![vec![true; colors]; nodes],
            entropy: vec![colors; nodes],
            output: vec![-1; nodes],
            affected_nodes: VecDeque::new(),
            min_index: None,
            finished: false,
            restart_flag: false,
        }
    }

    fn restart_wfc(&mut self) {
        self.available_colors = vec![vec![true; self.colors]; self.nodes];
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
            .iter()
            .position(|&x| x)
            .ok_or_else(|| "No available color".to_string())?;

        self.available_colors[index] = vec![false; self.colors];
        self.available_colors[index][color_index] = true;
        self.output[index] = color_index as isize;

        Ok(())
    }

    fn propagate(&mut self) -> Result<(), String> {
        let mut visited = vec![false; self.nodes];

        while let Some(index) = self.affected_nodes.pop_front() {
            let color_index = self.available_colors[index]
                .iter()
                .position(|&x| x)
                .ok_or_else(|| "No available color during propagation".to_string())?;

            for node_index in 0..self.nodes {
                if self.connections[index][node_index]
                    && self.entropy[node_index] != usize::MAX
                    && self.available_colors[node_index][color_index]
                {
                    self.available_colors[node_index][color_index] = false;
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

    fn run(&mut self) -> Result<Vec<isize>, String> {
        while !self.finished {
            self.restart_flag = false;
            self.find_lowest_entropy();

            if let Some(index) = self.min_index {
                if self.restart_flag {
                    continue;
                }
                self.collapse(index)?;
                self.propagate()?;
            } else {
                break;
            }
        }
        Ok(self.output.clone())
    }
}
