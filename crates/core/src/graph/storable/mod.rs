use core::hash::Hash;

use crate::graph::Graph;

mod containers;

pub trait DataContainer<Id, Data> {
    fn get(&mut self, node_id: Id) -> &Data;
    fn get_mut(&mut self, node_id: Id) -> &mut Data;
    fn insert(&mut self, node_id: Id, data: Data);
    fn remove(&mut self, node_id: Id);
    fn clear(&mut self);
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
    fn clear(&mut self);
}

pub trait QueueContainer<Data> {
    fn enqueue(&mut self, data: Data);
    fn dequeue(&mut self) -> Option<Data>;
    fn clear(&mut self);
}

pub trait Storable: Graph {
    fn node_visit_container(&self) -> impl VisitContainer<Self::NodeId>;
    fn node_data_container<Data>(&self) -> impl DataContainer<Self::NodeId, Data>;

    fn edge_visit_container(&self) -> impl VisitContainer<Self::EdgeId>;
    fn edge_data_container<Data>(&self) -> impl DataContainer<Self::EdgeId, Data>;
}
