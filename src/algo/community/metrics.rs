use crate::visit::{
    Data, EdgeRef, GraphBase, GraphProp, IntoEdgeReferences, IntoEdgesDirected, IntoNodeReferences,
    NodeCount, NodeIndexable,
};
use alloc::{vec, vec::Vec};
use core::{error::Error, fmt, hash::Hash};
use hashbrown::HashSet;

/// Traits for graphs to compute modularity
/// and apply the Louvain community detection method.
pub trait ModularityEdgeWeight: Into<f64> + Copy {}
impl<E: Into<f64> + Copy> ModularityEdgeWeight for E {}
pub trait ModularityNodeId: Hash + Eq + Copy {}
impl<N: Hash + Eq + Copy> ModularityNodeId for N {}

pub trait Modularity:
    Data
    + GraphProp
    + IntoEdgeReferences
    + NodeCount
    + IntoNodeReferences
    + NodeIndexable
    + IntoEdgesDirected
{
}
impl<
        G: Data
            + GraphProp
            + IntoEdgeReferences
            + NodeCount
            + IntoNodeReferences
            + NodeIndexable
            + IntoEdgesDirected,
    > Modularity for G
{
}

/// Struct representing a partition of a graph as a vector
/// `[s_0, ... s_n]`, where `n` is the number of nodes in
/// the graph and node `i` belongs to subset `s_i`.
pub struct Partition<'g, G>
where
    G: Modularity,
    <G as Data>::EdgeWeight: ModularityEdgeWeight,
    <G as GraphBase>::NodeId: ModularityNodeId,
{
    pub graph: &'g G,
    pub n_subsets: usize,
    pub node_to_subset: Vec<usize>,
}

impl<'g, G: Modularity> Partition<'g, G>
where
    <G as Data>::EdgeWeight: ModularityEdgeWeight,
    <G as GraphBase>::NodeId: ModularityNodeId,
{
    /// Creates a partition where each node of the input graph is placed
    /// into its own subset, e.g. for the first step of the Louvain algorithm.
    pub fn new(graph: &'g G) -> Partition<'g, G> {
        Partition {
            graph,
            n_subsets: graph.node_count(),
            node_to_subset: (0..graph.node_count()).collect(),
        }
    }

    /// Creates a `Partition` from sets of graph nodes. Checks whether the
    /// sets actually form a partition of the input graph.
    pub fn from_subsets(
        graph: &'g G,
        subsets: &[HashSet<G::NodeId>],
    ) -> Result<Partition<'g, G>, NotAPartitionError> {
        let mut seen = vec![false; graph.node_count()];

        let mut node_to_subset = vec![0; graph.node_count()];

        for (ii, v) in subsets.iter().enumerate() {
            for &node in v {
                let idx = graph.to_index(node);
                if seen[idx] {
                    // argument `communities` contains a duplicate node
                    return Err(NotAPartitionError {});
                }
                node_to_subset[idx] = ii;
                seen[idx] = true;
            }
        }

        if !seen.iter().all(|&t| t) {
            return Err(NotAPartitionError {});
        }

        Ok(Partition::<'g, G> {
            graph,
            n_subsets: subsets.len(),
            node_to_subset,
        })
    }

    /// Returns the index of the subset that contains `node`.
    pub fn subset_idx(&self, node: G::NodeId) -> usize {
        let idx = self.graph.to_index(node);
        self.node_to_subset[idx]
    }

    /// Returns the modularity of the graph with the current partition.
    pub fn modularity(&self, resolution: f64) -> f64 {
        let mut internal_weights = vec![0.0; self.n_subsets];
        let mut outgoing_weights = vec![0.0; self.n_subsets];

        let m: f64 = total_edge_weight(self.graph);
        if m == 0.0 {
            return 0.0;
        }

        let directed = self.graph.is_directed();
        let mut incoming_weights = if directed {
            Some(vec![0.0; self.n_subsets])
        } else {
            None
        };

        for edge in self.graph.edge_references() {
            let (a, b) = (edge.source(), edge.target());
            let (c_a, c_b) = (self.subset_idx(a), self.subset_idx(b));
            let w: f64 = (*edge.weight()).into();
            if c_a == c_b {
                internal_weights[c_a] += w;
            }
            outgoing_weights[c_a] += w;
            if let Some(ref mut incoming) = incoming_weights {
                incoming[c_b] += w;
            } else {
                outgoing_weights[c_b] += w;
            }
        }

        let sigma_internal: f64 = internal_weights.iter().sum();

        let sigma_total_squared: f64 = if let Some(incoming) = incoming_weights {
            incoming
                .iter()
                .zip(outgoing_weights.iter())
                .map(|(&x, &y)| x * y)
                .sum()
        } else {
            outgoing_weights.iter().map(|&x| x * x).sum::<f64>() / 4.0
        };
        (sigma_internal - resolution * sigma_total_squared / m) / m
    }
}

/// Computes the modularity of a graph, given a partition of its nodes.
///
/// ## Arguments:
/// * `graph` - The input graph
/// * `communities` - Sets of nodes that form a partition of `graph`
/// * `resolution` - Controls the relative weight of intra-community and inter-community edges
///
/// ## Returns:
/// * Result of the modularity of the graph. Can error if the given `communities` are not correct
pub fn modularity<G>(
    graph: G,
    communities: &[HashSet<G::NodeId>],
    resolution: f64,
) -> Result<f64, NotAPartitionError>
where
    G: Modularity,
    <G as Data>::EdgeWeight: ModularityEdgeWeight,
    <G as GraphBase>::NodeId: ModularityNodeId,
{
    let partition = Partition::from_subsets(&graph, communities)?;
    Ok(partition.modularity(resolution))
}

pub fn total_edge_weight<G>(graph: &G) -> f64
where
    G: Modularity,
    <G as Data>::EdgeWeight: ModularityEdgeWeight,
{
    graph
        .edge_references()
        .map(|edge| *edge.weight())
        .fold(0.0, |s, e| s + e.into())
}

#[derive(Debug, PartialEq, Eq)]
pub struct NotAPartitionError;
impl Error for NotAPartitionError {}
impl fmt::Display for NotAPartitionError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "The input subsets do not form a partition of the input graph."
        )
    }
}
