use core::{
    fmt::{self, Display},
    hash::Hash,
    ops::AddAssign,
};

use hashbrown::HashMap;

use crate::{
    edge::{Edge, EdgeMut, EdgeRef},
    graph::{DirectedGraph, Graph},
    id::Id,
    node::{Node, NodeMut, NodeRef},
};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct NodeId(usize);

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

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Default)]
pub struct EdgeId(usize);

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

    pub fn remove_node(&mut self, node_id: NI) -> Option<N> {
        self.edges
            .retain(|_, (source, target, _)| *source != node_id && *target != node_id);
        self.nodes.remove(&node_id)
    }

    pub fn remove_edge(&mut self, edge_id: EI) -> Option<E> {
        self.edges.remove(&edge_id).map(|(_, _, data)| data)
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

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_directed_graph;

    fn remove_node_with_unwrap(
        graph: &mut DirectedTestGraph<(), (), NodeId, EdgeId>,
        node_id: NodeId,
    ) {
        graph.remove_node(node_id).unwrap();
    }

    fn add_edge_with_unwrap(
        graph: &mut DirectedTestGraph<(), (), NodeId, EdgeId>,
        source: NodeId,
        target: NodeId,
        _data: (),
    ) -> EdgeId {
        graph.add_edge(source, target, ()).unwrap()
    }

    fn remove_edge_with_unwrap(
        graph: &mut DirectedTestGraph<(), (), NodeId, EdgeId>,
        edge_id: EdgeId,
    ) {
        graph.remove_edge(edge_id).unwrap();
    }

    test_directed_graph!(
        DirectedTestGraph::<(), (), NodeId, EdgeId>::new,
        DirectedTestGraph::<(), (), NodeId, EdgeId>::add_node,
        remove_node_with_unwrap,
        add_edge_with_unwrap,
        remove_edge_with_unwrap
    );
}
