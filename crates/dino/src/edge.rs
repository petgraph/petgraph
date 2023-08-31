use crate::node::NodeId;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EdgeId(usize);

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
