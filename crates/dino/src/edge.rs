use petgraph_core::{
    attributes::Never,
    id::{GraphId, ManagedGraphId},
};

use crate::node::NodeId;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EdgeId(usize);

impl EdgeId {
    pub(crate) fn new(id: usize) -> Self {
        Self(id)
    }

    pub(crate) fn increment(self) -> Self {
        Self(self.0 + 1)
    }
}

impl GraphId for EdgeId {
    type AttributeIndex = Never;
}

impl ManagedGraphId for EdgeId {}

pub(crate) struct Edge<T> {
    pub(crate) id: EdgeId,
    pub(crate) weight: T,

    pub(crate) source: NodeId,
    pub(crate) target: NodeId,
}

impl<T> Edge<T> {
    pub(crate) fn new(id: EdgeId, weight: T, source: NodeId, target: NodeId) -> Self {
        Self {
            id,
            weight,
            source,
            target,
        }
    }
}
