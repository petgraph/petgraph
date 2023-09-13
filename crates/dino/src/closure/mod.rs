mod union;

use either::Either;
use fnv::FnvBuildHasher;
// The closure tables have quite a bit of allocations (due to the nested nature of the data
// structure). Question is can we avoid them?
use hashbrown::HashMap;
use roaring::RoaringBitmap;

use self::union::UnionIterator;
use crate::{
    closure::union::UnionIntoIterator,
    edge::Edge,
    node::Node,
    slab::{EntryId, Key as _, Slab},
    EdgeId, NodeId,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum Key {
    SourceToTargets(NodeId),
    TargetToSources(NodeId),

    SourceToEdges(NodeId),
    TargetsToEdges(NodeId),

    EndpointsToEdges(NodeId, NodeId),
}

#[derive(Debug, Clone, PartialEq)]
struct ClosureStorage {
    inner: HashMap<Key, RoaringBitmap, FnvBuildHasher>,
    nodes: RoaringBitmap,
}

impl ClosureStorage {
    fn new() -> Self {
        Self {
            inner: HashMap::with_hasher(FnvBuildHasher::default()),
            nodes: RoaringBitmap::new(),
        }
    }

    fn create_edge<T>(&mut self, edge: &Edge<T>) {
        let raw_index = edge.id.into_id().raw();

        let source = edge.source;
        let target = edge.target;

        self.inner
            .entry(Key::SourceToTargets(source))
            .or_default()
            .insert(target.into_id().raw());

        self.inner
            .entry(Key::TargetToSources(target))
            .or_default()
            .insert(source.into_id().raw());

        self.inner
            .entry(Key::SourceToEdges(source))
            .or_default()
            .insert(raw_index);

        self.inner
            .entry(Key::TargetsToEdges(target))
            .or_default()
            .insert(raw_index);

        self.inner
            .entry(Key::EndpointsToEdges(source, target))
            .or_default()
            .insert(raw_index);
    }

    fn remove_edge<T>(&mut self, edge: &Edge<T>) {
        let raw_index = edge.id.into_id().raw();

        let source = edge.source;
        let target = edge.target;

        let is_multi = self
            .inner
            .get(&Key::EndpointsToEdges(edge.source, edge.target))
            .map_or(false, |bitmap| bitmap.len() > 1);

        if !is_multi {
            if let Some(targets) = self.inner.get_mut(&Key::SourceToTargets(source)) {
                targets.remove(edge.target.into_id().raw());
            }

            if let Some(sources) = self.inner.get_mut(&Key::TargetToSources(target)) {
                sources.remove(edge.source.into_id().raw());
            }
        }

        if let Some(edges) = self.inner.get_mut(&Key::SourceToEdges(source)) {
            println!("removing edge from source to edges");
            edges.remove(raw_index);
        }

        if let Some(edges) = self.inner.get_mut(&Key::TargetsToEdges(target)) {
            println!("removing edge from targets to edges");
            edges.remove(raw_index);
        }

        if let Some(edges) = self.inner.get_mut(&Key::EndpointsToEdges(source, target)) {
            edges.remove(raw_index);

            if edges.is_empty() {
                self.inner.remove(&Key::EndpointsToEdges(source, target));
            }
        }
    }

    fn create_node<T>(&mut self, node: &Node<T>) {
        self.inner
            .insert(Key::SourceToTargets(node.id), RoaringBitmap::new());
        self.inner
            .insert(Key::TargetToSources(node.id), RoaringBitmap::new());

        self.inner
            .insert(Key::SourceToEdges(node.id), RoaringBitmap::new());
        self.inner
            .insert(Key::TargetsToEdges(node.id), RoaringBitmap::new());

        self.nodes.insert(node.id.into_id().raw());
    }

    fn remove_node<T>(&mut self, node: &Node<T>) {
        let raw_index = node.id.into_id().raw();

        let targets = self.inner.remove(&Key::SourceToTargets(node.id));

        if let Some(targets) = targets {
            for target in targets {
                let target = NodeId::from_id(EntryId::new_unchecked(target));
                if let Some(sources) = self.inner.get_mut(&Key::TargetToSources(target)) {
                    sources.remove(raw_index);
                }

                self.inner.remove(&Key::EndpointsToEdges(node.id, target));
            }
        }

        let sources = self.inner.remove(&Key::TargetToSources(node.id));

        if let Some(sources) = sources {
            for source in sources {
                let source = NodeId::from_id(EntryId::new_unchecked(source));
                if let Some(targets) = self.inner.get_mut(&Key::SourceToTargets(source)) {
                    targets.remove(raw_index);
                }

                self.inner.remove(&Key::EndpointsToEdges(source, node.id));
            }
        }

        self.inner.remove(&Key::SourceToEdges(node.id));
        self.inner.remove(&Key::TargetsToEdges(node.id));

        self.nodes.remove(raw_index);
    }

    fn clear(&mut self) {
        self.inner.clear();
    }

    fn refresh<N, E>(&mut self, nodes: &Slab<NodeId, Node<N>>, edges: &Slab<EdgeId, Edge<E>>) {
        self.clear();

        for node in nodes.iter() {
            self.create_node(node);
        }

        for edge in edges.iter() {
            self.create_edge(edge);
        }
    }

    fn reserve(&mut self, additional: usize) {
        self.inner.reserve(additional);
    }

    fn shrink_to_fit(&mut self) {
        self.inner.shrink_to_fit();
    }
}

pub(crate) struct NodeClosure<'a> {
    storage: &'a ClosureStorage,
}

