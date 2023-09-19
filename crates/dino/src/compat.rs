#![allow(deprecated)]
use bitvec::boxed::BitBox;
use petgraph_core::deprecated::visit::VisitMap;

use crate::NodeId;

// TODO: Visitable!

struct NodeVisitMap(BitBox);

impl VisitMap<NodeId> for NodeVisitMap {
    fn visit(&mut self, a: NodeId) -> bool {
        todo!()
    }

    fn is_visited(&self, a: &NodeId) -> bool {
        todo!()
    }
}
