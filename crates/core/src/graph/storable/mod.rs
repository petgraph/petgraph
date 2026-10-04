use crate::graph::Graph;

#[cfg(feature = "default-impls")]
mod containers;

/// A container for storing data associated with nodes or edges in a graph.
pub trait DataContainer<Id, Data> {
    /// Returns a reference to the data associated with the given id.
    fn get(&self, node_id: Id) -> &Data;

    /// Returns a mutable reference to the data associated with the given id.
    fn get_mut(&mut self, node_id: Id) -> &mut Data;

    /// Inserts data associated with the given id. If data already exists for the id, it will be
    /// replaced.
    fn insert(&mut self, node_id: Id, data: Data);

    /// Removes the data associated with the given id.
    fn remove(&mut self, node_id: Id);

    /// Clears all data in the container, resetting it to its default state.
    fn clear(&mut self);

    /// Consumes the container and returns an iterator over the (id, data) pairs. All data
    /// with non-default values will be included in the iterator.
    ///
    /// The iterator is in no particular order.
    ///
    /// # Caution
    /// The iterator may not iterate over all existing ids. That is, it only guarantees to iterate
    /// over ids that have non-default data values.
    fn into_iter(self) -> impl Iterator<Item = (Id, Data)>;
}

/// A container for tracking visited nodes or edges in a graph.
pub trait VisitContainer<Id> {
    /// Marks the given id as visited.
    fn mark_visited(&mut self, node_id: Id);

    /// Marks the given id as unvisited.
    fn mark_unvisited(&mut self, node_id: Id);

    /// Returns whether the given id is marked as visited.
    fn is_visited(&self, node_id: Id) -> bool;

    /// Clears all visited marks in the container, resetting it to the unvisited state.
    fn clear(&mut self);
}

/// A container for storing data in a stack-like structure.
pub trait StackContainer<Data> {
    /// Pushes data onto the top of the stack.
    fn push(&mut self, data: Data);

    /// Pops data from the top of the stack. Returns `None` if the stack is empty.
    fn pop(&mut self) -> Option<Data>;

    /// Peeks at the data on the top of the stack without removing it. Returns `None` if the stack
    /// is empty.
    fn peek_last(&self) -> Option<&Data>;

    /// Clears all data in the stack and leaves it empty.
    fn clear(&mut self);
}

/// A container for storing data in a queue-like structure.
pub trait QueueContainer<Data> {
    /// Pushes data onto the back of the queue.
    fn push_back(&mut self, data: Data);

    /// Pops data from the front of the queue. Returns `None` if the queue is empty.
    fn pop_front(&mut self) -> Option<Data>;

    /// Peeks at the data at the front of the queue without removing it. Returns `None` if the queue
    /// is empty.
    fn peek_first(&self) -> Option<&Data>;

    /// Clears all data in the queue and leaves it empty.
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
