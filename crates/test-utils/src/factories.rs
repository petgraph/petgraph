use std::marker::PhantomData;

use petgraph::{
    core::{
        edge::{Directed, Undirected},
        visit::GraphProp,
    },
    data::Build,
    graph::Graph,
    visit::NodeIndexable,
    EdgeType,
};

/// Petersen A and B are isomorphic
///
/// http://www.dharwadker.org/tevet/isomorphism/
const PETERSEN_A: &str =
    include_str!("../../algorithms/tests/snapshots/isomorphism/petersen_a.txt");
const PETERSEN_B: &str =
    include_str!("../../algorithms/tests/snapshots/isomorphism/petersen_b.txt");

/// An almost full set, isomorphic
const FULL_A: &str = include_str!("../../algorithms/tests/snapshots/isomorphism/full_a.txt");
const FULL_B: &str = include_str!("../../algorithms/tests/snapshots/isomorphism/full_b.txt");

/// Praust A and B are not isomorphic
const PRAUST_A: &str = include_str!("../../algorithms/tests/snapshots/isomorphism/praust_a.txt");
const PRAUST_B: &str = include_str!("../../algorithms/tests/snapshots/isomorphism/praust_b.txt");

const BIGGER: &str = "
 0 0 0 0 0 0 0 0 1 0 1 0 0 0 0 0 0 0 1 1 0 1 1 1 1 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 1 0 0 0 1 1 0 0 0 0 0 0 0 1 0 1 0 1 1 0 1 0 0 0 1 0 0 0 0 0 0 0 0 0 0
 0 1 0 1 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 1 1 0 1 0 0 1 0 0 0 1 0 0 0 0 0 0 0 0 0
 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 1 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 1 0 1 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 1 1 1 0 1 0 0
 0 0 0 0 0 1 0 0 0 0 0 0 1 0 0 1 1 0 0 0 0 0 0 0 0 1 0 0 0 0 0 0 1 0 1 1 1 0 0 0
 0 0 0 0 0 0 1 0 0 0 0 0 1 1 0 1 0 0 0 1 0 0 0 0 0 0 1 0 0 0 0 0 1 1 0 1 0 0 0 1
 0 0 0 0 0 0 0 1 0 0 0 0 1 1 1 0 0 0 1 0 0 0 0 0 0 0 0 1 0 0 0 0 1 1 1 0 0 0 1 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 1 1 1 0 1 1 0 0 0 0 0 1 0 0 0 0 1 0 0 0 1 1 1
 0 0 0 0 1 0 0 0 0 0 0 0 1 0 0 0 1 0 1 1 0 1 1 1 0 0 0 0 0 1 0 0 1 0 0 0 1 0 1 1
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 1 1 1 0 1
 0 0 0 0 0 0 0 0 0 0 0 1 0 0 1 0 1 1 1 1 0 0 0 0 0 0 0 0 0 0 0 1 0 0 1 0 1 1 1 0
 0 1 1 0 1 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 1 1 1 1 1 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0
 1 0 1 0 0 1 0 0 0 1 0 0 0 0 0 0 0 1 1 0 1 0 1 1 0 1 0 0 0 1 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 1 0 1 1 0 1 0 0 0 0 0 0 1 0 0 0 0 0 0 0 1 0 1 1 0 1 0 0 0 0 0 0 1 0 0 0 0 0
 0 0 0 0 1 1 1 0 0 0 0 0 0 0 0 1 0 0 0 0 0 0 0 1 1 1 1 0 0 0 0 0 0 0 0 1 0 0 0 0
 1 0 0 0 0 0 0 0 0 1 1 1 0 0 0 0 1 0 0 0 1 0 0 0 0 0 0 0 0 1 1 1 0 0 0 0 1 0 0 0
 0 1 0 0 0 0 0 0 1 0 1 1 0 0 0 0 0 1 0 0 0 1 0 0 0 0 0 0 1 0 1 1 0 0 0 0 0 1 0 0
 0 0 1 0 0 0 0 0 1 1 0 1 0 0 0 0 0 0 1 0 0 0 1 0 0 0 0 0 1 1 0 1 0 0 0 0 0 0 1 0
 0 0 0 1 0 0 0 0 1 1 1 0 0 0 0 0 0 0 0 1 0 0 0 1 0 0 0 0 1 1 1 0 0 0 0 0 0 0 0 1
 0 0 0 0 1 0 0 0 0 0 0 0 0 1 1 1 0 1 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 1 1 1 0 1 0 0
 0 0 0 0 0 1 0 0 0 0 0 0 1 0 1 1 1 0 0 0 0 0 0 0 0 1 0 0 0 0 0 0 1 0 1 1 1 0 0 0
 0 1 0 0 0 0 1 0 0 0 0 0 1 1 0 1 0 0 0 1 0 0 0 0 0 0 1 0 0 0 0 0 1 1 0 1 0 0 0 1
 0 1 0 0 0 0 0 1 0 0 0 0 1 1 1 0 0 0 1 0 0 0 0 0 0 0 0 1 0 0 0 0 1 1 1 0 0 0 1 0
 0 1 0 0 0 0 0 0 1 0 0 0 0 1 0 0 0 1 1 1 0 0 0 0 0 0 0 0 1 0 0 0 0 1 0 0 0 1 1 1
 0 1 0 0 0 0 0 0 0 1 0 0 1 0 0 0 1 0 1 1 0 0 0 0 0 0 0 0 0 1 0 0 1 0 0 0 1 0 1 1
 0 1 0 0 0 0 0 0 0 0 1 0 0 0 0 1 1 1 0 1 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 1 1 1 0 1
 0 1 0 0 0 0 0 0 0 0 0 1 0 0 1 0 1 1 1 0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 1 0 1 1 1 0
