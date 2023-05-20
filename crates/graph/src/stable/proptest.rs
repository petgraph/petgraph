use core::fmt::Debug;

use petgraph_core::{edge::EdgeType, index::IndexType};
use petgraph_proptest::default::graph_strategy;
use proptest::{
    arbitrary::Arbitrary,
    collection::btree_set,
    prelude::{BoxedStrategy, Just, Strategy},
};

use crate::{stable::StableGraph, NodeIndex};

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
    N: Arbitrary + Debug + Clone + 'static,
    E: Arbitrary + Debug + Clone + 'static,
    Ty: EdgeType + Send + 'static,
    Ix: IndexType + Send,
{
    type Parameters = ();
    // impl Strategy<Value = Self> is nightly, and therefore not usable here.
    // TODO: revisit once impl_trait_in_assoc_type is stable. (https://github.com/rust-lang/rust/issues/63063)
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        graph_strategy(true, false, 0..=Ix::MAX.as_usize(), None)
            .prop_flat_map(|graph: Self| {
                let nodes = graph.node_count();

                // select a batch of random indices to remove (unique)
                (Just(graph), btree_set(0..nodes, 0..nodes))
            })
            .prop_map(|(mut graph, remove)| {
                for index in remove {
                    graph.remove_node(NodeIndex(Ix::from_usize(index)));
                }

                graph
            })
            .boxed()
    }
}
