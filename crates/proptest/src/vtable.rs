use petgraph_core::{
    data::{Build, Create},
    visit::Data,
};

pub struct VTable<G, NodeWeight, NodeIndex, EdgeWeight> {
    pub with_capacity: fn(usize, usize) -> G,
    pub add_node: fn(&mut G, NodeWeight) -> NodeIndex,
    pub add_edge: fn(&mut G, NodeIndex, NodeIndex, EdgeWeight),
}

impl<G, NodeWeight, NodeIndex, EdgeWeight> Copy for VTable<G, NodeWeight, NodeIndex, EdgeWeight> {}

impl<G, NodeWeight, NodeIndex, EdgeWeight> Clone for VTable<G, NodeWeight, NodeIndex, EdgeWeight> {
    fn clone(&self) -> Self {
        *self
    }
}

fn add_edge_no_return<G>(graph: &mut G, source: G::NodeId, target: G::NodeId, weight: G::EdgeWeight)
where
    G: Build,
{
    graph.add_edge(source, target, weight);
}

pub(crate) fn create<G: Create + Build + Data>()
-> VTable<G, G::NodeWeight, G::NodeId, G::EdgeWeight> {
    VTable {
        with_capacity: G::with_capacity,
        add_node: G::add_node,
        add_edge: add_edge_no_return::<G>,
    }
}
