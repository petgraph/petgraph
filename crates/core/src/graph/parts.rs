use error_stack::Result;

use super::Graph;
use crate::{DetachedEdge, DetachedNode};

// TODO: this shouldn't be a trait. I think?
pub trait FromGraph: Graph + Sized {
    /// Convert an existing graph into the current graph storage type.
    ///
    /// This takes an existing graph storage implementation and converts it into the current
    /// graph storage type by extracting its nodes and edges.
    ///
    /// This process is lossy, neither the node ids or the edge ids are guaranteed to be preserved.
    /// This function only guarantees that the weights of the nodes and edges are preserved as well
    /// as the structure of the graph.
    ///
    /// # Example
    ///
    /// ```
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// let a = *storage.insert_node(id, 1).unwrap().id();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// let b = *storage.insert_node(id, 2).unwrap().id();
    ///
    /// # let id = storage.next_edge_id(NoValue::new());
    /// storage.insert_edge(id, 3, &a, &b).unwrap();
    ///
    /// assert_eq!(storage.num_nodes(), 2);
    /// assert_eq!(storage.num_edges(), 1);
    ///
    /// let (nodes, edges) = storage.into_parts();
    ///
    /// let storage = DinoStorage::<_, _, Directed>::from_parts(nodes, edges).unwrap();
    ///
    /// assert_eq!(storage.num_nodes(), 2);
    /// assert_eq!(storage.num_edges(), 1);
    /// ```
    ///
    /// # Errors
    ///
    /// If any of the nodes or edges are invalid, or any of the constraint checks of the underlying
    /// implementation fail, an error is returned.
    ///
    /// The default implementation uses [`Self::insert_node`] and [`Self::insert_edge`] to insert
    /// the nodes and edges, which are fallible.
    /// The default implementation also works in a fail-slow manner, utilizing the `error-stack`
    /// feature of extending errors with others. This means that even if multiple errors occur, all
    /// of them will be returned, but has the potential downside of being slower in cases of
    /// failures.
    ///
    /// Implementations may choose to override this default implementation, but should try to also
    /// be fail-slow.
    // TODO: additionally should return a mapping!
    fn from_graph(
        graph: impl Graph<NodeWeight = Self::NodeWeight, EdgeWeight = Self::EdgeWeight>,
    ) -> Result<Self, Self::Error>;
}

pub trait GraphIntoParts: Graph {
    /// Convert the current graph storage into an iterable of nodes and edges.
    ///
    /// This is the reverse operation of [`Self::from_parts`], which takes an iterable of nodes and
    /// edges and tries to create a graph from them.
    ///
    /// The iterables returned by this function are not guaranteed to be in any particular order,
    /// but must contain all nodes and edges.
    /// The ids of said nodes and edges may also be changed during this operation, but the weights
    /// of the nodes and edges must be the same.
    ///
    /// It must always hold true that using the iterables returned by this function to create a new
    /// graph storage using [`Self::from_parts`] will result in a structurally identical graph and
    /// that calling [`Self::from_parts`] on the same implementation that invoked
    /// [`Self::into_parts`] must not error out.
    ///
    /// # Example
    ///
    /// ```
    /// use std::{collections::HashSet, iter::once};
    ///
    /// # use petgraph_core::attributes::NoValue;
    /// use petgraph_core::{edge::marker::Directed, storage::GraphStorage};
    /// use petgraph_dino::DinoStorage;
    ///
    /// let mut storage = DinoStorage::<u8, u8, Directed>::new();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// let a = *storage.insert_node(id, 1).unwrap().id();
    ///
    /// # let id = storage.next_node_id(NoValue::new());
    /// let b = *storage.insert_node(id, 2).unwrap().id();
    ///
    /// # let id = storage.next_edge_id(NoValue::new());
    /// let ab = *storage.insert_edge(id, 3, &a, &b).unwrap().id();
    ///
    /// assert_eq!(storage.num_nodes(), 2);
    /// assert_eq!(storage.num_edges(), 1);
    ///
    /// let (nodes, edges) = storage.into_parts();
    ///
    /// let node_ids: HashSet<_> = nodes.map(|detached_node| detached_node.id).collect();
    /// let edge_ids: HashSet<_> = edges.map(|detached_edge| detached_edge.id).collect();
    ///
    /// assert_eq!(node_ids, [a, b].into_iter().collect());
    /// assert_eq!(edge_ids, once(ab).collect());
    /// ```
    fn into_parts(
        self,
    ) -> (
        impl Iterator<Item = DetachedNode<Self::NodeWeight>>,
        impl Iterator<Item = DetachedEdge<Self::EdgeWeight>>,
    );
}
