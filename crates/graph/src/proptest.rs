use alloc::{boxed::Box, sync::Arc};
use core::fmt::Debug;

use petgraph_core::{edge::EdgeType, index::IndexType};
use petgraph_proptest::default::graph_strategy;
use proptest::{arbitrary::Arbitrary, prelude::BoxedStrategy, strategy::Strategy};

use crate::Graph;

/// `Arbitrary` for `Graph` creates a graph by selecting a node count
///
/// The result will be simple graph or digraph, self loops
/// possible, no parallel edges.
///
/// The exact properties of the produced graph is subject to change.
///
/// Requires crate feature `"proptest"`
impl<N, E, Ty, Ix> Arbitrary for Graph<N, E, Ty, Ix>
where
    N: Arbitrary + Debug + Clone + 'static,
    E: Arbitrary + Debug + 'static,
    Ty: EdgeType + Send + 'static,
    Ix: IndexType + Send,
{
    type Parameters = ();
    // impl Strategy<Value = Self> is nightly, and therefore not usable here.
    // TODO: revisit once impl_trait_in_assoc_type is stable. (https://github.com/rust-lang/rust/issues/63063)
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        graph_strategy(true, false, 0..=Ix::MAX.as_usize(), None).boxed()
    }
}
