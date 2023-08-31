use crate::node::NodeId;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct EdgeId(usize);

pub(crate) struct Edge<T> {
    id: EdgeId,
    weight: T,

    pub(crate) source: NodeId,
    pub(crate) target: NodeId,
}
