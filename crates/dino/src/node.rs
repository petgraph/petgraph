#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

pub struct Node<T> {
    pub(crate) id: NodeId,
    pub(crate) weight: T,
}

impl<T> Node<T> {
    pub fn new(id: NodeId, weight: T) -> Self {
        Self { id, weight }
    }
}
