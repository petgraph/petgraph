use core::fmt;

use super::metrics::{
    total_edge_weight, Modularity, ModularityEdgeWeight, ModularityNodeId, Partition,
};

use crate::algo::community::modularity;
use crate::graph::UnGraph;
use crate::{
    visit::{Data, EdgeRef, GraphBase},
    EdgeDirection,
};
use alloc::{vec, vec::Vec};
use hashbrown::{HashMap, HashSet};

pub struct LouvainReturn<G>
where
    G: Modularity,
{
    pub communities: Vec<HashSet<G::NodeId>>,
    pub nodes_to_communities: HashMap<G::NodeId, usize>,
    pub modularity: f64,
}

/// Enum that holds an "inner graph" for one level of the Louvain algorithm,
/// i.e. a graph in which each community from the previous level is treated
/// as a single node.
///
/// For the first stage of the algorithm, each node from the input graph
/// start out in its own community, so the inner graph is the same as the
/// input graph. In this case we avoid copying the input.
enum InnerGraph<'g, G>
where
    G: Modularity,
{
    Init(&'g G),
    Undirected(UnGraph<(), f64, usize>),
    // Directed case is not implemented yet
    // Directed(DiGraph<(), f64, usize>)
}

impl<'g, G: Modularity> InnerGraph<'g, G>
where
    <G as Data>::EdgeWeight: ModularityEdgeWeight,
{
    /// Returns the number of nodes in the inner graph
    pub fn node_count(&self) -> usize {
        match self {
            InnerGraph::Init(&g) => g.node_count(),
            InnerGraph::Undirected(g) => g.node_count(),
        }
    }

    /// Returns a vector `w` where `w[i]` is the total weight of the
    /// edges incident on the `i`th node.
    pub fn degrees(&self) -> Vec<f64> {
        let mut degrees = vec![0.0; self.node_count()];
        match self {
            InnerGraph::Init(&g) => {
                for e in g.edge_references() {
                    let w: f64 = (*e.weight()).into();
                    let (a, b) = (g.to_index(e.source()), g.to_index(e.target()));
                    degrees[a] += w;
                    degrees[b] += w;
                }
            }
            InnerGraph::Undirected(g) => {
                for e in g.edge_references() {
                    let w = e.weight();
                    degrees[e.source().index()] += w;
                    degrees[e.target().index()] += w;
                }
            }
        }
        degrees
    }

    /// Given a node index `idx`, returns a map `w`. For each neighbor
    /// `nbr` of `idx`, `w[nbr]` is the total weight of all the edges
    /// connecting `idx` and `nbr`.
    pub fn neighbor_community_weights(
        &self,
        idx: usize,
        node_to_community: &[usize],
    ) -> HashMap<usize, f64> {
        let mut weights = HashMap::new();

        let mut add_weight = |n: usize, w: f64| {
            let com = node_to_community[n];
            weights.entry(com).and_modify(|x| *x += w).or_insert(w);
        };

        match self {
            InnerGraph::Init(&g) => {
                let node = g.from_index(idx);
                for edge in g.edges_directed(node, EdgeDirection::Outgoing) {
                    let n = g.to_index(edge.target());
                    add_weight(n, (*edge.weight()).into());
                }
            }
            InnerGraph::Undirected(g) => {
                for edge in g.edges_directed(idx.into(), EdgeDirection::Outgoing) {
                    let n = edge.target().index();
                    add_weight(n, *edge.weight());
                }
            }
        }

        weights
    }
}

/// Trait for additional functions used in the Louvain algorithm. Since the idea
/// is to compute increasingly coarse partitions of the input graph, we implement
/// these for `Partition`.
trait LouvainAlgo<'g, G>
where
    G: Modularity,
{
    /// Compute the inner graph for a given partition.
    fn to_inner_graph(&self) -> InnerGraph<'g, G>;

    /// Replaces the current partition. The argument `new_partition` should be
    /// a vector of size `n` (where `n` is the number of nodes in `self.graph`).
    fn update(&mut self, new_partition: Vec<usize>);

    /// Returns the current graph partition as a vector of sets of `NodeId`, for
    /// example to return to the Python layer.
    fn to_vec_of_hashsets(&self) -> Vec<HashSet<G::NodeId>>;

    /// Returns a hashmap from Nodes to the id of the community they belong to
    fn to_hashmap(&self) -> HashMap<G::NodeId, usize>;
}

impl<'g, G: Modularity> LouvainAlgo<'g, G> for Partition<'g, G>
where
    <G as Data>::EdgeWeight: ModularityEdgeWeight,
    <G as GraphBase>::NodeId: ModularityNodeId,
{
    fn to_inner_graph(&self) -> InnerGraph<'g, G> {
        if self.n_subsets == self.graph.node_count() {
            return InnerGraph::Init(self.graph);
        }

        // Construct a new graph where:
        //   - Node `n_i` corresponds to the `i`th community in the partition
        //   - Nodes `n_i` and `n_j` have an edge with weight `w`, where `w` is
        //     the sum of all edge weights connecting nodes in `n_i` and `n_j`.
        //     (including self-loops)
        let mut edges: HashMap<(usize, usize), f64> = HashMap::new();
        for e in self.graph.edge_references() {
            let (a, b) = (self.subset_idx(e.source()), self.subset_idx(e.target()));
            let inner_edge = if self.graph.is_directed() {
                (core::cmp::min(a, b), core::cmp::max(a, b))
            } else {
                (a, b)
            };
            let w: f64 = (*e.weight()).into();
            edges.entry(inner_edge).and_modify(|x| *x += w).or_insert(w);
        }

        InnerGraph::Undirected(UnGraph::from_edges(
            edges.iter().map(|(k, &v)| (k.0, k.1, v)),
        ))
    }

    fn update(&mut self, new_partition: Vec<usize>) {
        self.node_to_subset = new_partition;
        self.n_subsets = *self.node_to_subset.iter().max().unwrap_or(&0) + 1;
    }

    fn to_vec_of_hashsets(&self) -> Vec<HashSet<G::NodeId>> {
        let mut v = vec![HashSet::new(); self.n_subsets];
        for (idx, &s) in self.node_to_subset.iter().enumerate() {
            let node = self.graph.from_index(idx);
            v[s].insert(node);
        }
        v
    }

    fn to_hashmap(&self) -> HashMap<G::NodeId, usize> {
        let mut out = HashMap::new();
        for (idx, &subset) in self.node_to_subset.iter().enumerate() {
            let node = self.graph.from_index(idx);
            out.insert(node, subset);
        }
        out
    }
}

