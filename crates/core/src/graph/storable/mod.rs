use crate::graph::Graph;

mod containers;

pub trait DataContainer<Id, Data> {
    fn get(&self, node_id: Id) -> &Data;
    fn get_mut(&mut self, node_id: Id) -> &mut Data;
    fn insert(&mut self, node_id: Id, data: Data);
    fn remove(&mut self, node_id: Id);
    fn clear(&mut self);
    fn into_iter(self) -> impl Iterator<Item = (Id, Data)>;
}

pub trait VisitContainer<Id> {
    fn mark_visited(&mut self, node_id: Id);
    fn mark_unvisited(&mut self, node_id: Id);
    fn is_visited(&self, node_id: Id) -> bool;
    fn clear(&mut self);
}

pub trait StackContainer<Data> {
    fn push(&mut self, data: Data);
    fn pop(&mut self) -> Option<Data>;
    fn peek_last(&self) -> Option<&Data>;
    fn clear(&mut self);
}

pub trait QueueContainer<Data> {
    fn push_back(&mut self, data: Data);
    fn pop_front(&mut self) -> Option<Data>;
    fn peek_first(&self) -> Option<&Data>;
    fn clear(&mut self);
}

pub trait Storable: Graph {
    type NodeVisitContainer: VisitContainer<Self::NodeId>;
    type NodeDataContainer<Data>: DataContainer<Self::NodeId, Data>;
    type EdgeVisitContainer: VisitContainer<Self::EdgeId>;
    type EdgeDataContainer<Data>: DataContainer<Self::EdgeId, Data>;
    type StackContainer<Data>: StackContainer<Data>;
    type QueueContainer<Data>: QueueContainer<Data>;

    fn node_visit_container(&self) -> Self::NodeVisitContainer;
    fn node_data_container<Data>(&self) -> Self::NodeDataContainer<Data>;

    fn edge_visit_container(&self) -> Self::EdgeVisitContainer;
    fn edge_data_container<Data>(&self) -> Self::EdgeDataContainer<Data>;

    fn stack_container<Data>(&self) -> Self::StackContainer<Data>;
    fn queue_container<Data>(&self) -> Self::QueueContainer<Data>;
}
