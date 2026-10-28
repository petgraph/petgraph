use core::ops::Index;

use crate::graph::Graph;

#[cfg(feature = "alloc")]
pub mod containers;

/// A container for storing data associated with nodes or edges in a graph.
pub trait DataContainer<Id, Data: Clone> {
    /// Returns a reference to the data associated with the given id.
    ///
    /// May return a reference to a default value if no data is associated with the id.
    fn get(&self, id: Id) -> &Data;

    /// Returns a mutable reference to the data associated with the given id.
    ///
    /// May insert a default value and return a mutable reference to it if no data is associated
    /// with the id.
    fn get_mut(&mut self, id: Id) -> &mut Data;

    /// Inserts data associated with the given id. If data already exists for the id, it will be
    /// replaced.
    fn insert(&mut self, id: Id, data: Data);

    /// Removes the data associated with the given id.
    fn remove(&mut self, id: Id);

    /// Clears all data in the container, resetting it to its default state.
    fn clear(&mut self);

    // ToDo: Possibly re-add such a method, if it can be designed/implemented in a consistent way.
    // /// Consumes the container and returns an iterator over the (id, data) pairs. All data
    // /// with non-default values will be included in the iterator.
    // ///
    // /// The iterator is in no particular order.
    // ///
    // /// # Caution
    // /// The iterator may not iterate over all existing ids. That is, it only guarantees to
    // /// iterate over ids that have non-default data values.
    // fn into_iter(self) -> impl Iterator<Item = (Id, Data)>;
}

/// A container for tracking visited nodes or edges in a graph.
pub trait VisitContainer<Id> {
    /// Marks the given id as visited.
    fn mark_visited(&mut self, id: Id);

    /// Marks the given id as unvisited.
    fn mark_unvisited(&mut self, id: Id);

    /// Returns whether the given id is marked as visited.
    fn is_visited(&self, id: Id) -> bool;

    /// Clears all visited marks in the container, resetting it to the unvisited state.
    fn clear(&mut self);
}

/// A container for storing data in a stack-like structure.
pub trait StackContainer<Data>: Index<usize, Output = Data> {
    /// Pushes data onto the top of the stack.
    fn push(&mut self, data: Data);

    /// Pops data from the top of the stack. Returns `None` if the stack is empty.
    fn pop(&mut self) -> Option<Data>;

    /// Removes and returns the data at the given index in the stack. Returns `None` if the index is
    /// out of bounds.
    ///
    /// The removed element is replaced by the element on top of the stack.
    fn swap_remove(&mut self, index: usize) -> Option<Data>;

    /// Peeks at the data on the top of the stack without removing it. Returns `None` if the stack
    /// is empty.
    fn peek_last(&self) -> Option<&Data>;

    /// Returns a reference to the data at the given index in the stack. Returns `None` if the index
    /// is out of bounds.
    fn get(&self, index: usize) -> Option<&Data>;

    /// Returns the number of elements in the stack.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Clears all data in the stack and leaves it empty.
    fn clear(&mut self);
}

/// A container for storing data in a queue-like structure.
pub trait QueueContainer<Data>: Index<usize, Output = Data> {
    /// Pushes data onto the back of the queue.
    fn push_back(&mut self, data: Data);

    /// Pops data from the front of the queue. Returns `None` if the queue is empty.
    fn pop_front(&mut self) -> Option<Data>;

    /// Peeks at the data at the front of the queue without removing it. Returns `None` if the queue
    /// is empty.
    fn peek_first(&self) -> Option<&Data>;

    /// Returns a reference to the data at the given index in the queue. Returns `None` if the index
    /// is out of bounds.
    fn get(&self, index: usize) -> Option<&Data>;

    /// Returns the number of elements in the queue.
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

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
pub trait StorableGraph: Graph {
    type NodeVisitContainer: VisitContainer<Self::NodeId>;
    type NodeDataContainer<Data: Clone>: DataContainer<Self::NodeId, Data>;
    type EdgeVisitContainer: VisitContainer<Self::EdgeId>;
    type EdgeDataContainer<Data: Clone>: DataContainer<Self::EdgeId, Data>;

    /// Returns a new container for tracking visited nodes.
    ///
    /// Holds enough capacity to track all nodes in the graph or can grow dynamically.
    fn node_visit_container(&self) -> Self::NodeVisitContainer;
    /// Returns a new container for storing data associated with nodes.
    ///
    /// Holds enough capacity to store data for all nodes in the graph or can grow dynamically.
    fn node_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::NodeDataContainer<Data>;

    /// Returns a new container for tracking visited edges.
    ///
    /// Holds enough capacity to track all edges in the graph or can grow dynamically.
    fn edge_visit_container(&self) -> Self::EdgeVisitContainer;
    /// Returns a new container for storing data associated with edges.
    ///
    /// Holds enough capacity to store data for all edges in the graph or can grow dynamically.
    fn edge_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::EdgeDataContainer<Data>;
}
