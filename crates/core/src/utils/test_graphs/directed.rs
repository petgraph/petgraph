use alloc::{collections::VecDeque, vec::Vec};
use core::{
    fmt::{self, Display},
    hash::Hash,
    ops::AddAssign,
};

use hashbrown::{HashMap, HashSet};

use crate::{
    edge::{Edge, EdgeMut, EdgeRef},
    graph::{
        DirectedGraph, Graph,
        storable::{DataContainer, Storable},
    },
    id::{Id, IndexId, IndexIdTryFromIntError},
    node::{Node, NodeMut, NodeRef},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct DirNodeId(usize);

impl AddAssign<usize> for DirNodeId {
    fn add_assign(&mut self, other: usize) {
        self.0 += other;
    }
}

impl Display for DirNodeId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl Id for DirNodeId {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct DirEdgeId(usize);

impl TryFrom<u16> for DirNodeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<u32> for DirNodeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<u64> for DirNodeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<usize> for DirNodeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl IndexId for DirNodeId {
    const MAX: Self = Self(usize::MAX);
    const MIN: Self = Self(0);

    fn as_u16(self) -> u16 {
        self.0 as u16
    }

    fn as_u32(self) -> u32 {
        self.0 as u32
    }

    fn as_u64(self) -> u64 {
        self.0 as u64
    }

    fn as_usize(self) -> usize {
        self.0
    }
}

impl AddAssign<usize> for DirEdgeId {
    fn add_assign(&mut self, other: usize) {
        self.0 += other;
    }
}

impl Display for DirEdgeId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl Id for DirEdgeId {}

impl TryFrom<u16> for DirEdgeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<u32> for DirEdgeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<u64> for DirEdgeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<usize> for DirEdgeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl IndexId for DirEdgeId {
    const MAX: Self = Self(usize::MAX);
    const MIN: Self = Self(0);

    fn as_u16(self) -> u16 {
        self.0 as u16
    }

    fn as_u32(self) -> u32 {
        self.0 as u32
    }

    fn as_u64(self) -> u64 {
        self.0 as u64
    }

    fn as_usize(self) -> usize {
        self.0
    }
}

pub struct DirectedTestGraph<N, E, NI = DirNodeId, EI = DirEdgeId> {
    next_node: NI,
    next_edge: EI,

    nodes: HashMap<NI, N, foldhash::fast::RandomState>,
    edges: HashMap<EI, (NI, NI, E), foldhash::fast::RandomState>,
}

impl<N, E, NI, EI> DirectedTestGraph<N, E, NI, EI>
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

    pub fn add_node(&mut self, node: N) -> NI {
        let id = self.next_node;
        self.next_node += 1;

        self.nodes.insert(id, node);
        id
    }

    pub fn add_edge(&mut self, source: NI, target: NI, edge: E) -> Option<EI> {
        if !self.nodes.contains_key(&source) || !self.nodes.contains_key(&target) {
            return None;
        }

        let id = self.next_edge;
        self.next_edge += 1;

        self.edges.insert(id, (source, target, edge));
        Some(id)
    }

    pub fn remove_node(&mut self, node_id: NI) -> Option<N> {
        self.edges
            .retain(|_, (source, target, _)| *source != node_id && *target != node_id);
        self.nodes.remove(&node_id)
    }

    pub fn remove_edge(&mut self, edge_id: EI) -> Option<E> {
        self.edges.remove(&edge_id).map(|(_, _, data)| data)
    }

    pub fn extend_with_edges<IntoNI: TryInto<NI>>(
        &mut self,
        edges: impl IntoIterator<Item = (IntoNI, IntoNI, E)>,
    ) {
        for (source, target, edge) in edges {
            self.add_edge(
                source.try_into().ok().unwrap(),
                target.try_into().ok().unwrap(),
                edge,
            );
        }
    }
}

impl<N, E, NI, EI> Default for DirectedTestGraph<N, E, NI, EI>
where
    NI: Id + Eq + Hash + AddAssign<usize> + Default,
    EI: Id + Eq + Hash + AddAssign<usize> + Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<N, E, NI, EI> Graph for DirectedTestGraph<N, E, NI, EI>
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

impl<N, E, NI, EI> DirectedGraph for DirectedTestGraph<N, E, NI, EI>
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

pub struct DefaultMap<Id, Data> {
    map: HashMap<Id, Data>,
    default: Data,
}

impl<Id: Eq + Hash, Data: Default> DataContainer<Id, Data> for DefaultMap<Id, Data> {
    fn get(&self, node_id: Id) -> &Data {
        self.map.get(&node_id).unwrap_or(&self.default)
    }

    fn get_mut(&mut self, node_id: Id) -> &mut Data {
        self.map.entry(node_id).or_insert_with(Default::default)
    }

    fn insert(&mut self, node_id: Id, data: Data) {
        self.map.insert(node_id, data);
    }

    fn remove(&mut self, node_id: Id) {
        self.map.remove(&node_id);
    }

    fn clear(&mut self) {
        self.map.clear();
    }

    fn into_iter(self) -> impl Iterator<Item = (Id, Data)> {
        self.map.into_iter()
    }
}

impl<N, E> Storable for DirectedTestGraph<N, E, DirNodeId, DirEdgeId> {
    type EdgeDataContainer<Data: Default> = DefaultMap<DirEdgeId, Data>;
    type EdgeVisitContainer = HashSet<DirEdgeId, foldhash::fast::RandomState>;
    type NodeDataContainer<Data: Default> = DefaultMap<DirNodeId, Data>;
    type NodeVisitContainer = HashSet<DirNodeId, foldhash::fast::RandomState>;
    type QueueContainer<Data> = VecDeque<Data>;
    type StackContainer<Data> = Vec<Data>;

    fn node_visit_container(&self) -> Self::NodeVisitContainer {
        HashSet::default()
    }

    fn node_data_container<Data: Default>(&self) -> Self::NodeDataContainer<Data> {
        DefaultMap {
            map: HashMap::default(),
            default: Data::default(),
        }
    }

    fn edge_visit_container(&self) -> Self::EdgeVisitContainer {
        HashSet::default()
    }

    fn edge_data_container<Data: Default>(&self) -> Self::EdgeDataContainer<Data> {
        DefaultMap {
            map: HashMap::default(),
            default: Data::default(),
        }
    }

    fn stack_container<Data>(&self) -> Self::StackContainer<Data> {
        Vec::new()
    }

    fn queue_container<Data>(&self) -> Self::QueueContainer<Data> {
        VecDeque::new()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_directed_graph;

    fn remove_node_with_unwrap(
        graph: &mut DirectedTestGraph<(), (), DirNodeId, DirEdgeId>,
        node_id: DirNodeId,
    ) {
        graph.remove_node(node_id).unwrap();
    }

    fn add_edge_with_unwrap(
        graph: &mut DirectedTestGraph<(), (), DirNodeId, DirEdgeId>,
        source: DirNodeId,
        target: DirNodeId,
        _data: (),
    ) -> DirEdgeId {
        graph.add_edge(source, target, ()).unwrap()
    }

    fn remove_edge_with_unwrap(
        graph: &mut DirectedTestGraph<(), (), DirNodeId, DirEdgeId>,
        edge_id: DirEdgeId,
    ) {
        graph.remove_edge(edge_id).unwrap();
    }

    test_directed_graph!(
        DirectedTestGraph::<(), (), DirNodeId, DirEdgeId>::new,
        DirectedTestGraph::<(), (), DirNodeId, DirEdgeId>::add_node,
        remove_node_with_unwrap,
        add_edge_with_unwrap,
        remove_edge_with_unwrap
    );
}
