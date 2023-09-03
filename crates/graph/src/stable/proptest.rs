use core::fmt::Debug;
use std::{collections::BTreeSet, sync::Arc};

use petgraph_core::{edge::EdgeType, index::IndexType};
use petgraph_proptest::default::graph_strategy;
use proptest::{
    arbitrary::Arbitrary,
    bits::BitSetLike,
    collection::{btree_set, SizeRange},
    prelude::{BoxedStrategy, Just, Strategy},
    strategy::{LazyJust, TupleUnion},
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
        // `MAX` is not a valid node index.
        graph_strategy(
            true,
            false,
            0..Ix::MAX.as_usize(),
            Some(Arc::new(|max| {
                SizeRange::new(0..=usize::min(max.pow(2), Ix::MAX.as_usize() - 1))
            })),
        )
        .prop_flat_map(|graph: Self| {
            let nodes = graph.node_count();
            let is_empty = nodes == 0;

            let removed_nodes = TupleUnion::new((
                (u32::from(is_empty), Arc::new(LazyJust::new(BTreeSet::new))),
                (
                    u32::from(!is_empty),
                    Arc::new(btree_set(0..nodes.len(), 0..nodes.len())),
                ),
            ));

            // select a batch of random indices to remove (unique)
            (Just(graph), removed_nodes)
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