";

/// A random bipartite graph.
const BIPARTITE: &str = "
 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 1
 0 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 1 0 1 1
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 1 0
 0 0 0 0 0 0 0 0 0 0 0 1 0 0 1 0 1 1 1 0
 0 0 0 0 0 0 0 0 0 0 0 0 1 0 1 0 0 1 1 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 0 1 0 1 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 1 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 1 1 1 0
 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 1 0 1 1 1
 0 0 0 0 0 0 0 0 0 0 1 0 0 0 0 1 0 0 1 1
 1 0 0 0 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0
 0 1 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 1 1 0 0 0 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 0 0 0 0 0 1 1 1 0 0 0 0 0 0 0 0 0 0
 0 1 0 1 0 1 0 1 0 0 0 0 0 0 0 0 0 0 0 0
 0 0 1 1 1 0 1 1 1 0 0 0 0 0 0 0 0 0 0 0
 0 1 1 1 1 1 1 1 1 1 0 0 0 0 0 0 0 0 0 0
 1 1 0 0 0 0 0 0 1 1 0 0 0 0 0 0 0 0 0 0
";

/// Parse a text adjacency matrix format into a directed graph
fn parse_graph<G>(s: &str) -> G
where
    G: Default + Build<NodeWeight = (), EdgeWeight = ()> + NodeIndexable + GraphProp,
{
    let mut graph: G = Default::default();

    let s = s.trim();
    let lines = s.lines().filter(|l| !l.is_empty());
    for (row, line) in lines.enumerate() {
        for (column, value) in line
            .split_ascii_whitespace()
            .filter(|s| !s.is_empty())
            .enumerate()
        {
            let has_edge = value.parse::<u8>().expect("invalid number");
            assert!(has_edge == 0 || has_edge == 1);

            if has_edge == 0 {
                continue;
            }

            while column >= graph.node_count() || row >= graph.node_count() {
                graph.add_node(());
            }

            let source = graph.from_index(row);
            let target = graph.from_index(column);

            graph.update_edge(source, target, ());
        }
    }

    graph
}

pub struct GraphFactory<G = Graph<(), (), Directed>> {
    _marker: PhantomData<fn() -> *const G>,
}

#[allow(clippy::unused_self)]
impl<G> GraphFactory<G>
where
    G: Default + Build<NodeWeight = (), EdgeWeight = ()> + NodeIndexable + GraphProp,
{
    const fn new() -> Self {
        Self {
            _marker: PhantomData,
        }
    }

    #[must_use]
    pub fn petersen_a(self) -> G {
        parse_graph(PETERSEN_A)
    }

    #[must_use]
    pub fn petersen_b(self) -> G {
        parse_graph(PETERSEN_B)
    }

    #[must_use]
    pub fn full_a(self) -> G {
        parse_graph(FULL_A)
    }

    #[must_use]
    pub fn full_b(self) -> G {
        parse_graph(FULL_B)
    }

    #[must_use]
    pub fn praust_a(self) -> G {
        parse_graph(PRAUST_A)
    }

    #[must_use]
    pub fn praust_b(self) -> G {
        parse_graph(PRAUST_B)
    }

    #[must_use]
    pub fn bigger(self) -> G {
        parse_graph(BIGGER)
    }

    #[must_use]
    pub fn bipartite(self) -> G {
        parse_graph(BIPARTITE)
    }
}

#[must_use]
pub const fn graph<Ty: EdgeType>() -> GraphFactory<Graph<(), (), Ty>> {
    GraphFactory::new()
}

#[must_use]
pub const fn undirected_graph() -> GraphFactory<Graph<(), (), Undirected>> {
    graph()
}

#[must_use]
pub const fn directed_graph() -> GraphFactory<Graph<(), (), Directed>> {
    graph()
}
