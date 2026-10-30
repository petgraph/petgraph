use core::{
    convert::Infallible,
    fmt::{self, Display},
    hash::Hash,
    ops::AddAssign,
};

use hashbrown::{HashMap, HashSet};

use crate::{
    edge::{Edge, EdgeMut, EdgeRef},
    graph::{EditableGraph, Graph, UndirectedGraph, storable::StorableGraph},
    id::Id,
    node::{Node, NodeMut, NodeRef},
    utils::test_graphs::DefaultMap,
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct UndirNodeId(usize);

impl AddAssign<usize> for UndirNodeId {
    fn add_assign(&mut self, other: usize) {
        self.0 += other;
    }
}

impl Display for UndirNodeId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl Id for UndirNodeId {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct UndirEdgeId(usize);

impl AddAssign<usize> for UndirEdgeId {
    fn add_assign(&mut self, other: usize) {
        self.0 += other;
    }
}

impl Display for UndirEdgeId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl Id for UndirEdgeId {}

pub struct UndirectedTestGraph<N, E, NI = UndirNodeId, EI = UndirEdgeId> {
    next_node: NI,
    next_edge: EI,

    nodes: HashMap<NI, N, foldhash::fast::RandomState>,
    edges: HashMap<EI, (NI, NI, E), foldhash::fast::RandomState>,
}

impl<N, E, NI, EI> UndirectedTestGraph<N, E, NI, EI>
where
    NI: Id + Eq + Hash + AddAssign<usize>,
    EI: Id + Eq + Hash + AddAssign<usize>,
{
    #[must_use]
    pub fn new() -> Self
    where
        NI: Default,
        EI: Default,
    {
        Self {
            next_node: NI::default(),
            next_edge: EI::default(),

            nodes: HashMap::default(),
            edges: HashMap::default(),
        }
    }

    fn add_node(&mut self, node: N) -> NI {
        let id = self.next_node;
        self.next_node += 1;

        self.nodes.insert(id, node);
        id
    }

    fn add_edge(&mut self, source: NI, target: NI, edge: E) -> Option<EI> {
        if !self.nodes.contains_key(&source) || !self.nodes.contains_key(&target) {
            return None;
        }

        let id = self.next_edge;
        self.next_edge += 1;

        self.edges.insert(id, (source, target, edge));
        Some(id)
    }

    fn remove_node(&mut self, node_id: NI) -> Option<N> {
        self.edges
            .retain(|_, (source, target, _)| *source != node_id && *target != node_id);
        self.nodes.remove(&node_id)
    }

    fn remove_edge(&mut self, edge_id: EI) -> Option<E> {
        self.edges.remove(&edge_id).map(|(_, _, data)| data)
    }
}

impl<N, E, NI, EI> Default for UndirectedTestGraph<N, E, NI, EI>
where
    NI: Id + Eq + Hash + AddAssign<usize> + Default,
    EI: Id + Eq + Hash + AddAssign<usize> + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E, NI, EI> Graph for UndirectedTestGraph<N, E, NI, EI>
where
    NI: Id,
    EI: Id,
{
    type EdgeData<'graph>
        = E
    where
        Self: 'graph;
    type EdgeDataMut<'graph>
        = &'graph mut E
    where
        Self: 'graph;
    type EdgeDataRef<'graph>
        = &'graph E
    where
        Self: 'graph;
    type EdgeId = EI;
    type NodeData<'graph>
        = N
    where
        Self: 'graph;
    type NodeDataMut<'graph>
        = &'graph mut N
    where
        Self: 'graph;
    type NodeDataRef<'graph>
        = &'graph N
    where
        Self: 'graph;
    type NodeId = NI;
}

impl<N, E, NI, EI> UndirectedGraph for UndirectedTestGraph<N, E, NI, EI>
where
    NI: Id,
    EI: Id,
{
    fn nodes(&self) -> impl Iterator<Item = NodeRef<'_, Self>> {
        self.nodes.iter().map(|(&id, data)| Node { id, data })
    }

    fn nodes_mut(&mut self) -> impl Iterator<Item = NodeMut<'_, Self>> {
        self.nodes.iter_mut().map(|(&id, data)| Node { id, data })
    }

    fn edges(&self) -> impl Iterator<Item = EdgeRef<'_, Self>> {
        self.edges.iter().map(|(&id, (source, target, data))| Edge {
            id,
            source: *source,
            target: *target,
            data,
        })
    }

    fn edges_mut(&mut self) -> impl Iterator<Item = EdgeMut<'_, Self>> {
        self.edges
            .iter_mut()
            .map(|(&id, (source, target, data))| Edge {
                id,
                source: *source,
                target: *target,
                data,
            })
    }
}

