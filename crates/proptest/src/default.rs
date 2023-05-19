use alloc::{collections::btree_map::Entry, sync::Arc, vec::Vec};
use core::{
    cmp::Ordering,
    fmt::Debug,
    hash::{Hash, Hasher},
};

use petgraph_core::{
    data::{Build, Create},
    visit::Data,
};
use proptest::{
    arbitrary::{any, arbitrary, Arbitrary, StrategyFor},
    collection::{btree_set, vec},
    prop_compose, prop_oneof,
    strategy::{Just, NewTree, Strategy, TupleUnion},
    test_runner::TestRunner,
};

#[derive(Debug)]
struct Edge<T, U> {
    start: T,
    end: T,
    weight: U,
}

impl<T, U> PartialEq for Edge<T, U>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (&self.start, &self.end).eq(&(&other.start, &other.end))
    }
}

impl<T, U> Eq for Edge<T, U> where T: Eq {}

impl<T, U> PartialOrd for Edge<T, U>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        (&self.start, &self.end).partial_cmp(&(&other.start, &other.end))
    }
}

impl<T, U> Ord for Edge<T, U>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        (&self.start, &self.end).cmp(&(&other.start, &other.end))
    }
}

impl<T, U> Hash for Edge<T, U>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        (&self.start, &self.end).hash(state)
    }
}

fn edge_strategy<T, U, S1, S2>(endpoints: S1, weight: Arc<S2>) -> impl Strategy<Value = Edge<T, U>>
where
    T: PartialOrd + Hash + Debug,
    U: Debug,
    S1: Strategy<Value = (T, T)>,
    S2: Strategy<Value = U>,
{
    endpoints.prop_flat_map(move |(start, end)| {
        Arc::clone(&weight).prop_map(move |weight| Edge { start, end, weight })
    })
}

pub fn graph_strategy<G>(self_loops: bool, parallel_edges: bool) -> impl Strategy<Value = G>
where
    G: Create + Build + Data + Debug,
    G::NodeWeight: Arbitrary + Clone + Debug,
    G::EdgeWeight: Arbitrary + Debug,
{
    vec(any::<G::NodeWeight>(), 0..usize::MAX)
        .prop_flat_map(|nodes: Vec<G::NodeWeight>| {
            let nodes_len = nodes.len();

            // TODO: make configurable
            // generate an edge where no self edges are allowed, this allows the use in
            // a lot more graphs
            let edge_endpoints = (0..nodes_len).prop_flat_map(move |start| {
                (0..start)
                    .prop_union(start..nodes_len)
                    .prop_map(move |end| (start, end))
            });

            // TODO: make configurable
            // using btree_set here, as while it is slower, it is usable in no-std
            (
                Just(nodes),
                btree_set(
                    edge_strategy(edge_endpoints, Arc::new(any::<G::EdgeWeight>())),
                    0..nodes_len.pow(2),
                ),
            )
        })
        .prop_map(move |(nodes, edges)| {
            let mut graph = G::with_capacity(nodes.len(), edges.len());

            // reference table for edges
            let nodes: Vec<_> = nodes
                .into_iter()
                .map(|weight| graph.add_node(weight))
                .collect();

            for Edge { start, end, weight } in edges {
                graph.add_edge(nodes[start], nodes[end], weight);
            }

            graph
        })
}

#[cfg(test)]
mod tests {
    use petgraph::Graph;
    use proptest::strategy::check_strategy_sanity;

    use crate::default::graph_strategy;

    #[test]
    fn sanity() {
        check_strategy_sanity(graph_strategy::<Graph<(), ()>>(false, false), None);
    }
}