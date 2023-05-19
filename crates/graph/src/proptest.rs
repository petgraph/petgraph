use alloc::{boxed::Box, sync::Arc};
use core::fmt::Debug;

use petgraph_core::{edge::EdgeType, index::IndexType};
use petgraph_proptest::default::graph_strategy;
use proptest::{arbitrary::Arbitrary, strategy::Strategy};

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
    N: Arbitrary + Debug + Clone,
    E: Arbitrary,
    Ty: EdgeType + Send + 'static,
    Ix: IndexType + Send,
{
    type Parameters = ();
    type Strategy = Arc<impl Strategy<Value = Self>>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        Arc::new(graph_strategy(true, false))
    }
}
