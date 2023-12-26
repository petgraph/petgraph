use numi::borrow::Moo;

use crate::{id::GraphId, storage::GraphStorage};

/// Index mapper for a graph.
///
/// The index mapper is a type, that maps a specific value (`From`), typically a [`LinearGraphId`],
/// into a different value ([`usize`]).
///
/// Mapping is bijective, meaning that `From -> usize` and `usize -> From` are both possible, and
/// that every `From` value has a unique `usize` value in the range specified by `0..Self::MAX`.
/// Meaning it is not possible to have a mapping of e.g. three nodes, like: `A`, `B`, `C` to `1`,
/// `3`, `5` with `MAX = 5`, because `2` and `4` are not mapped to anything.
///
/// How this conversion is done is up to the implementation, but should be consistent, i.e. the same
/// input value should always map to the same output value.
/// Index lookup should also be (if possible) `O(1)` for `Id -> usize`, but not necessarily for
/// `usize -> Id`.
pub trait IndexMapper<Id> {
    /// The maximum value that can be mapped to.
    ///
    /// This **must** be equal to the number of nodes in the graph.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::id::{IndexMapper, LinearGraphId};
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// # let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let mapper = NodeId::index_mapper(graph.storage());
    ///
    /// assert_eq!(mapper.max(), 2);
    /// ```
    fn max(&self) -> usize;

    /// Map a value from `From` to [`usize`].
    ///
    /// This **must** be pure and **must** return a valid value for any `Id` value in the graph it
    /// is bound to.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::id::{IndexMapper, LinearGraphId};
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// # let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let mut mapper = NodeId::index_mapper(graph.storage());
    ///
    /// // The mapping is highly dependent on the implementation, but should be consistent.
    /// // The order for which node maps to which value is not guaranteed.
    /// assert_ne!(mapper.get(&a), mapper.get(&b));
    /// ```
    fn get(&self, from: &Id) -> Option<usize>;

    /// Lookup a value from `To` to [`usize`].
    ///
    /// This **must** be pure and **must** return a valid value for any `Id` value in the graph
    /// it is bound to.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::id::{IndexMapper, LinearGraphId};
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// # let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let mut mapper = NodeId::index_mapper(graph.storage());
    ///
    /// // The mapping is highly dependent on the implementation, but should be consistent.
    /// // The order for which node maps to which value is not guaranteed.
    /// assert_ne!(mapper.index(&a), mapper.index(&b));
    /// ```
    ///
    /// # Panics
    ///
    /// Panics if the provided id is not valid, meaning it is not part of the graph (e.g. id from
    /// another instance)
    fn index(&self, from: &Id) -> usize {
        self.get(from).expect("invalid id provided")
    }

    /// Reverse lookup a value from `To` to `From`.
    ///
    /// This **must** be pure, but **may** return `None` if a reverse mapping does not exist (for
    /// example `usize` is too large and therefore does not exist).
    ///
    /// # Example
    ///
    /// ```
    /// use numi::borrow::Moo;
    /// use petgraph_core::id::{IndexMapper, LinearGraphId};
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// # let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let mut mapper = NodeId::index_mapper(graph.storage());
    ///
    /// let mapped = mapper.index(&a);
    /// assert_eq!(mapper.reverse(mapped), Some(Moo::Borrowed(&a)));
    /// ```
    fn reverse(&self, to: usize) -> Option<Moo<Id>>;
}

/// Linear graph identifier.
///
/// A linear graph identifier is a graph identifier that has a linear mapping to a `usize` value,
/// that mapping must be continuous .
pub trait LinearGraphId<S>: GraphId + Sized
where
    S: GraphStorage,
{
    /// The index mapper for this graph identifier.
    type Mapper<'graph>: IndexMapper<Self>
    where
        S: 'graph;

    /// Get the index mapper for this graph identifier.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::id::{IndexMapper, LinearGraphId};
    /// use petgraph_dino::{DiDinoGraph, NodeId};
    ///
    /// let mut graph = DiDinoGraph::new();
    ///
    /// let a = *graph.insert_node("A").id();
    /// let b = *graph.insert_node("B").id();
    /// # let ab = graph.insert_edge("A → B", &a, &b);
    ///
    /// let mapper = NodeId::index_mapper(graph.storage());
    /// ```
    fn index_mapper(storage: &S) -> Self::Mapper<'_>;
}