impl<'a> NodeClosure<'a> {
    const fn new(storage: &'a ClosureStorage) -> Self {
        Self { storage }
    }

    pub(crate) fn outgoing_neighbours(&self, id: NodeId) -> impl Iterator<Item = NodeId> + 'a {
        let Some(bitmap) = self.storage.inner.get(&Key::SourceToTargets(id)) else {
            return Either::Left(core::iter::empty());
        };

        Either::Right(
            bitmap
                .iter()
                .map(|value| NodeId::from_id(EntryId::new_unchecked(value))),
        )
    }

    pub(crate) fn incoming_neighbours(&self, id: NodeId) -> impl Iterator<Item = NodeId> + 'a {
        let Some(bitmap) = self.storage.inner.get(&Key::TargetToSources(id)) else {
            return Either::Left(core::iter::empty());
        };

        Either::Right(
            bitmap
                .iter()
                .map(|value| NodeId::from_id(EntryId::new_unchecked(value))),
        )
    }

    pub(crate) fn neighbours(&self, id: NodeId) -> impl Iterator<Item = NodeId> + 'a {
        let Some(left) = self.storage.inner.get(&Key::SourceToTargets(id)) else {
            return Either::Left(core::iter::empty());
        };

        let Some(right) = self.storage.inner.get(&Key::TargetToSources(id)) else {
            return Either::Right(Either::Left(
                left.iter()
                    .map(|value| NodeId::from_id(EntryId::new_unchecked(value))),
            ));
        };

        Either::Right(Either::Right(
            UnionIterator::new(left, right)
                .map(|value| NodeId::from_id(EntryId::new_unchecked(value))),
        ))
    }

    pub(crate) fn outgoing_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + 'a {
        let Some(bitmap) = self.storage.inner.get(&Key::SourceToEdges(id)) else {
            return Either::Left(core::iter::empty());
        };

        Either::Right(
            bitmap
                .iter()
                .map(|value| EdgeId::from_id(EntryId::new_unchecked(value))),
        )
    }

    pub(crate) fn incoming_edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + 'a {
        let Some(bitmap) = self.storage.inner.get(&Key::TargetsToEdges(id)) else {
            return Either::Left(core::iter::empty());
        };

        Either::Right(
            bitmap
                .iter()
                .map(|value| EdgeId::from_id(EntryId::new_unchecked(value))),
        )
    }

    pub(crate) fn edges(&self, id: NodeId) -> impl Iterator<Item = EdgeId> + 'a {
        let Some(left) = self.storage.inner.get(&Key::SourceToEdges(id)) else {
            return Either::Left(core::iter::empty());
        };

        let Some(right) = self.storage.inner.get(&Key::TargetsToEdges(id)) else {
            return Either::Right(Either::Left(
                left.iter()
                    .map(|value| EdgeId::from_id(EntryId::new_unchecked(value))),
            ));
        };

        Either::Right(Either::Right(
            UnionIterator::new(left, right)
                .map(|value| EdgeId::from_id(EntryId::new_unchecked(value))),
        ))
    }

    pub(crate) fn externals(&self) -> impl Iterator<Item = NodeId> + 'a {
        self.storage.nodes.iter().filter_map(|index| {
            let id = NodeId::from_id(EntryId::new_unchecked(index));

            let has_source_to_targets = self
                .storage
                .inner
                .get(&Key::SourceToTargets(id))
                .map_or(false, |bitmap| !bitmap.is_empty());

            let has_target_to_sources = self
                .storage
                .inner
                .get(&Key::TargetToSources(id))
                .map_or(false, |bitmap| !bitmap.is_empty());

            let is_external = !has_source_to_targets && !has_target_to_sources;

            is_external.then_some(id)
        })
    }
}

