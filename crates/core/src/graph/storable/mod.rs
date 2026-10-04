use crate::graph::Graph;

#[cfg(feature = "alloc")]
mod containers;

/// A container for storing data associated with nodes or edges in a graph.
pub trait DataContainer<Id, Data> {
    fn get(&self, node_id: Id) -> &Data;
    fn get_mut(&mut self, node_id: Id) -> &mut Data;
    fn insert(&mut self, node_id: Id, data: Data);
    fn remove(&mut self, node_id: Id);
    fn clear(&mut self);
    fn into_iter(self) -> impl Iterator<Item = (Id, Data)>;
}

/// A container for tracking visited nodes or edges in a graph.
pub trait VisitContainer<Id> {
    fn mark_visited(&mut self, node_id: Id);
    fn mark_unvisited(&mut self, node_id: Id);
    fn is_visited(&self, node_id: Id) -> bool;
    fn clear(&mut self);
}

/// A container for storing data in a stack-like structure.
pub trait StackContainer<Data> {
    fn push(&mut self, data: Data);
    fn pop(&mut self) -> Option<Data>;
    fn peek_last(&self) -> Option<&Data>;
    fn clear(&mut self);
}

/// A container for storing data in a queue-like structure.
pub trait QueueContainer<Data> {
    fn push_back(&mut self, data: Data);
    fn pop_front(&mut self) -> Option<Data>;
    fn peek_first(&self) -> Option<&Data>;
    fn clear(&mut self);
}

/// A trait for graphs that can provide containers for storing node and edge data, as well as
/// visited nodes and edges, stacks, and queues.
///
/// This trait enables each graph implementation to define its own storage mechanisms and thus
/// optimize for specific use cases or performance characteristics.
///
/// # Caution
/// The trait expects that all containers are created with sufficient capacity to hold data for all
/// nodes and/or edges respectively, or that they can grow dynamically.
pub trait Storable: Graph {
    type NodeVisitContainer: VisitContainer<Self::NodeId>;
    type NodeDataContainer<Data: Default>: DataContainer<Self::NodeId, Data>;
    type EdgeVisitContainer: VisitContainer<Self::EdgeId>;
    type EdgeDataContainer<Data: Default>: DataContainer<Self::EdgeId, Data>;
    type StackContainer<Data>: StackContainer<Data>;
    type QueueContainer<Data>: QueueContainer<Data>;

    /// Returns a new container for tracking visited nodes.
    ///
    /// Holds enough capacity to track all nodes in the graph or can grow dynamically.
    fn node_visit_container(&self) -> Self::NodeVisitContainer;
    /// Returns a new container for storing data associated with nodes.
    ///
    /// Holds enough capacity to store data for all nodes in the graph or can grow dynamically.
    fn node_data_container<Data: Default>(&self) -> Self::NodeDataContainer<Data>;

    /// Returns a new container for tracking visited edges.
    ///
    /// Holds enough capacity to track all edges in the graph or can grow dynamically.
    fn edge_visit_container(&self) -> Self::EdgeVisitContainer;
    /// Returns a new container for storing data associated with edges.
    ///
    /// Holds enough capacity to store data for all edges in the graph or can grow dynamically.
    fn edge_data_container<Data: Default>(&self) -> Self::EdgeDataContainer<Data>;

    /// Returns a new container for storing data on a stack.
    fn stack_container<Data>(&self) -> Self::StackContainer<Data>;
    /// Returns a new container for storing data in a queue.
    fn queue_container<Data>(&self) -> Self::QueueContainer<Data>;
}
