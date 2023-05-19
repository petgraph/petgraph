mod matching;
mod semantic;
mod state;

use petgraph_core::{
    data::DataMap,
    visit::{
        EdgeCount, GetAdjacencyMatrix, GraphProp, IntoEdgesDirected, IntoNeighborsDirected,
        NodeCompactIndexable,
    },
};

use crate::isomorphism::{semantic::NoSemanticMatch, state::Vf2State};

/// \[Generic\] Return `true` if the graphs `g0` and `g1` are isomorphic.
///
/// Using the VF2 algorithm, only matching graph syntactically (graph
/// structure).
///
/// The graphs should not be multigraphs.
///
/// **Reference**
///
/// * Luigi P. Cordella, Pasquale Foggia, Carlo Sansone, Mario Vento; *A (Sub)Graph Isomorphism
///   Algorithm for Matching Large Graphs*
pub fn is_isomorphic<G0, G1>(g0: G0, g1: G1) -> bool
where
    G0: NodeCompactIndexable + EdgeCount + GetAdjacencyMatrix + GraphProp + IntoNeighborsDirected,
    G1: NodeCompactIndexable
        + EdgeCount
        + GetAdjacencyMatrix
        + GraphProp<EdgeType = G0::EdgeType>
        + IntoNeighborsDirected,
{
    if g0.node_count() != g1.node_count() || g0.edge_count() != g1.edge_count() {
        return false;
    }

    let mut st = (Vf2State::new(&g0), Vf2State::new(&g1));
    matching::try_match(&mut st, &mut NoSemanticMatch, &mut NoSemanticMatch, false).unwrap_or(false)
}

/// \[Generic\] Return `true` if the graphs `g0` and `g1` are isomorphic.
///
/// Using the VF2 algorithm, examining both syntactic and semantic
/// graph isomorphism (graph structure and matching node and edge weights).
///
/// The graphs should not be multigraphs.
pub fn is_isomorphic_matching<G0, G1, NM, EM>(
    g0: G0,
    g1: G1,
    mut node_match: NM,
    mut edge_match: EM,
) -> bool
where
    G0: NodeCompactIndexable
        + EdgeCount
        + DataMap
        + GetAdjacencyMatrix
        + GraphProp
        + IntoEdgesDirected,
    G1: NodeCompactIndexable
        + EdgeCount
        + DataMap
        + GetAdjacencyMatrix
        + GraphProp<EdgeType = G0::EdgeType>
        + IntoEdgesDirected,
    NM: FnMut(&G0::NodeWeight, &G1::NodeWeight) -> bool,
    EM: FnMut(&G0::EdgeWeight, &G1::EdgeWeight) -> bool,
{
    if g0.node_count() != g1.node_count() || g0.edge_count() != g1.edge_count() {
        return false;
    }

    let mut st = (Vf2State::new(&g0), Vf2State::new(&g1));
    matching::try_match(&mut st, &mut node_match, &mut edge_match, false).unwrap_or(false)
}