impl<N, E> StorableGraph for UndirectedTestGraph<N, E, UndirNodeId, UndirEdgeId> {
    type EdgeDataContainer<Data: Clone> = DefaultMap<UndirEdgeId, Data>;
    type EdgeVisitContainer = HashSet<UndirEdgeId, foldhash::fast::RandomState>;
    type NodeDataContainer<Data: Clone> = DefaultMap<UndirNodeId, Data>;
    type NodeVisitContainer = HashSet<UndirNodeId, foldhash::fast::RandomState>;

    fn node_visit_container(&self) -> Self::NodeVisitContainer {
        HashSet::default()
    }

    fn node_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::NodeDataContainer<Data> {
        DefaultMap {
            map: HashMap::default(),
            default: default_value,
        }
    }

    fn edge_visit_container(&self) -> Self::EdgeVisitContainer {
        HashSet::default()
    }

    fn edge_data_container<Data: Clone>(
        &self,
        default_value: Data,
    ) -> Self::EdgeDataContainer<Data> {
        DefaultMap {
            map: HashMap::default(),
            default: default_value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceOrTargetNotFound {}

impl Display for SourceOrTargetNotFound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Source or target node not found")
    }
}

impl core::error::Error for SourceOrTargetNotFound {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EdgeNotFoundError {}
impl Display for EdgeNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Edge not found")
    }
}

impl core::error::Error for EdgeNotFoundError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeNotFoundError {}

impl Display for NodeNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Node not found")
    }
}

impl core::error::Error for NodeNotFoundError {}

impl<N, E> EditableGraph for UndirectedTestGraph<N, E, UndirNodeId, UndirEdgeId> {
    type AddEdgeError = SourceOrTargetNotFound;
    type AddNodeError = Infallible;
    type OwnedEdgeData = E;
    type OwnedNodeData = N;
    type RemoveEdgeError = EdgeNotFoundError;
    type RemoveNodeError = NodeNotFoundError;

    fn add_node(&mut self, data: Self::OwnedNodeData) -> Result<Self::NodeId, Self::AddNodeError> {
        Ok(self.add_node(data))
    }

    fn remove_node(
        &mut self,
        node_id: Self::NodeId,
    ) -> Result<Self::OwnedNodeData, Self::RemoveNodeError> {
        self.remove_node(node_id).ok_or(NodeNotFoundError {})
    }

    fn add_edge(
        &mut self,
        source: Self::NodeId,
        target: Self::NodeId,
        data: Self::OwnedEdgeData,
    ) -> Result<Self::EdgeId, Self::AddEdgeError> {
        self.add_edge(source, target, data)
            .ok_or(SourceOrTargetNotFound {})
    }

    fn remove_edge(
        &mut self,
        edge_id: Self::EdgeId,
    ) -> Result<Self::OwnedEdgeData, Self::RemoveEdgeError> {
        self.remove_edge(edge_id).ok_or(EdgeNotFoundError {})
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test_undirected_editable_graph, test_undirected_graph, test_undirected_storable_graph,
    };

    test_undirected_graph!(UndirectedTestGraph::<(), (), UndirNodeId, UndirEdgeId>::new,);
    test_undirected_editable_graph!(UndirectedTestGraph::<(), (), UndirNodeId, UndirEdgeId>::new,);
    test_undirected_storable_graph!(UndirectedTestGraph::<(), (), UndirNodeId, UndirEdgeId>::new,);
}
