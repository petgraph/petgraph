#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct NodeId(usize);

pub struct Node<T> {
    id: NodeId,
    weight: T,
}
