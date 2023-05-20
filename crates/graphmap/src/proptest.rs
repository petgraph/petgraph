use core::fmt::Debug;

use petgraph_core::edge::EdgeType;
use petgraph_proptest::{default::graph_strategy_from_vtable, vtable::VTable};
use proptest::{arbitrary::Arbitrary, prelude::BoxedStrategy, strategy::Strategy};

use crate::{GraphMap, NodeTrait};

fn add_edge_no_return<N, E, Ty>(graph: &mut GraphMap<N, E, Ty>, a: N, b: N, weight: E)
where
    N: NodeTrait,
    Ty: EdgeType,
{
    graph.add_edge(a, b, weight);
}

// GraphMap does not implement `Create` or `Build`, so we need to use the vtable.
fn create_vtable<N, E, Ty>() -> VTable<GraphMap<N, E, Ty>, N, N, E>
where
    N: NodeTrait,
    Ty: EdgeType,
{
    VTable {
        with_capacity: GraphMap::with_capacity,
        add_node: GraphMap::add_node,
        add_edge: add_edge_no_return::<N, E, Ty>,
    }
}

impl<N, E, Ty> Arbitrary for GraphMap<N, E, Ty>
where
    N: NodeTrait + Arbitrary + Clone + Debug + 'static,
    E: Arbitrary + Debug + 'static,
    Ty: EdgeType + 'static,
{
    type Parameters = ();
    // impl Strategy<Value = Self> is nightly, and therefore not usable here.
    // TODO: revisit once impl_trait_in_assoc_type is stable. (https://github.com/rust-lang/rust/issues/63063)
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        graph_strategy_from_vtable(
            create_vtable::<N, E, Ty>(),
            true,
            false,
            0..=usize::MAX,
            None,
        )
        .boxed()
    }
}
