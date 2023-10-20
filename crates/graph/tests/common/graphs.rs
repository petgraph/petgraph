use petgraph_core::{
    edge::{Directed, EdgeType},
    id::{DefaultIx, IndexType},
};
use petgraph_graph::{stable::StableGraph, EdgeIndex, Graph, NodeIndex};

pub trait FromDefault {
    fn from_default() -> Self;
}

/// Graph: a ⤸
pub struct GraphSelfLoop<G, Ix = DefaultIx> {
    pub graph: G,
    pub a: NodeIndex<Ix>,
    pub aa: EdgeIndex<Ix>,
}

impl<N, E, Ty, Ix> FromDefault for GraphSelfLoop<Graph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = Graph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let aa = graph.add_edge(a, a, E::default());

        Self { graph, a, aa }
    }
}

impl<N, E, Ty, Ix> FromDefault for GraphSelfLoop<StableGraph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = StableGraph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let aa = graph.add_edge(a, a, E::default());

        Self { graph, a, aa }
    }
}

/// Graph: a → b
pub struct GraphLink<G, Ix = DefaultIx> {
    pub graph: G,

    pub a: NodeIndex<Ix>,
    pub b: NodeIndex<Ix>,
    pub ab: EdgeIndex<Ix>,
}

impl<N, E, Ty, Ix> FromDefault for GraphLink<Graph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = Graph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());
        let ab = graph.add_edge(a, b, E::default());

        Self { graph, a, b, ab }
    }
}

impl<N, E, Ty, Ix> FromDefault for GraphLink<StableGraph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = StableGraph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());
        let ab = graph.add_edge(a, b, E::default());

        Self { graph, a, b, ab }
    }
}

// Graph: a ⇆ b → c
pub struct GraphDoubleLink<G, Ix = DefaultIx> {
    pub graph: G,

    pub a: NodeIndex<Ix>,
    pub b: NodeIndex<Ix>,
    pub c: NodeIndex<Ix>,
    pub ab: EdgeIndex<Ix>,
    pub ba: EdgeIndex<Ix>,
    pub bc: EdgeIndex<Ix>,
}

impl<N, E, Ty, Ix> FromDefault for GraphDoubleLink<Graph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = Graph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());
        let c = graph.add_node(N::default());

        let ab = graph.add_edge(a, b, E::default());
        let ba = graph.add_edge(b, a, E::default());
        let bc = graph.add_edge(b, c, E::default());

        Self {
            graph,
            a,
            b,
            c,
            ab,
            ba,
            bc,
        }
    }
}

impl<N, E, Ty, Ix> FromDefault for GraphDoubleLink<StableGraph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = StableGraph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());
        let c = graph.add_node(N::default());

        let ab = graph.add_edge(a, b, E::default());
        let ba = graph.add_edge(b, a, E::default());
        let bc = graph.add_edge(b, c, E::default());

        Self {
            graph,
            a,
            b,
            c,
            ab,
            ba,
            bc,
        }
    }
}

// Graph: a ⇉ b
pub struct GraphDoubleSameDirection<G, Ix = DefaultIx> {
    pub graph: G,

    pub a: NodeIndex<Ix>,
    pub b: NodeIndex<Ix>,

    pub ab1: EdgeIndex<Ix>,
    pub ab2: EdgeIndex<Ix>,
}

impl<N, E, Ty, Ix> FromDefault for GraphDoubleSameDirection<Graph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = Graph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());

        let ab1 = graph.add_edge(a, b, E::default());
        let ab2 = graph.add_edge(a, b, E::default());

        Self {
            graph,
            a,
            b,
            ab1,
            ab2,
        }
    }
}

impl<N, E, Ty, Ix> FromDefault for GraphDoubleSameDirection<StableGraph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = StableGraph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());

        let ab1 = graph.add_edge(a, b, E::default());
        let ab2 = graph.add_edge(a, b, E::default());

        Self {
            graph,
            a,
            b,
            ab1,
            ab2,
        }
    }
}

/// Graph: a → b   c
pub struct GraphLoner<G, Ix = DefaultIx> {
    pub graph: G,

    pub a: NodeIndex<Ix>,
    pub b: NodeIndex<Ix>,
    pub c: NodeIndex<Ix>,

    pub ab: EdgeIndex<Ix>,
}

impl<N, E, Ty, Ix> FromDefault for GraphLoner<Graph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = Graph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());
        let c = graph.add_node(N::default());

        let ab = graph.add_edge(a, b, E::default());

        Self { graph, a, b, c, ab }
    }
}

impl<N, E, Ty, Ix> FromDefault for GraphLoner<StableGraph<N, E, Ty, Ix>, Ix>
where
    N: Default,
    E: Default,
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_default() -> Self {
        let mut graph = StableGraph::with_capacity(0, 0);

        let a = graph.add_node(N::default());
        let b = graph.add_node(N::default());
        let c = graph.add_node(N::default());

        let ab = graph.add_edge(a, b, E::default());

        Self { graph, a, b, c, ab }
    }
}
