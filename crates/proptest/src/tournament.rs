use alloc::vec::Vec;
use core::fmt::Debug;

use itertools::Itertools;
use petgraph_core::{
    data::{Build, Create},
    visit::Data,
};
use proptest::{
    collection::{vec, SizeRange},
    prelude::*,
};

use crate::{vtable, vtable::VTable};

/// ***Internal API***.
///
/// Creates a strategy for a graph type from a vtable.
///
/// This API is not stable and may change at any time.
pub fn graph_tournament_strategy_from_vtable<G, NodeWeight, NodeIndex, EdgeWeight>(
    vtable: VTable<G, NodeWeight, NodeIndex, EdgeWeight>,
    node_range: impl Into<SizeRange>,
) -> impl Strategy<Value = G>
where
    G: Debug,
    NodeIndex: Copy,
    NodeWeight: Arbitrary + Clone + Debug,
    EdgeWeight: Arbitrary + Debug,
{
    vec(any::<NodeWeight>(), node_range.into())
        .prop_flat_map(|nodes| {
            // generate a vec of all possible mappings between two nodes
            let endpoints: Vec<_> = (0..nodes.len())
                .permutations(2)
                .filter(|endpoints| endpoints[0] < endpoints[1])
                .collect();

            // generate the direction of all edges (boolean)
            let direction = vec(any::<bool>(), endpoints.len());
            let weights = vec(any::<EdgeWeight>(), endpoints.len());

            (
                Just(nodes),
                (Just(endpoints), direction, weights).prop_map(
                    move |(endpoints, direction, weight)| {
                        let edges = endpoints
                            .into_iter()
                            .zip(direction.into_iter())
                            .zip(weight.into_iter());

                        edges
                            .map(move |((endpoints, direction), weight)| {
                                let (start, end) = if direction {
                                    (endpoints[0], endpoints[1])
                                } else {
                                    (endpoints[1], endpoints[0])
                                };

                                (start, end, weight)
                            })
                            .collect::<Vec<_>>()
                    },
                ),
            )
        })
        .prop_map(move |(nodes, edges)| {
            let mut graph = (vtable.with_capacity)(nodes.len(), edges.len());

            let nodes: Vec<_> = nodes
                .into_iter()
                .map(|weight| (vtable.add_node)(&mut graph, weight))
                .collect();

            for (start, end, weight) in edges {
                (vtable.add_edge)(&mut graph, nodes[start], nodes[end], weight);
            }

            graph
        })
}

pub fn graph_tournament_strategy<G>(node_range: impl Into<SizeRange>) -> impl Strategy<Value = G>
where
    G: Create + Build + Data + Debug,
    G::NodeWeight: Arbitrary + Clone + Debug,
    G::EdgeWeight: Arbitrary + Debug,
{
    let vtable = vtable::create::<G>();

    graph_tournament_strategy_from_vtable(vtable, node_range)
}
