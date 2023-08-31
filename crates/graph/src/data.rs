use petgraph_core::{
    data::{from_elements_indexable, Build, Create, DataMap, DataMapMut, Element, FromElements},
    edge::EdgeType,
    id::IndexType,
    visit::Data,
};

use crate::Graph;

impl<N, E, Ty, Ix> Create for Graph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn with_capacity(nodes: usize, edges: usize) -> Self {
        Self::with_capacity(nodes, edges)
    }
}

impl<N, E, Ty, Ix> Build for Graph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn add_node(&mut self, weight: Self::NodeWeight) -> Self::NodeId {
        self.add_node(weight)
    }

    fn add_edge(
        &mut self,
        a: Self::NodeId,
        b: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Option<Self::EdgeId> {
        Some(self.add_edge(a, b, weight))
    }

    fn update_edge(
        &mut self,
        a: Self::NodeId,
        b: Self::NodeId,
        weight: Self::EdgeWeight,
    ) -> Self::EdgeId {
        self.update_edge(a, b, weight)
    }
}

impl<N, E, Ty, Ix> FromElements for Graph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn from_elements<I>(iterable: I) -> Self
    where
        Self: Sized,
        I: IntoIterator<Item = Element<Self::NodeWeight, Self::EdgeWeight>>,
    {
        from_elements_indexable(iterable)
    }
}

impl<N, E, Ty, Ix> Data for Graph<N, E, Ty, Ix>
where
    Ix: IndexType,
{
    type EdgeWeight = E;
    type NodeWeight = N;
}

impl<N, E, Ty, Ix> DataMap for Graph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn node_weight(&self, id: Self::NodeId) -> Option<&Self::NodeWeight> {
        self.node_weight(id)
    }

    fn edge_weight(&self, id: Self::EdgeId) -> Option<&Self::EdgeWeight> {
        self.edge_weight(id)
    }
}

impl<N, E, Ty, Ix> DataMapMut for Graph<N, E, Ty, Ix>
where
    Ty: EdgeType,
    Ix: IndexType,
{
    fn node_weight_mut(&mut self, id: Self::NodeId) -> Option<&mut Self::NodeWeight> {
        self.node_weight_mut(id)
    }

    fn edge_weight_mut(&mut self, id: Self::EdgeId) -> Option<&mut Self::EdgeWeight> {
        self.edge_weight_mut(id)
    }
}
