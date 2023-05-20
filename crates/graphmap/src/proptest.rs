use core::fmt::Debug;

use petgraph_core::edge::EdgeType;
use proptest::{arbitrary::Arbitrary, prelude::BoxedStrategy};
use petgraph_proptest::default::graph_strategy;

use crate::GraphMap;

impl<N, E, Ty> Arbitrary for GraphMap<N, E, Ty>
where
    N: Arbitrary + Clone + Debug,
    E: Arbitrary + Debug,
    Ty: EdgeType,
{
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        graph_strategy(true, false, 0..=usize::MAX).boxed()
    }
}
