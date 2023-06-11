use std::fmt::Debug;

use petgraph_core::visit::{EdgeRef, GraphBase, IntoEdgeReferences, IntoNodeReferences, NodeRef};
use proptest::{prelude::*, test_runner::TestCaseResult};

pub fn assert_nodes_eq<G1, G2>(graph1: G1, graph2: G2) -> TestCaseResult
where
    G1: IntoNodeReferences,
    <G1 as GraphBase>::NodeId: Ord,
    G1::NodeRef: PartialEq<G2::NodeRef> + Debug,
    G2: IntoNodeReferences,
    <G2 as GraphBase>::NodeId: Ord,
    G2::NodeRef: PartialEq + Debug,
{
    let mut nodes1 = graph1.node_references().collect::<Vec<_>>();
    let mut nodes2 = graph2.node_references().collect::<Vec<_>>();

    nodes1.sort_by_key(|node| node.id());
    nodes2.sort_by_key(|node| node.id());

    prop_assert_eq!(nodes1, nodes2);
    Ok(())
}

pub fn assert_edges_eq<G1, G2>(graph1: G1, graph2: G2) -> TestCaseResult
where
    G1: IntoEdgeReferences,
    <G1 as GraphBase>::NodeId: Ord,
    G1::EdgeRef: PartialEq<G2::EdgeRef> + Debug,
    G2: IntoEdgeReferences,
    <G2 as GraphBase>::NodeId: Ord,
    G2::EdgeRef: PartialEq + Debug,
{
    let mut edges1 = graph1.edge_references().collect::<Vec<_>>();
    let mut edges2 = graph2.edge_references().collect::<Vec<_>>();

    edges1.sort_by_key(|edge| (edge.source(), edge.target()));
    edges2.sort_by_key(|edge| (edge.source(), edge.target()));

    prop_assert_eq!(edges1, edges2);

    Ok(())
}

pub fn assert_edges_without_weight_eq<G1, G2>(graph1: G1, graph2: G2) -> TestCaseResult
where
    G1: IntoEdgeReferences,
    <G1 as GraphBase>::NodeId: Ord,
    <G1::EdgeRef as EdgeRef>::NodeId: PartialEq<<G2::EdgeRef as EdgeRef>::NodeId> + Debug,
    G2: IntoEdgeReferences,
    <G2 as GraphBase>::NodeId: Ord,
    <G2::EdgeRef as EdgeRef>::NodeId: PartialEq + Debug,
    (<G1 as GraphBase>::NodeId, <G1 as GraphBase>::NodeId):
        PartialEq<(<G2 as GraphBase>::NodeId, <G2 as GraphBase>::NodeId)>,
{
    let mut edges1 = graph1
        .edge_references()
        .map(|edge| (edge.source(), edge.target()))
        .collect::<Vec<_>>();
    let mut edges2 = graph2
        .edge_references()
        .map(|edge| (edge.source(), edge.target()))
        .collect::<Vec<_>>();

    edges1.sort();
    edges2.sort();

    prop_assert_eq!(edges1, edges2);

    Ok(())
}