pub(crate) struct EdgeClosure<'a> {
    storage: &'a ClosureStorage,
}

impl<'a> EdgeClosure<'a> {
    const fn new(storage: &'a ClosureStorage) -> Self {
        Self { storage }
    }

    pub(crate) fn endpoints_to_edges(
        &self,
        source: NodeId,
        target: NodeId,
    ) -> impl Iterator<Item = EdgeId> + 'a {
        let Some(bitmap) = self
            .storage
            .inner
            .get(&Key::EndpointsToEdges(source, target))
        else {
            return Either::Left(core::iter::empty());
        };

        Either::Right(
            bitmap
                .iter()
                .map(|value| EdgeId::from_id(EntryId::new_unchecked(value))),
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Closures {
    storage: ClosureStorage,
}

impl Closures {
    pub(crate) fn new() -> Self {
        Self {
            storage: ClosureStorage::new(),
        }
    }

    pub(crate) const fn nodes(&self) -> NodeClosure<'_> {
        NodeClosure::new(&self.storage)
    }

    pub(crate) fn create_node<T>(&mut self, node: &Node<T>) {
        self.storage.create_node(node);
    }

    pub(crate) fn remove_node<T>(&mut self, node: &Node<T>) {
        self.storage.remove_node(node);
    }

    pub(crate) const fn edges(&self) -> EdgeClosure<'_> {
        EdgeClosure::new(&self.storage)
    }

    pub(crate) fn create_edge<T>(&mut self, edge: &Edge<T>) {
        self.storage.create_edge(edge);
    }

    pub(crate) fn remove_edge<T>(&mut self, edge: &Edge<T>) {
        self.storage.remove_edge(edge);
    }

    /// Drain the `SourceToTargets` and `TargetToSources` entries for the given node
    ///
    /// Returns an iterator of all edges and ensures that there are no duplicates.
    ///
    /// # Note
    ///
    /// To completely remove the edge you also need to call `remove_edge`.
    /// This leaves the closure table in an incomplete (although recoverable) state, use
    /// `remove_node` to completely remove a node, or call `refresh` to rebuild the closure
    /// table.
    pub(crate) fn drain_edges(&mut self, node: NodeId) -> impl Iterator<Item = EdgeId> {
        let left = self.storage.inner.remove(&Key::SourceToEdges(node));
        let right = self.storage.inner.remove(&Key::TargetsToEdges(node));

        let iter = match (left, right) {
            (None, None) => Either::Left(core::iter::empty()),
            (Some(left), None) => Either::Right(Either::Left(left.into_iter())),
            (None, Some(right)) => Either::Right(Either::Left(right.into_iter())),
            (Some(left), Some(right)) => {
                Either::Right(Either::Right(UnionIntoIterator::new(left, right)))
            }
        };

        iter.map(|id| EdgeId::from_id(EntryId::new_unchecked(id)))
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        self.storage.reserve(additional);
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.storage.shrink_to_fit();
    }

    pub(crate) fn refresh<N, E>(
        &mut self,
        nodes: &Slab<NodeId, Node<N>>,
        edges: &Slab<EdgeId, Edge<E>>,
    ) {
        self.storage.refresh(nodes, edges);
    }

