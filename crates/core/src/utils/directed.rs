use core::{
    fmt::{self, Display},
    hash::Hash,
    ops::AddAssign,
};

use hashbrown::HashMap;

use crate::{
    edge::{Edge, EdgeMut, EdgeRef},
    graph::{DirectedGraph, Graph},
    id::{Id, IndexId, IndexIdTryFromIntError},
    node::{Node, NodeMut, NodeRef},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct NodeId(pub usize);

impl AddAssign<usize> for NodeId {
    fn add_assign(&mut self, other: usize) {
        self.0 += other;
    }
}

impl Display for NodeId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl Id for NodeId {}

impl TryFrom<u16> for NodeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<u32> for NodeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<u64> for NodeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<usize> for NodeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl IndexId for NodeId {
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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default, PartialOrd, Ord)]
pub struct EdgeId(pub usize);

impl AddAssign<usize> for EdgeId {
    fn add_assign(&mut self, other: usize) {
        self.0 += other;
    }
}

impl Display for EdgeId {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.0, fmt)
    }
}

impl Id for EdgeId {}

impl TryFrom<u16> for EdgeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<u32> for EdgeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u32) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<u64> for EdgeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: u64) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl TryFrom<usize> for EdgeId {
    type Error = IndexIdTryFromIntError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        Ok(Self(value as usize))
    }
}

impl IndexId for EdgeId {
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

pub struct DirectedTestGraph<N, E, NI = NodeId, EI = EdgeId> {
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

    pub fn remove_node(&mut self, node_id: NI) {
        self.nodes.remove(&node_id);
        self.edges
            .retain(|_, (source, target, _)| *source != node_id && *target != node_id);
    }

    pub fn remove_edge(&mut self, edge_id: EI) {
        self.edges.remove(&edge_id);
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
