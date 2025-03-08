use crate::data::DataMap;

use crate::visit::{IntoNeighbors, IntoNodeReferences, NodeCount, NodeRef};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// Finds for a given node N, the neighborhood collecting the nodes that are far from N by at most k nodes.
fn k_neighborhood<G>(graph: G, node: G::NodeId, k: usize) -> Vec<G::NodeId>
where
    G: IntoNeighbors,
{
    if k == 0 {
        vec![]
    } else if k == 1 {
        return graph.neighbors(node).collect();
    } else {
        let mut neighbor_nodes = graph.neighbors(node).collect::<Vec<G::NodeId>>();
        let mut collector = Vec::new();
        for node_id in neighbor_nodes.iter() {
            collector.extend(k_neighborhood(graph, *node_id, k - 1));
        }
        neighbor_nodes.extend(collector);
        return neighbor_nodes;
    }
}

/// \[Generic\] Label propagation.
///
/// Labels nodes by propagating available node labels through the graph following the edges.
///
/// The predicted labels and their stability depends on the order of nodes iteration, that is
/// the implementation of [IntoNodeReferences].
///
/// # Example
/// ```rust
/// use petgraph::algo::label_propagation;
/// use petgraph::prelude::{Graph, NodeIndex};
///
/// // Example adapted from Neo4j documentation
/// let mut graph = Graph::<Option<&str>, ()>::with_capacity(7, 10);
/// graph.add_node(Some("A")); // Node 0 with label "A"
/// graph.add_node(None); // Node 1 without label
/// graph.add_node(None); // Node 2 without label
/// graph.add_node(None); // Node 3 without label
/// graph.add_node(None); // Node 4 without label
/// graph.add_node(Some("B")); // Node 5 with label "B"
/// graph.add_node(None); // Node 6 without label
/// graph.extend_with_edges(&[
///     (0, 1),
///     (0, 3),
///     (5, 4),
///     (1, 2),
///     (4, 5),
///     (2, 0),
///     (0, 2),
///     (1, 0),
///     (2, 1),
///     (3, 4),
/// ]);
///
/// // A representation of the above graph.
/// // (2,None) <-----
/// //  ^            |
/// //  |            |
/// //  v            v
/// // (0,A) <--> (1,None)
/// //  |
/// //  v
/// // (3,None)
/// //  |
/// //  v
/// // (4,None) <--> (5,B)  (6,None)
///
/// // Notice Node 6 is not linked to any node.
///
/// // The goal is to predict labels for nodes without label.
///
/// let predicted_labels = label_propagation(&graph, &[Some("A"), Some("B")], 2, 1);
/// let expected_labels = std::collections::HashMap::from([
///     (NodeIndex::new(1), Some("A")),
///     (NodeIndex::new(4), Some("B")),
///     (NodeIndex::new(3), Some("B")),
///     (NodeIndex::new(2), Some("A")),
/// ]); // Node 6 has no label prediction since it is not linked to any node.
///
/// assert_eq!(expected_labels, predicted_labels);
/// ```
pub fn label_propagation<G>(
    graph: G,
    labels: &[G::NodeWeight],
    k: usize,
    nb_iter: usize,
) -> HashMap<G::NodeId, G::NodeWeight>
where
    G: IntoNodeReferences + NodeCount + IntoNeighbors + DataMap,
    G::NodeId: Hash + Eq,
    G::NodeWeight: PartialEq + Clone + Missing,
{
    let mut predicted_labels = HashMap::new();
    if graph.node_count() == 0 || labels.is_empty() {
        return predicted_labels;
    }
    for _ in 0..nb_iter {
        for node in graph.node_references() {
            // Ignore nodes with label.
            if predicted_labels.contains_key(&node.id()) || node.weight().is_missing() {
                let mut label_frequencies = labels
                    .iter()
                    .map(|label| Tracker::new(label.clone(), 0))
                    .collect::<Vec<Tracker<G::NodeWeight>>>();
                // Find the most frequent label in the neighbourhood of the current node.
                for neighbor in k_neighborhood(graph, node.id(), k) {
                    let mut neighbor_label = graph.node_weight(neighbor).unwrap();
                    if neighbor_label.is_missing() && predicted_labels.contains_key(&neighbor) {
                        neighbor_label = &predicted_labels[&neighbor];
                    };
                    label_frequencies.iter_mut().for_each(|labelf| {
                        labelf.freq += usize::from(
                            (labelf.label == neighbor_label.clone())
                                && !neighbor_label.is_missing(),
                        )
                    });
                }
                label_frequencies.sort();
                // Propagate the most frequent label if any.
                let most_frequent = label_frequencies.last().unwrap(); // label_frequencies is not empty at this stage, so it is safe to .unwrap()
                if most_frequent.freq > 0 {
                    predicted_labels.insert(node.id(), most_frequent.label.clone());
                }
            }
        }
    }
    predicted_labels
}

/// Helper trait for types with possibly missing values.
pub trait Missing {
    type Wrapped;
    fn is_missing(&self) -> bool;
}
impl<T> Missing for Option<T> {
    type Wrapped = T;
    fn is_missing(&self) -> bool {
        self.is_none()
    }
}

/// Helper to compare node labels by their frequencies.
#[derive(Debug)]
struct Tracker<L> {
    label: L,
    freq: usize,
}

impl<L> Tracker<L> {
    pub fn new(label: L, freq: usize) -> Self {
        Self { label, freq }
    }
}

impl<L> PartialEq for Tracker<L> {
    fn eq(&self, other: &Self) -> bool {
        self.freq.eq(&other.freq)
    }
}
impl<L> Eq for Tracker<L> {}
impl<L> PartialOrd for Tracker<L> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}
impl<L> Ord for Tracker<L> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.freq.cmp(&other.freq)
    }
}
