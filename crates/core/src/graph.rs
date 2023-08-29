use crate::{
    edge::{DetachedEdge, Direction, Edge, EdgeMut},
    index::{ArbitraryGraphIndex, ManagedGraphIndex},
    node::{DetachedNode, Node, NodeMut},
    storage::{DirectedGraphStorage, GraphStorage},
};

pub struct Graph<S> {
    storage: S,
}

impl<S> Graph<S>
where
    S: GraphStorage,
{
    pub fn new() -> Self {
        Self {
            storage: S::with_capacity(None, None),
        }
    }

    pub fn with_capacity(node_capacity: Option<usize>, edge_capacity: Option<usize>) -> Self {
        Self {
            storage: S::with_capacity(node_capacity, edge_capacity),
        }
    }

    pub fn num_nodes(&self) -> usize {
        self.storage.num_nodes()
    }

    pub fn num_edges(&self) -> usize {
        self.storage.num_edges()
    }

    pub fn is_empty(&self) -> bool {
        self.num_nodes() == 0 && self.num_edges() == 0
    }

    pub fn clear(&mut self) {
        self.storage.clear();
    }

    pub fn node(&self, id: &S::NodeIndex) -> Option<Node<S>> {
        self.storage.node(id)
    }

    pub fn node_mut(&mut self, id: &S::NodeIndex) -> Option<NodeMut<S>> {
        self.storage.node_mut(id)
    }

    pub fn remove_node(
        &mut self,
        id: S::NodeIndex,
    ) -> Option<DetachedNode<S::NodeIndex, S::NodeWeight>> {
        self.storage.remove_node(id)
    }

    pub fn edge(&self, id: &S::EdgeIndex) -> Option<Edge<S>> {
        self.storage.edge(id)
    }

    pub fn edge_mut(&mut self, id: &S::EdgeIndex) -> Option<EdgeMut<S>> {
        self.storage.edge_mut(id)
    }

    pub fn remove_edge(
        &mut self,
        id: S::EdgeIndex,
    ) -> Option<DetachedEdge<S::EdgeIndex, S::NodeIndex, S::EdgeWeight>> {
        self.storage.remove_edge(id)
    }

    #[inline(always)]
    pub fn neighbors(&self, id: &S::NodeIndex) -> impl Iterator<Item = Node<S>> {
        self.neighbours(id)
    }

    pub fn neighbours(&self, id: &S::NodeIndex) -> impl Iterator<Item = Node<S>> {
        self.storage.node_neighbours(id)
    }

    #[inline(always)]
    pub fn neighbors_mut(&mut self, id: &S::NodeIndex) -> impl Iterator<Item = NodeMut<S>> {
        self.neighbours_mut(id)
    }

    pub fn neighbours_mut(&mut self, id: &S::NodeIndex) -> impl Iterator<Item = NodeMut<S>> {
        self.storage.node_neighbours_mut(id)
    }

    pub fn connections(&self, id: &S::NodeIndex) -> impl Iterator<Item = Edge<S>> {
        self.storage.node_connections(id)
    }

    pub fn connections_mut(&mut self, id: &S::NodeIndex) -> impl Iterator<Item = EdgeMut<S>> {
        self.storage.node_connections_mut(id)
    }
}

impl<S> Graph<S>
where
    S: DirectedGraphStorage,
{
    #[inline(always)]
    pub fn neighbors_directed(
        &self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = S::NodeIndex> {
        self.neighbours_directed(id, direction)
    }

    pub fn neighbours_directed(
        &self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = S::NodeIndex> {
        self.storage.node_directed_neighbours(id, direction)
    }

    pub fn neighbors_directed_mut(
        &mut self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = S::NodeIndex> {
        self.neighbours_directed_mut(id, direction)
    }

    #[inline(always)]
    pub fn neighbours_directed_mut(
        &mut self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = S::NodeIndex> {
        self.storage.node_directed_neighbours_mut(id, direction)
    }

    pub fn connections_directed(
        &self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = Edge<S>> {
        self.storage.node_directed_connections(id, direction)
    }

    pub fn connections_directed_mut(
        &mut self,
        id: &S::NodeIndex,
        direction: Direction,
    ) -> impl Iterator<Item = EdgeMut<S>> {
        self.storage.node_directed_connections_mut(id, direction)
    }

    // TODO: find_edge, find_edge_undirected, externals, externals_mut, edges, nodes, edge_mut,
    // nodes_mut, reverse, retain, map, filter_map, filter, into_undirected, into_directed,
    // from_parts

    // TODO: (storage):
    //  * find_edge (default impl)
    //  * find_edge_undirected (default impl),
    //  * externals (default impl),
    //  * externals_mut (default impl)
    //  * retain (no default impl) ~> only if marker trait!
    // GraphResize as trait instead!
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::NodeIndex: ManagedGraphIndex<S>,
{
    pub fn insert_node(&mut self, weight: S::NodeWeight) -> Result<S::NodeIndex, S::Error> {
        let id = S::NodeIndex::next(&self.storage);

        self.storage.insert_node(id, weight)
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::EdgeIndex: ManagedGraphIndex<S>,
{
    pub fn insert_edge(
        &mut self,
        source: S::NodeIndex,
        target: S::NodeIndex,
        weight: S::EdgeWeight,
    ) -> Result<S::EdgeIndex, S::Error> {
        let id = S::EdgeIndex::next(&self.storage);

        self.storage.insert_edge(id, source, target, weight)
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::NodeIndex: ArbitraryGraphIndex<S>,
{
    pub fn insert_node(&mut self, id: S::NodeIndex, weight: S::NodeWeight) -> Result<(), S::Error> {
        self.storage.insert_node(id, weight)
    }

    pub fn upsert_node(&mut self, id: S::NodeIndex, weight: S::NodeWeight) -> Result<(), S::Error> {
        if let Some(mut node) = self.storage.node_mut(&id) {
            *node.weight_mut() = weight;
            Ok(())
        } else {
            self.storage.insert_node(id, weight)
        }
    }
}

impl<S> Graph<S>
where
    S: GraphStorage,
    S::EdgeIndex: ArbitraryGraphIndex<S>,
{
    pub fn insert_edge(
        &mut self,
        id: S::EdgeIndex,
        source: S::NodeIndex,
        target: S::NodeIndex,
        weight: S::EdgeWeight,
    ) -> Result<(), S::Error> {
        self.storage.insert_edge(id, source, target, weight)
    }

    pub fn upsert_edge(
        &mut self,
        id: S::EdgeIndex,
        source: S::NodeIndex,
        target: S::NodeIndex,
        weight: S::EdgeWeight,
    ) -> Result<(), S::Error> {
        if let Some(mut edge) = self.storage.edge_mut(&id) {
            *edge.weight_mut() = weight;
            Ok(())
        } else {
            self.storage.insert_edge(id, source, target, weight)
        }
    }
}