    pub(crate) fn clear(&mut self) {
        self.storage.clear();
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec::Vec;
    use core::iter::once;

    use hashbrown::{HashMap, HashSet};
    use petgraph_core::{attributes::Attributes, edge::marker::Directed};
    use roaring::RoaringBitmap;

    use crate::{
        closure::{Closures, Key, NodeClosure, UnionIterator},
        slab::{EntryId, Key as _},
        DinoGraph, EdgeId, NodeId,
    };

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct EvaluatedNodeClosure {
        outgoing_neighbours: HashSet<NodeId>,
        incoming_neighbours: HashSet<NodeId>,

        neighbours: HashSet<NodeId>,

        outgoing_edges: HashSet<EdgeId>,
        incoming_edges: HashSet<EdgeId>,

        edges: HashSet<EdgeId>,
    }

    impl EvaluatedNodeClosure {
        fn new(closures: &Closures, id: NodeId) -> Self {
            let lookup = closures.nodes();

            Self {
                outgoing_neighbours: lookup.outgoing_neighbours(id).collect(),
                incoming_neighbours: lookup.incoming_neighbours(id).collect(),

                neighbours: lookup.neighbours(id).collect(),

                outgoing_edges: lookup.outgoing_edges(id).collect(),
                incoming_edges: lookup.incoming_edges(id).collect(),

                edges: lookup.edges(id).collect(),
            }
        }
    }

    #[derive(Debug, Clone, PartialEq, Eq)]
    pub(crate) struct EvaluatedEdgeClosures {
        source_to_targets: HashMap<NodeId, HashSet<NodeId>>,
        target_to_sources: HashMap<NodeId, HashSet<NodeId>>,

        source_to_edges: HashMap<NodeId, HashSet<EdgeId>>,
        targets_to_edges: HashMap<NodeId, HashSet<EdgeId>>,

        endpoints_to_edges: HashMap<(NodeId, NodeId), HashSet<EdgeId>>,
    }

    impl EvaluatedEdgeClosures {
        fn new(closures: &Closures) -> Self {
            Self {
                source_to_targets: closures
                    .storage
                    .inner
                    .iter()
                    .filter_map(|(key, bitmap)| match key {
                        Key::SourceToTargets(id) => Some((
                            *id,
                            bitmap
                                .iter()
                                .map(|id| NodeId::from_id(EntryId::new_unchecked(id)))
                                .collect::<HashSet<_>>(),
                        )),
                        _ => None,
                    })
                    .collect(),
                target_to_sources: closures
                    .storage
                    .inner
                    .iter()
                    .filter_map(|(key, bitmap)| match key {
                        Key::TargetToSources(id) => Some((
                            *id,
                            bitmap
                                .iter()
                                .map(|id| NodeId::from_id(EntryId::new_unchecked(id)))
                                .collect::<HashSet<_>>(),
                        )),
                        _ => None,
                    })
                    .collect(),

                source_to_edges: closures
                    .storage
                    .inner
                    .iter()
                    .filter_map(|(key, bitmap)| match key {
                        Key::SourceToEdges(id) => Some((
                            *id,
                            bitmap
                                .iter()
                                .map(|id| EdgeId::from_id(EntryId::new_unchecked(id)))
                                .collect::<HashSet<_>>(),
                        )),
                        _ => None,
                    })
                    .collect(),
                targets_to_edges: closures
                    .storage
                    .inner
                    .iter()
                    .filter_map(|(key, bitmap)| match key {
                        Key::TargetsToEdges(id) => Some((
                            *id,
                            bitmap
                                .iter()
                                .map(|id| EdgeId::from_id(EntryId::new_unchecked(id)))
                                .collect::<HashSet<_>>(),
                        )),
                        _ => None,
                    })
                    .collect(),

                endpoints_to_edges: closures
                    .storage
                    .inner
                    .iter()
                    .filter_map(|(key, bitmap)| match key {
                        Key::EndpointsToEdges(source, target) => Some((
                            (*source, *target),
                            bitmap
                                .iter()
                                .map(|id| EdgeId::from_id(EntryId::new_unchecked(id)))
                                .collect::<HashSet<_>>(),
                        )),
                        _ => None,
                    })
                    .collect(),
            }
        }
    }

    macro_rules! map {
        (
            $(
                $key:expr => $value:expr
            ),*
            $(,)?
        ) => {{
            let mut map = ::hashbrown::HashMap::new();
            $(
                map.insert($key, $value);
            )*
            map
        }};
    }

    fn assert_storage_size(closure: &Closures, expected_nodes: usize, expected_endpoints: usize) {
        // expected_nodes * (SourceToTargets + TargetToSources + SourceToEdges + TargetToEdges)
        // + expected_endpoints * EndpointsToEdges

        assert_eq!(
            closure.storage.inner.len(),
            4 * expected_nodes + expected_endpoints
        );
    }

    #[test]
    fn single_node() {
        let mut graph = DinoGraph::<u8, u8, Directed>::new();

        let node = graph.insert_node(1).unwrap();
        let id = *node.id();

        let closures = &graph.storage().closures;

        assert_storage_size(closures, 1, 0);
        assert_eq!(
            closures.nodes().externals().collect::<HashSet<_>>(),
            once(id).collect()
        );

        assert_eq!(
            EvaluatedNodeClosure::new(closures, id),
            EvaluatedNodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: HashSet::new(),
                neighbours: HashSet::new(),
                outgoing_edges: HashSet::new(),
                incoming_edges: HashSet::new(),
                edges: HashSet::new(),
            }
        );

        assert_eq!(
            EvaluatedEdgeClosures::new(closures),
            EvaluatedEdgeClosures {
                source_to_targets: map! {
                    id => HashSet::new(),
                },
                target_to_sources: map! {
                    id => HashSet::new(),
                },
                source_to_edges: map! {
                    id => HashSet::new(),
                },
                targets_to_edges: map! {
                    id => HashSet::new(),
                },
                endpoints_to_edges: HashMap::new(),
            }
        );
    }