/// \[Generic\] Return `true` if `g0` is isomorphic to a subgraph of `g1`.
///
/// Using the VF2 algorithm, only matching graph syntactically (graph
/// structure).
///
/// The graphs should not be multigraphs.
///
/// # Subgraph isomorphism
///
/// (adapted from [`networkx` documentation](https://networkx.github.io/documentation/stable/reference/algorithms/isomorphism.vf2.html))
///
/// Graph theory literature can be ambiguous about the meaning of the above statement,
/// and we seek to clarify it now.
///
/// In the VF2 literature, a mapping **M** is said to be a *graph-subgraph isomorphism*
/// iff **M** is an isomorphism between **G2** and a subgraph of **G1**. Thus, to say
/// that **G1** and **G2** are graph-subgraph isomorphic is to say that a subgraph of
/// **G1** is isomorphic to **G2**.
///
/// Other literature uses the phrase ‘subgraph isomorphic’ as in
/// ‘**G1** does not have a subgraph isomorphic to **G2**’. Another use is as an in adverb
/// for isomorphic. Thus, to say that **G1** and **G2** are subgraph isomorphic is to say
/// that a subgraph of **G1** is isomorphic to **G2**.
///
/// Finally, the term ‘subgraph’ can have multiple meanings. In this context,
/// ‘subgraph’ always means a ‘node-induced subgraph’. Edge-induced subgraph
/// isomorphisms are not directly supported. For subgraphs which are not
/// induced, the term ‘monomorphism’ is preferred over ‘isomorphism’.
///
/// **Reference**
///
/// * Luigi P. Cordella, Pasquale Foggia, Carlo Sansone, Mario Vento; *A (Sub)Graph Isomorphism
///   Algorithm for Matching Large Graphs*
pub fn is_isomorphic_subgraph<G0, G1>(g0: G0, g1: G1) -> bool
where
    G0: NodeCompactIndexable + EdgeCount + GetAdjacencyMatrix + GraphProp + IntoNeighborsDirected,
    G1: NodeCompactIndexable
        + EdgeCount
        + GetAdjacencyMatrix
        + GraphProp<EdgeType = G0::EdgeType>
        + IntoNeighborsDirected,
{
    if g0.node_count() > g1.node_count() || g0.edge_count() > g1.edge_count() {
        return false;
    }

    let mut st = (Vf2State::new(&g0), Vf2State::new(&g1));
    matching::try_match(&mut st, &mut NoSemanticMatch, &mut NoSemanticMatch, true).unwrap_or(false)
}

/// \[Generic\] Return `true` if `g0` is isomorphic to a subgraph of `g1`.
///
/// Using the VF2 algorithm, examining both syntactic and semantic
/// graph isomorphism (graph structure and matching node and edge weights).
///
/// The graphs should not be multigraphs.
pub fn is_isomorphic_subgraph_matching<G0, G1, NM, EM>(
    g0: G0,
    g1: G1,
    mut node_match: NM,
    mut edge_match: EM,
) -> bool
where
    G0: NodeCompactIndexable
        + EdgeCount
        + DataMap
        + GetAdjacencyMatrix
        + GraphProp
        + IntoEdgesDirected,
    G1: NodeCompactIndexable
        + EdgeCount
        + DataMap
        + GetAdjacencyMatrix
        + GraphProp<EdgeType = G0::EdgeType>
        + IntoEdgesDirected,
    NM: FnMut(&G0::NodeWeight, &G1::NodeWeight) -> bool,
    EM: FnMut(&G0::EdgeWeight, &G1::EdgeWeight) -> bool,
{
    if g0.node_count() > g1.node_count() || g0.edge_count() > g1.edge_count() {
        return false;
    }

    let mut st = (Vf2State::new(&g0), Vf2State::new(&g1));
    matching::try_match(&mut st, &mut node_match, &mut edge_match, true).unwrap_or(false)
}

/// Using the VF2 algorithm, examine both syntactic and semantic graph
/// isomorphism (graph structure and matching node and edge weights) and,
/// if `g0` is isomorphic to a subgraph of `g1`, return the mappings between
/// them.
///
/// The graphs should not be multigraphs.
pub fn subgraph_isomorphisms_iter<'a, G0, G1, NM, EM>(
    g0: &'a G0,
    g1: &'a G1,
    node_match: &'a mut NM,
    edge_match: &'a mut EM,
) -> Option<impl Iterator<Item = Vec<usize>> + 'a>
where
    G0: 'a
        + NodeCompactIndexable
        + EdgeCount
        + DataMap
        + GetAdjacencyMatrix
        + GraphProp
        + IntoEdgesDirected,
    G1: 'a
        + NodeCompactIndexable
        + EdgeCount
        + DataMap
        + GetAdjacencyMatrix
        + GraphProp<EdgeType = G0::EdgeType>
        + IntoEdgesDirected,
    NM: 'a + FnMut(&G0::NodeWeight, &G1::NodeWeight) -> bool,
    EM: 'a + FnMut(&G0::EdgeWeight, &G1::EdgeWeight) -> bool,
{
    if g0.node_count() > g1.node_count() || g0.edge_count() > g1.edge_count() {
        return None;
    }

    Some(matching::GraphMatcher::new(
        g0, g1, node_match, edge_match, true,
    ))
}
