//! Auxiliary storage for graphs.
//!
//! This module provides traits for auxiliary storage for graphs, which can be used to associate
//! arbitrary additional data with nodes and edges.
use crate::{edge::EdgeId, node::NodeId, GraphStorage};

/// Hints about frequency.
///
/// These hints are used to optimize the performance of secondary storage, specifically these are
/// used to tell how frequently data is expected to be accessed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum FrequencyHint {
    /// Items are accessed frequently.
    #[default]
    Frequent,
    /// Items are accessed infrequently.
    Infrequent,
}

/// Hints about performance.
///
/// These hints are used to optimize the performance of secondary storage, specifically these are
/// used to tell how frequently data is expected to be accessed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PerformanceHint {
    /// Hints about the frequency of reads.
    pub read: FrequencyHint,
    /// Hints about the frequency of writes.
    pub write: FrequencyHint,
}

/// Hints about occupancy.
///
/// These hints are used to optimize the performance of secondary storage, specifically these are
/// used to tell how much space is expected to be filled with data.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OccupancyHint {
    /// Storage is expected to be densely populated.
    #[default]
    Dense,
    /// Storage is expected to be sparsely populated.
    Sparse,
}

/// Hints for secondary storage.
///
/// These hints are used to optimize the performance of secondary storage.
///
/// These hints are not guaranteed to be respected, and are used as a best-effort heuristic.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Hints {
    /// Hints about performance.
    pub performance: PerformanceHint,
    /// Hints about occupancy.
    pub occupancy: OccupancyHint,
}

/// Secondary storage for a graph.
///
/// This trait is used to provide secondary storage for a graph to associate arbitrary additional
/// data with nodes and edges.
///
/// If you want to only store boolean values for nodes and edges, you can use the
/// [`BooleanGraphStorage`] trait instead, which usually has a more efficient implementation.
pub trait SecondaryGraphStorage<K, V> {
    type Iter<'a>: Iterator<Item = (K, &'a V)>
    where
        V: 'a,
        Self: 'a;

    fn get(&self, id: K) -> Option<&V>;
    fn get_mut(&mut self, id: K) -> Option<&mut V>;

    fn set(&mut self, id: K, value: V) -> Option<V>;
    fn remove(&mut self, id: K) -> Option<V>;

    fn iter(&self) -> Self::Iter<'_>;
}

/// Secondary storage for a graph.
///
/// This trait is used to provide secondary storage for a graph to associate boolean values with
/// nodes and edges.
pub trait BooleanGraphStorage<K> {
    fn get(&self, id: K) -> Option<bool>;

    fn set(&mut self, id: K, flag: bool) -> Option<bool>;
}

/// Auxiliary storage for a graph.
///
/// Provides secondary storage for a graph to associate arbitrary additional data with nodes and
/// edges, as well as boolean values with nodes and edges.
///
/// For boolean values prefer [`Self::boolean_edge_storage`] and [`Self::boolean_node_storage`],
/// as they usually have a more efficient implementation.
pub trait AuxiliaryGraphStorage: GraphStorage {
    type BooleanEdgeStorage<'graph>: BooleanGraphStorage<EdgeId>
    where
        Self: 'graph;

    type BooleanNodeStorage<'graph>: BooleanGraphStorage<NodeId>
    where
        Self: 'graph;

    type SecondaryEdgeStorage<'graph, V>: SecondaryGraphStorage<EdgeId, V>
    where
        Self: 'graph;

    type SecondaryNodeStorage<'graph, V>: SecondaryGraphStorage<NodeId, V>
    where
        Self: 'graph;

    fn secondary_node_storage<V>(&self, hints: Hints) -> Self::SecondaryNodeStorage<'_, V>;
    fn secondary_edge_storage<V>(&self, hints: Hints) -> Self::SecondaryEdgeStorage<'_, V>;

    fn boolean_node_storage(&self, hints: Hints) -> Self::BooleanNodeStorage<'_>;

    fn boolean_edge_storage(&self, hints: Hints) -> Self::BooleanEdgeStorage<'_>;
}