#[derive(Debug, Clone)]
pub struct LouvainError;
impl fmt::Display for LouvainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Given a graph with negative edges")
    }
}

/// Performs one level of the Louvain algorithm.
///
/// ## Arguments:
/// * `partition`: The current partition of the input graph
/// * `m`: Total weight of the edges of `graph`
/// * `resolution` : controls whether the algorithm favors larger communities (`resolution < 1`) or smaller communities (`resolution < 1`)
/// * `gain_threshold` : minimum acceptable increase in modularity
/// * 'seed' : seed for RNG that determines the order in which we consider moving each node into a neighboring community
///
/// ## Returns:
/// * true if it was possible to meet the specified `gain_threshold` by combining nodes into communities.
fn one_level_undirected<G>(
    partition: &mut Partition<G>,
    total_edge_weight: f64,
    resolution: f64,
    gain_threshold: f64,
    seed: Option<u64>,
) -> Result<bool, LouvainError>
where
    G: Modularity,
    <G as Data>::EdgeWeight: ModularityEdgeWeight,
    <G as GraphBase>::NodeId: ModularityNodeId,
{
    let inner_graph = partition.to_inner_graph();

    let node_count = inner_graph.node_count();

    let degrees = inner_graph.degrees();
    let mut total_community_degrees = degrees.clone();

    if node_count <= 1 {
        return Ok(false);
    }

    // place each node into its own community
    let mut node_to_community: Vec<usize> = (0..node_count).collect();

    // Keep track of the total modularity gain during this level of the
    // algorithm. Note that we actually keep track of m * delta, where
    // delta is the change in modularity. Later we will compare this
    // against m * gain_threshold.
    let mut total_gain = 0.0;

    let mut performed_move = true;
    while performed_move {
        performed_move = false;

        // randomize node order based on seed
        let mut rng_state = seed.unwrap_or(12345);
        let mut next_rand = || {
            rng_state = rng_state.wrapping_mul(1103515245).wrapping_add(12345);
            rng_state
        };
        let mut node_order: Vec<usize> = (0..node_count).collect();
        // Fisher-Yates shuffle
        for i in (1..node_order.len()).rev() {
            let j = (next_rand() as usize) % (i + 1);
            node_order.swap(i, j);
        }

        // Try moving each node into a neighboring community. For each node,
        // select the neighboring community that gives the largest
        // increase in modularity (if any)
        for node in node_order {
            let neighbor_weights = inner_graph.neighbor_community_weights(node, &node_to_community);

            let mut best_gain = 0.0;
            let initial_community = node_to_community[node];
            let degree = degrees[node];
            let mut best_com = initial_community;

            total_community_degrees[best_com] -= degree;

            let community_delta = |c: usize| -> Result<f64, LouvainError> {
                let weight = neighbor_weights.get(&c).copied().unwrap_or(0.0);

                // Early return for negative weights
                if weight < 0.0 {
                    return Err(LouvainError);
                }

                Ok(-weight
                    + 0.5 * resolution * total_community_degrees[c] * degree / total_edge_weight)
            };

            let remove_cost = community_delta(best_com)?;

            for &nbr_com in neighbor_weights.keys() {
                let gain = remove_cost - community_delta(nbr_com)?;
                if gain > best_gain {
                    best_gain = gain;
                    best_com = nbr_com;
                }
            }

            total_community_degrees[best_com] += degree;

            if best_com != initial_community {
                performed_move = true;
                total_gain += best_gain;
                node_to_community[node] = best_com;
            }
        }
    }

    if total_gain < total_edge_weight * gain_threshold {
        return Ok(false);
    }

    // Compute the resulting new partition of the input graph
    let input_graph = &partition.graph;
    let mut final_index = HashMap::new();
    let mut next_com = 0;
    let mut updated_partition: Vec<usize> = vec![0; input_graph.node_count()];

    for n in input_graph.node_identifiers() {
        let prev_com = partition.subset_idx(n);
        let inner_com = node_to_community[prev_com];
        let new_com = match final_index.get(&inner_com) {
            Some(&c) => c,
            None => {
                let c = next_com;
                final_index.insert(inner_com, c);
                next_com += 1;
                c
            }
        };
        updated_partition[input_graph.to_index(n)] = new_com;
    }
    partition.update(updated_partition);

    Ok(true)
}