    #[test]
    fn multiple_nodes() {
        let mut graph = DinoGraph::<u8, u8, Directed>::new();

        let a = graph.insert_node(Attributes::new(1)).unwrap();
        let a = *a.id();

        let b = graph.insert_node(Attributes::new(2)).unwrap();
        let b = *b.id();

        let closures = &graph.storage().closures;

        assert_storage_size(closures, 2, 0);
        assert_eq!(
            closures.nodes().externals().collect::<HashSet<_>>(),
            [a, b].into_iter().collect()
        );

        assert_eq!(
            EvaluatedNodeClosure::new(closures, a),
            EvaluatedNodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: HashSet::new(),
                neighbours: HashSet::new(),
                outgoing_edges: HashSet::new(),
                incoming_edges: HashSet::new(),
                edges: HashSet::new(),
            }
        );

        assert_eq!(
            EvaluatedNodeClosure::new(closures, b),
            EvaluatedNodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: HashSet::new(),
                neighbours: HashSet::new(),
                outgoing_edges: HashSet::new(),
                incoming_edges: HashSet::new(),
                edges: HashSet::new(),
            }
        );

        assert_eq!(
            EvaluatedEdgeClosures::new(closures),
            EvaluatedEdgeClosures {
                source_to_targets: map! {
                    a => HashSet::new(),
                    b => HashSet::new(),
                },
                target_to_sources: map! {
                    a => HashSet::new(),
                    b => HashSet::new(),
                },
                source_to_edges: map! {
                    a => HashSet::new(),
                    b => HashSet::new(),
                },
                targets_to_edges: map! {
                    a => HashSet::new(),
                    b => HashSet::new(),
                },
                endpoints_to_edges: HashMap::new(),
            }
        );
    }

    #[test]
    fn connection() {
        let mut graph = DinoGraph::<u8, u8, Directed>::new();

        let a = graph.insert_node(1u8).unwrap();
        let a = *a.id();

        let b = graph.insert_node(1u8).unwrap();
        let b = *b.id();

        let edge = graph.insert_edge(1u8, &a, &b).unwrap();
        let edge = *edge.id();

        let closures = &graph.storage().closures;

        assert_storage_size(closures, 2, 1);
        assert_eq!(closures.nodes().externals().count(), 0);

        assert_eq!(
            EvaluatedNodeClosure::new(closures, a),
            EvaluatedNodeClosure {
                outgoing_neighbours: once(b).collect(),
                incoming_neighbours: HashSet::new(),
                neighbours: once(b).collect(),
                outgoing_edges: once(edge).collect(),
                incoming_edges: HashSet::new(),
                edges: once(edge).collect(),
            }
        );

        assert_eq!(
            EvaluatedNodeClosure::new(closures, b),
            EvaluatedNodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: once(a).collect(),
                neighbours: once(a).collect(),
                outgoing_edges: HashSet::new(),
                incoming_edges: once(edge).collect(),
                edges: once(edge).collect(),
            }
        );

        assert_eq!(
            EvaluatedEdgeClosures::new(closures),
            EvaluatedEdgeClosures {
                source_to_targets: map! {
                    a => once(b).collect(),
                    b => HashSet::new(),
                },
                target_to_sources: map! {
                    b => once(a).collect(),
                    a => HashSet::new(),
                },
                source_to_edges: map! {
                    a => once(edge).collect(),
                    b => HashSet::new(),
                },
                targets_to_edges: map! {
                    b => once(edge).collect(),
                    a => HashSet::new(),
                },
                endpoints_to_edges: map! {
                    (a, b) => once(edge).collect(),
                }
            }
        );
    }

    #[test]
    fn self_loop() {
        let mut graph = DinoGraph::<u8, u8, Directed>::new();

        let a = graph.insert_node(1u8).unwrap();
        let a = *a.id();

        let edge = graph.insert_edge(1u8, &a, &a).unwrap();
        let edge = *edge.id();

        let closures = &graph.storage().closures;

        assert_storage_size(closures, 1, 1);
        assert_eq!(closures.nodes().externals().count(), 0);

        assert_eq!(
            EvaluatedNodeClosure::new(closures, a),
            EvaluatedNodeClosure {
                outgoing_neighbours: once(a).collect(),
                incoming_neighbours: once(a).collect(),
                neighbours: once(a).collect(),
                outgoing_edges: once(edge).collect(),
                incoming_edges: once(edge).collect(),
                edges: once(edge).collect(),
            }
        );

        assert_eq!(
            EvaluatedEdgeClosures::new(closures),
            EvaluatedEdgeClosures {
                source_to_targets: map! {
                    a => once(a).collect(),
                },
                target_to_sources: map! {
                    a => once(a).collect(),
                },
                source_to_edges: map! {
                    a => once(edge).collect(),
                },
                targets_to_edges: map! {
                    a => once(edge).collect(),
                },
                endpoints_to_edges: map! {
                    (a, a) => once(edge).collect(),
                }
            }
        );
    }

    struct MultipleConnections {
        graph: DinoGraph<u8, u8, Directed>,

        a: NodeId,
        b: NodeId,
        c: NodeId,

        ab: EdgeId,
        bc: EdgeId,
        ca: EdgeId,
    }

    impl MultipleConnections {
        fn create() -> Self {
            let mut graph = DinoGraph::<u8, u8, Directed>::new();

            let a = graph.insert_node(1u8).unwrap();
            let a = *a.id();

            let b = graph.insert_node(1u8).unwrap();
            let b = *b.id();

            let c = graph.insert_node(1u8).unwrap();
            let c = *c.id();

            let ab = graph.insert_edge(1u8, &a, &b).unwrap();
            let ab = *ab.id();

            let bc = graph.insert_edge(1u8, &b, &c).unwrap();
            let bc = *bc.id();

            let ca = graph.insert_edge(1u8, &c, &a).unwrap();
            let ca = *ca.id();

            Self {
                graph,
                a,
                b,
                c,
                ab,
                bc,
                ca,
            }
        }

        fn assert(&self) {
            let Self {
                graph,
                a,
                b,
                c,
                ab,
                bc,
                ca,
            } = self;

            let (a, b, c, ab, bc, ca) = (*a, *b, *c, *ab, *bc, *ca);

            let closures = &graph.storage().closures;

            assert_storage_size(closures, 3, 3);
            assert_eq!(closures.nodes().externals().count(), 0);

            assert_eq!(
                EvaluatedNodeClosure::new(closures, a),
                EvaluatedNodeClosure {
                    outgoing_neighbours: once(b).collect(),
                    incoming_neighbours: once(c).collect(),
                    neighbours: [b, c].into_iter().collect(),
                    outgoing_edges: once(ab).collect(),
                    incoming_edges: once(ca).collect(),
                    edges: [ab, ca].into_iter().collect(),
                }
            );

            assert_eq!(
                EvaluatedNodeClosure::new(closures, b),
                EvaluatedNodeClosure {
                    outgoing_neighbours: once(c).collect(),
                    incoming_neighbours: once(a).collect(),
                    neighbours: [c, a].into_iter().collect(),
                    outgoing_edges: once(bc).collect(),
                    incoming_edges: once(ab).collect(),
                    edges: [bc, ab].into_iter().collect(),
                }
            );

            assert_eq!(
                EvaluatedNodeClosure::new(closures, c),
                EvaluatedNodeClosure {
                    outgoing_neighbours: once(a).collect(),
                    incoming_neighbours: once(b).collect(),
                    neighbours: [a, b].into_iter().collect(),
                    outgoing_edges: once(ca).collect(),
                    incoming_edges: once(bc).collect(),
                    edges: [ca, bc].into_iter().collect(),
                }
            );

            assert_eq!(
                EvaluatedEdgeClosures::new(closures),
                EvaluatedEdgeClosures {
                    source_to_targets: map! {
                        a => once(b).collect(),
                        b => once(c).collect(),
                        c => once(a).collect(),
                    },
                    target_to_sources: map! {
                        a => once(c).collect(),
                        b => once(a).collect(),
                        c => once(b).collect(),
                    },
                    source_to_edges: map! {
                        a => once(ab).collect(),
                        b => once(bc).collect(),
                        c => once(ca).collect(),
                    },
                    targets_to_edges: map! {
                        a => once(ca).collect(),
                        b => once(ab).collect(),
                        c => once(bc).collect(),
                    },
                    endpoints_to_edges: map! {
                        (a, b) => once(ab).collect(),
                        (b, c) => once(bc).collect(),
                        (c, a) => once(ca).collect(),
                    },
                }
            );
        }
    }

    #[test]
    fn multiple_connections() {
        let graph = MultipleConnections::create();
        graph.assert();
    }

    #[test]
    fn multi_graph() {
        let mut graph = DinoGraph::<u8, u8, Directed>::new();

        let a = graph.insert_node(1u8).unwrap();
        let a = *a.id();

        let b = graph.insert_node(1u8).unwrap();
        let b = *b.id();

        let ab1 = graph.insert_edge(1u8, &a, &b).unwrap();
        let ab1 = *ab1.id();

        let ab2 = graph.insert_edge(1u8, &a, &b).unwrap();
        let ab2 = *ab2.id();

        let closures = &graph.storage().closures;

        assert_storage_size(closures, 2, 1);
        assert_eq!(closures.nodes().externals().count(), 0);

        assert_eq!(
            EvaluatedNodeClosure::new(closures, a),
            EvaluatedNodeClosure {
                outgoing_neighbours: once(b).collect(),
                incoming_neighbours: HashSet::new(),
                neighbours: once(b).collect(),
                outgoing_edges: [ab1, ab2].into_iter().collect(),
                incoming_edges: HashSet::new(),
                edges: [ab1, ab2].into_iter().collect(),
            }
        );

        assert_eq!(
            EvaluatedNodeClosure::new(closures, b),
            EvaluatedNodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: once(a).collect(),
                neighbours: once(a).collect(),
                outgoing_edges: HashSet::new(),
                incoming_edges: [ab1, ab2].into_iter().collect(),
                edges: [ab1, ab2].into_iter().collect(),
            }
        );

        assert_eq!(
            EvaluatedEdgeClosures::new(closures),
            EvaluatedEdgeClosures {
                source_to_targets: map! {
                    a => once(b).collect(),
                    b => HashSet::new(),
                },
                target_to_sources: map! {
                    a => HashSet::new(),
                    b => once(a).collect(),
                },
                source_to_edges: map! {
                    a => [ab1, ab2].into_iter().collect(),
                    b => HashSet::new(),
                },
                targets_to_edges: map! {
                    a => HashSet::new(),
                    b => [ab1, ab2].into_iter().collect(),
                },
                endpoints_to_edges: map! {
                    (a, b) => [ab1, ab2].into_iter().collect(),
                },
            }
        );
    }

    #[test]
    fn remove_node() {
        let graph = MultipleConnections::create();
        graph.assert();

        let MultipleConnections {
            mut graph,
            a,
            b,
            c,
            ab,
            bc,
            ca,
        } = graph;

        graph.remove_node(&b).unwrap();

        let closures = &graph.storage().closures;

        assert_storage_size(closures, 2, 1);
        assert_eq!(closures.nodes().externals().count(), 0);

        assert_eq!(
            EvaluatedNodeClosure::new(closures, a),
            EvaluatedNodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: once(c).collect(),
                neighbours: once(c).collect(),
                outgoing_edges: HashSet::new(),
                incoming_edges: once(ca).collect(),
                edges: once(ca).collect(),
            }
        );

        assert_eq!(
            EvaluatedNodeClosure::new(closures, c),
            EvaluatedNodeClosure {
                outgoing_neighbours: once(a).collect(),
                incoming_neighbours: HashSet::new(),
                neighbours: once(a).collect(),
                outgoing_edges: once(ca).collect(),
                incoming_edges: HashSet::new(),
                edges: once(ca).collect(),
            }
        );

        assert_eq!(
            EvaluatedEdgeClosures::new(closures),
            EvaluatedEdgeClosures {
                source_to_targets: map! {
                    a => HashSet::new(),
                    c => once(a).collect(),
                },
                target_to_sources: map! {
                    a => once(c).collect(),
                    c => HashSet::new(),
                },
                source_to_edges: map! {
                    a => HashSet::new(),
                    c => once(ca).collect(),
                },
                targets_to_edges: map! {
                    a => once(ca).collect(),
                    c => HashSet::new(),
                },
                endpoints_to_edges: map! {
                    (c, a) => once(ca).collect(),
                },
            }
        );
    }

    #[test]
    fn remove_edge() {
        let graph = MultipleConnections::create();
        graph.assert();

        let MultipleConnections {
            mut graph,
            a,
            b,
            c,
            ab,
            bc,
            ca,
        } = graph;

        graph.remove_edge(&bc).unwrap();

        let closures = &graph.storage().closures;

        assert_storage_size(closures, 3, 2);
        assert_eq!(closures.nodes().externals().count(), 0);

        assert_eq!(
            EvaluatedNodeClosure::new(closures, a),
            EvaluatedNodeClosure {
                outgoing_neighbours: once(b).collect(),
                incoming_neighbours: once(c).collect(),
                neighbours: [b, c].into_iter().collect(),
                outgoing_edges: once(ab).collect(),
                incoming_edges: once(ca).collect(),
                edges: [ab, ca].into_iter().collect(),
            }
        );

        assert_eq!(
            EvaluatedNodeClosure::new(closures, b),
            EvaluatedNodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: once(a).collect(),
                neighbours: once(a).collect(),
                outgoing_edges: HashSet::new(),
                incoming_edges: once(ab).collect(),
                edges: once(ab).collect(),
            }
        );

        assert_eq!(
            EvaluatedNodeClosure::new(closures, c),
            EvaluatedNodeClosure {
                outgoing_neighbours: once(a).collect(),
                incoming_neighbours: HashSet::new(),
                neighbours: once(a).collect(),
                outgoing_edges: once(ca).collect(),
                incoming_edges: HashSet::new(),
                edges: once(ca).collect(),
            }
        );

        assert_eq!(
            EvaluatedEdgeClosures::new(closures),
            EvaluatedEdgeClosures {
                source_to_targets: map! {
                    a => once(b).collect(),
                    b => HashSet::new(),
                    c => once(a).collect(),
                },
                target_to_sources: map! {
                    a => once(c).collect(),
                    b => once(a).collect(),
                    c => HashSet::new(),
                },
                source_to_edges: map! {
                    a => once(ab).collect(),
                    b => HashSet::new(),
                    c => once(ca).collect(),
                },
                targets_to_edges: map! {
                    a => once(ca).collect(),
                    b => once(ab).collect(),
                    c => HashSet::new(),
                },
                endpoints_to_edges: map! {
                    (a, b) => once(ab).collect(),
                    (c, a) => once(ca).collect(),
                },
            }
        );
    }
}
