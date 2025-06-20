use crate::algo::vec;
use crate::algo::Vec;
use crate::alloc::collections::vec_deque::VecDeque;
use crate::visit::{GraphProp, IntoNeighbors, NodeCount, NodeIndexable, Visitable};
use core::hash::Hash;
use fixedbitset::FixedBitSet;
use hashbrown::HashMap;

/// Error type for WFC coloring algorithm.
#[derive(Debug, Clone, PartialEq)]
pub enum WfcColoringError {
    /// The input graph is directed, but the algorithm requires an undirected graph.
    DirectedGraph,
    /// No valid coloring configuration could be found.
    NoValidConfiguration,
}

impl core::fmt::Display for WfcColoringError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            WfcColoringError::DirectedGraph => write!(f, "Graph must be undirected"),
            WfcColoringError::NoValidConfiguration => {
                write!(f, "No valid coloring configuration found")
            }
        }
    }
}

#[cfg(feature = "std")]
impl std::error::Error for WfcColoringError {}

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
/// * [`struct@std::collections::HashMap`] that associates to each `NodeId` its color (1-based numbering), or
/// * [`WfcColoringError`]: error if the graph cannot be colored or is directed.
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
pub fn wfc_coloring<G>(graph: G) -> Result<HashMap<G::NodeId, usize>, WfcColoringError>
where
    G: IntoNeighbors + NodeCount + NodeIndexable + Visitable + GraphProp,
    G::NodeId: Eq + Hash + Copy,
{
    if graph.is_directed() {
        return Err(WfcColoringError::DirectedGraph);
    }

    let node_count = graph.node_count();

    // Convert graph to adjacency matrix using FixedBitSet
    let mut connections = FixedBitSet::with_capacity(node_count * node_count);
    for i in 0..node_count {
        let node = graph.from_index(i);
        for neighbor in graph.neighbors(node) {
            let j = graph.to_index(neighbor);
            connections.set(i * node_count + j, true);
            connections.set(j * node_count + i, true);
        }
    }

    // Calculate maximum degree for color count
    let mut max_degree = 0;
    for i in 0..node_count {
        let degree = (0..node_count)
            .filter(|&j| connections[i * node_count + j])
            .count();
        max_degree = max_degree.max(degree);
    }
    let colors = max_degree + 1;

    // Create and run WFC state
    let mut wfc_state = WfcState::new(node_count, colors, connections);
    let result = wfc_state.run()?;

    // Convert result to hashmap
    let mut color_map = HashMap::with_capacity(node_count);
    for (i, &color) in result.iter().enumerate() {
        let node = graph.from_index(i);
        color_map.insert(node, (color as usize) + 1); // Convert to 1-based colors
    }

    Ok(color_map)
}

#[derive(Debug)]
enum EntropyResult {
    Found(usize),
    Restart,
    Finished,
}

#[derive(Debug)]
struct WfcState {
    nodes: usize,
    colors: usize,
    connections: FixedBitSet,
    available_colors: Vec<FixedBitSet>,
    entropy: Vec<Option<usize>>,
    output: Vec<isize>,
    affected_nodes: VecDeque<usize>,
    finished: bool,
}

impl WfcState {
    fn new(nodes: usize, colors: usize, connections: FixedBitSet) -> Self {
        Self {
            nodes,
            colors,
            connections,
            available_colors: (0..nodes)
                .map(|_| {
                    let mut bitset = FixedBitSet::with_capacity(colors);
                    bitset.set_range(.., true);
                    bitset
                })
                .collect(),
            entropy: vec![Some(colors); nodes],
            output: vec![-1; nodes],
            affected_nodes: VecDeque::new(),
            finished: false,
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
        self.entropy = vec![Some(self.colors); self.nodes];
        self.output = vec![-1; self.nodes];
        self.affected_nodes.clear();
        self.finished = false;
    }

    fn find_lowest_entropy(&mut self) -> EntropyResult {
        let mut min_value = self.colors + 1;
        let mut min_index = None;
        self.finished = true;

        for (index, &val) in self.entropy.iter().enumerate() {
            if val.is_none() {
                continue;
            }
            if val == Some(0) {
                self.restart_wfc();
                return EntropyResult::Restart;
            }
            if let Some(entropy_val) = val {
                if entropy_val < min_value {
                    min_value = entropy_val;
                    min_index = Some(index);
                    self.finished = false;
                }
            }
        }

        match min_index {
            Some(index) => EntropyResult::Found(index),
            None => EntropyResult::Finished,
        }
    }

    fn collapse(&mut self, index: usize) -> Result<(), WfcColoringError> {
        self.entropy[index] = None;
        self.affected_nodes.push_back(index);

        let color_index = self.available_colors[index]
            .ones()
            .next()
            .expect("A color should be available, since otherwise entropy should be 0 and we would have restarted the algorithm.");

        self.available_colors[index].clear();
        self.available_colors[index].set(color_index, true);
        self.output[index] = color_index as isize;

        Ok(())
    }

    fn propagate(&mut self) -> Result<(), WfcColoringError> {
        let mut visited = FixedBitSet::with_capacity(self.nodes); // Replaces Vec<bool>

        while let Some(index) = self.affected_nodes.pop_front() {
            let color_index = self.available_colors[index]
                .ones()
                .next()
                .ok_or(WfcColoringError::NoValidConfiguration)?;

            // Immediately finalize node assignment
            self.output[index] = color_index as isize;
            self.entropy[index] = None;

            for node_index in 0..self.nodes {
                if self.connections[index * self.nodes + node_index]
                    && self.entropy[node_index].is_some()
                    && self.available_colors[node_index].contains(color_index)
                {
                    self.available_colors[node_index].set(color_index, false);

                    let current_entropy = self.entropy[node_index]
                        .expect("Entropy should be some according to if block condition");
                    self.entropy[node_index] = Some(current_entropy.saturating_sub(1));

                    if self.entropy[node_index] == Some(0) {
                        return Err(WfcColoringError::NoValidConfiguration);
                    }
                    if self.entropy[node_index] == Some(1) && !visited.contains(node_index) {
                        visited.insert(node_index); // Mark as visited
                        self.affected_nodes.push_back(node_index);
                    }
                }
            }
        }

        Ok(())
    }

    fn run(&mut self) -> Result<Vec<isize>, WfcColoringError> {
        while !self.finished {
            match self.find_lowest_entropy() {
                EntropyResult::Found(index) => {
                    self.collapse(index)?;
                    self.propagate()?;
                }
                EntropyResult::Restart => continue,
                EntropyResult::Finished => break,
            }
        }
        Ok(self.output.clone())
    }
}
