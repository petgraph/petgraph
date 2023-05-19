use alloc::sync::Arc;
use core::fmt::Debug;

use petgraph_core::{edge::EdgeType, index::IndexType};
use petgraph_proptest::default::graph_strategy;
use proptest::{
    arbitrary::Arbitrary,
    collection::btree_set,
    prelude::{Just, Strategy},
};

use crate::{stable::StableGraph, Graph, NodeIndex};

/// `Arbitrary` for `Graph` creates a graph by selecting a node count
///
/// The result will be simple graph or digraph, self loops
/// possible, no parallel edges.
///
/// The exact properties of the produced graph is subject to change.
///
/// Requires crate feature `"proptest"`
impl<N, E, Ty, Ix> Arbitrary for StableGraph<N, E, Ty, Ix>
where
    N: Arbitrary + Debug + Clone,
    E: Arbitrary,
    Ty: EdgeType + Send + 'static,
    Ix: IndexType + Send,
{
    type Parameters = ();
    type Strategy = Arc<impl Strategy<Value = Self>>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        Arc::new(
            graph_strategy(true, false)
                .prop_flat_map(|graph| {
                    let nodes = graph.node_count();

                    // select a batch of random indices to remove (unique)
                    (Just(graph), btree_set(0..nodes, 0..nodes))
                })
                .prop_map(|(graph, remove)| {
                    for index in remove {
                        graph.remove_node(NodeIndex::new(Ix::from_usize(index)))
                    }
                }),
        )
    }
}
