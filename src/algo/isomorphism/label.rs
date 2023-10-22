use crate::data::DataMap;
use crate::visit::GraphBase;

pub const DEFAULT_NODE_LABEL: usize = 0usize;

pub trait NodeLabel<G: GraphBase> {
    fn get_node_label(&mut self, g: G, node_id: G::NodeId) -> usize;
}

pub struct NoNodeLabel;

impl<G: GraphBase> NodeLabel<G> for NoNodeLabel {
    fn get_node_label(&mut self, _g: G, _id: <G as GraphBase>::NodeId) -> usize {
        DEFAULT_NODE_LABEL
    }
}

impl<G, F> NodeLabel<G> for F
where
    G: DataMap,
    F: FnMut(&G::NodeWeight) -> usize,
{
    fn get_node_label(&mut self, g: G, node_id: <G as GraphBase>::NodeId) -> usize {
        let node_weight = g.node_weight(node_id).unwrap();
        self(node_weight)
    }
}