/// Runs the Louvain community detection algorithm to detect the communities present in the input graph.
/// Assigns each node a community, then combines those communities using the `Modularity` until the
/// maximum value is reached
/// 
/// Errors if it is given a graph with a negative edge
///
/// # Arguments
/// * `graph`: The input graph
/// * `resolution` : controls whether the algorithm favors larger communities (`resolution < 1`) or smaller communities (`resolution < 1`) (1.0 is a good default)
/// * `gain_threshold` : minimum acceptable increase in modularity at each level. The algorithm will
///   terminate if it is not possible to meet this threshold by performing another level of aggregation. (0.001 is a good default)
/// * `max_level` : Maximum number of levels (aggregation steps) to perform
/// * `seed` : seed for RNG that determines the order in which we consider moving each node into a neighboring community
/// 
/// # Returns
/// * A custom struct containing information about the communities, or an error if the input
/// graph has a negative edge (currently unsupported)
/// 
/// # Time Complexity
/// * Time complexity: **O(n log(n))**
/// * Auxiliary space: **O(n)**
/// 
/// # Examples
/// ```
/// // create graph with nodes and edges
/// let mut graph: Graph<u32, f64, petgraph::Undirected> = UnGraph::new_undirected();
/// let nodes: Vec<_> = (0..4).map(|i| graph.add_node(i)).collect();
/// 
/// // Add edges with weights
/// graph.add_edge(nodes[0], nodes[1], 1.0);
/// graph.add_edge(nodes[0], nodes[2], 1.0);
/// graph.add_edge(nodes[2], nodes[3], 1.0);
/// 
/// // Currently the graph looks like this:
/// // 0 - 1
/// // |
/// // 2 - 3
/// 
/// let communities = louvain_communities(&graph, 1.0, 0.001, None, None).unwrap();
/// 
/// // two communities: ((0,1), (2,3))
/// assert!(communities.communities.len() == 2);
/// assert!(communities.communities.contains(&HashSet::from([nodes[0], nodes[1]])));
/// assert!(communities.communities.contains(&HashSet::from([nodes[2], nodes[3]])));
/// ```
pub fn louvain_communities<G>(
    graph: G,
    resolution: f64,
    gain_threshold: f64,
    max_level: Option<u32>,
    seed: Option<u64>,
) -> Result<LouvainReturn<G>, LouvainError>
where
    G: Modularity,
    <G as Data>::EdgeWeight: ModularityEdgeWeight,
    <G as GraphBase>::NodeId: ModularityNodeId,
{
    let mut partition = Partition::new(&graph);

    let m = total_edge_weight(&graph);

    let mut n_levels = 0;
    while one_level_undirected(&mut partition, m, resolution, gain_threshold, seed)? && m != 0.0 {
        if let Some(limit) = max_level {
            n_levels += 1;
            if n_levels >= limit {
                break;
            }
        }
    }

    let communities = partition.to_vec_of_hashsets();
    let modularity = modularity(graph, &communities, resolution).unwrap_or(0.0);

    Ok(LouvainReturn {
        communities,
        nodes_to_communities: partition.to_hashmap(),
        modularity,
    })
}
