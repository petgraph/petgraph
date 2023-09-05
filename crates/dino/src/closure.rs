// The closure tables have quite a bit of allocations (due to the nested nature of the data
// structure). Question is can we avoid them?
use hashbrown::{HashMap, HashSet};

use crate::{edge::Edge, node::Node, slab::Slab, EdgeId, NodeId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NodeClosure {
    outgoing_neighbours: HashSet<NodeId>,
    incoming_neighbours: HashSet<NodeId>,

    neighbours: HashSet<NodeId>,

    outgoing_edges: HashSet<EdgeId>,
    incoming_edges: HashSet<EdgeId>,

    edges: HashSet<EdgeId>,
}

impl NodeClosure {
    fn new() -> Self {
        Self {
            outgoing_neighbours: HashSet::new(),
            incoming_neighbours: HashSet::new(),

            neighbours: HashSet::new(),

            outgoing_edges: HashSet::new(),
            incoming_edges: HashSet::new(),

            edges: HashSet::new(),
        }
    }

    pub(crate) const fn outgoing_neighbours(&self) -> &HashSet<NodeId> {
        &self.outgoing_neighbours
    }

    pub(crate) const fn incoming_neighbours(&self) -> &HashSet<NodeId> {
        &self.incoming_neighbours
    }

    pub(crate) const fn neighbours(&self) -> &HashSet<NodeId> {
        &self.neighbours
    }

    pub(crate) const fn outgoing_edges(&self) -> &HashSet<EdgeId> {
        &self.outgoing_edges
    }

    pub(crate) const fn incoming_edges(&self) -> &HashSet<EdgeId> {
        &self.incoming_edges
    }

    pub(crate) const fn edges(&self) -> &HashSet<EdgeId> {
        &self.edges
    }

    fn refresh(&mut self, id: NodeId, closure: &EdgeClosures) {
        self.outgoing_neighbours.clear();
        self.incoming_neighbours.clear();
        self.neighbours.clear();

        self.outgoing_edges.clear();
        self.incoming_edges.clear();
        self.edges.clear();

        if let Some(source_to_targets) = closure.source_to_targets.get(&id) {
            self.outgoing_neighbours
                .extend(source_to_targets.iter().copied());
            self.neighbours.extend(source_to_targets.iter().copied());
        }

        if let Some(target_to_sources) = closure.target_to_sources.get(&id) {
            self.incoming_neighbours
                .extend(target_to_sources.iter().copied());
            self.neighbours.extend(target_to_sources.iter().copied());
        }

        if let Some(source_to_edges) = closure.source_to_edges.get(&id) {
            self.outgoing_edges.extend(source_to_edges.iter().copied());
            self.edges.extend(source_to_edges.iter().copied());
        }

        if let Some(targets_to_edges) = closure.targets_to_edges.get(&id) {
            self.incoming_edges.extend(targets_to_edges.iter().copied());
            self.edges.extend(targets_to_edges.iter().copied());
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct EdgeClosures {
    source_to_targets: HashMap<NodeId, HashSet<NodeId>>,
    target_to_sources: HashMap<NodeId, HashSet<NodeId>>,

    source_to_edges: HashMap<NodeId, HashSet<EdgeId>>,
    targets_to_edges: HashMap<NodeId, HashSet<EdgeId>>,

    endpoints_to_edges: HashMap<(NodeId, NodeId), HashSet<EdgeId>>,
}

impl EdgeClosures {
    fn new() -> Self {
        Self {
            source_to_targets: HashMap::new(),
            target_to_sources: HashMap::new(),

            source_to_edges: HashMap::new(),
            targets_to_edges: HashMap::new(),

            endpoints_to_edges: HashMap::new(),
        }
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        self.source_to_targets.reserve(additional);
        self.target_to_sources.reserve(additional);

        self.source_to_edges.reserve(additional);
        self.targets_to_edges.reserve(additional);

        self.endpoints_to_edges.reserve(additional);
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.source_to_targets.shrink_to_fit();
        self.target_to_sources.shrink_to_fit();

        self.source_to_edges.shrink_to_fit();
        self.targets_to_edges.shrink_to_fit();

        self.endpoints_to_edges.shrink_to_fit();
    }

    pub(crate) const fn endpoints_to_edges(&self) -> &HashMap<(NodeId, NodeId), HashSet<EdgeId>> {
        &self.endpoints_to_edges
    }

    fn update<E>(&mut self, edge: &Edge<E>) {
        self.source_to_targets
            .entry(edge.source)
            .or_default()
            .insert(edge.target);

        self.target_to_sources
            .entry(edge.target)
            .or_default()
            .insert(edge.source);

        self.source_to_edges
            .entry(edge.source)
            .or_default()
            .insert(edge.id);

        self.targets_to_edges
            .entry(edge.target)
            .or_default()
            .insert(edge.id);

        self.endpoints_to_edges
            .entry((edge.source, edge.target))
            .or_default()
            .insert(edge.id);
    }

    fn remove<E>(&mut self, edge: &Edge<E>) {
        // lookup in the current closure if there are multiple edges with the same source and target
        let multi = 'multi: {
            let matching_source = self.source_to_edges.get(&edge.source);
            let matching_target = self.targets_to_edges.get(&edge.target);

            let (Some(source), Some(target)) = (matching_source, matching_target) else {
                break 'multi false;
            };

            // This avoids an additional allocation
            let same_source_and_target = source
                .iter()
                .any(|&id| edge.id != id && target.contains(&id));

            same_source_and_target
        };

        if !multi {
            if let Some(targets) = self.source_to_targets.get_mut(&edge.source) {
                targets.remove(&edge.target);
            }

            if let Some(sources) = self.target_to_sources.get_mut(&edge.target) {
                sources.remove(&edge.source);
            }
        }

        if let Some(edges) = self.source_to_edges.get_mut(&edge.source) {
            edges.remove(&edge.id);
        }

        if let Some(edges) = self.targets_to_edges.get_mut(&edge.target) {
            edges.remove(&edge.id);
        }

        if let Some(edges) = self.endpoints_to_edges.get_mut(&(edge.source, edge.target)) {
            edges.remove(&edge.id);
        }
    }

    pub(crate) fn remove_node(&mut self, id: NodeId) -> HashSet<EdgeId> {
        self.source_to_targets.remove(&id);

        for nodes in self.source_to_targets.values_mut() {
            nodes.remove(&id);
        }

        self.target_to_sources.remove(&id);

        for nodes in self.target_to_sources.values_mut() {
            nodes.remove(&id);
        }

        let to_remove = self.source_to_edges.remove(&id).into_iter().flatten();
        let to_remove = to_remove.chain(self.targets_to_edges.remove(&id).into_iter().flatten());
        let to_remove: HashSet<_> = to_remove.collect();

        for edges in self.source_to_edges.values_mut() {
            edges.retain(|&id| !to_remove.contains(&id));
        }

        for edges in self.targets_to_edges.values_mut() {
            edges.retain(|&id| !to_remove.contains(&id));
        }

        for edges in self.endpoints_to_edges.values_mut() {
            edges.retain(|&id| !to_remove.contains(&id));
        }

        self.endpoints_to_edges
            .retain(|&(source, target), _| source != id && target != id);

        to_remove
    }

    fn clear(&mut self) {
        self.source_to_targets.clear();
        self.target_to_sources.clear();

        self.source_to_edges.clear();
        self.targets_to_edges.clear();

        self.endpoints_to_edges.clear();
    }

    fn refresh<E>(&mut self, edges: &Slab<EdgeId, Edge<E>>) {
        self.clear();

        for edge in edges.iter() {
            self.update(edge);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct NodeClosures {
    nodes: HashMap<NodeId, NodeClosure>,
    externals: HashSet<NodeId>,
}

impl NodeClosures {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            externals: HashSet::new(),
        }
    }

    pub(crate) fn reserve(&mut self, additional: usize) {
        self.nodes.reserve(additional);
        self.externals.reserve(additional);
    }

    pub(crate) fn shrink_to_fit(&mut self) {
        self.nodes.shrink_to_fit();
        self.externals.shrink_to_fit();
    }

    pub(crate) fn get(&self, id: NodeId) -> Option<&NodeClosure> {
        self.nodes.get(&id)
    }

    pub(crate) const fn externals(&self) -> &HashSet<NodeId> {
        &self.externals
    }

    fn get_or_insert(&mut self, id: NodeId) -> &mut NodeClosure {
        self.nodes.entry(id).or_insert_with(NodeClosure::new)
    }

    fn update(&mut self, id: NodeId, closure: &EdgeClosures) {
        let node = self.get_or_insert(id);
        node.refresh(id, closure);

        if node.neighbours().is_empty() {
            self.externals.insert(id);
        } else {
            self.externals.remove(&id);
        }
    }

    pub(crate) fn remove(&mut self, id: NodeId, edges: &HashSet<EdgeId>) {
        self.nodes.remove(&id);
        self.externals.remove(&id);

        for node in &mut self.nodes.values_mut() {
            node.outgoing_neighbours.remove(&id);
            node.incoming_neighbours.remove(&id);
            node.neighbours.remove(&id);

            node.outgoing_edges.retain(|&edge| !edges.contains(&edge));
            node.incoming_edges.retain(|&edge| !edges.contains(&edge));
            node.edges.retain(|&edge| !edges.contains(&edge));
        }
    }

    fn clear(&mut self) {
        self.nodes.clear();
    }

    fn refresh<N>(&mut self, nodes: &Slab<NodeId, Node<N>>, closure: &EdgeClosures) {
        for id in nodes.keys() {
            self.update(id, closure);
        }

        self.gc(nodes);
    }

    fn gc<N>(&mut self, nodes: &Slab<NodeId, Node<N>>) {
        let existing_nodes: HashSet<NodeId> = nodes.keys().collect();

        self.nodes.retain(|id, _| existing_nodes.contains(id));
        self.externals.retain(|id| existing_nodes.contains(id));
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Closures {
    pub(crate) nodes: NodeClosures,
    pub(crate) edges: EdgeClosures,
}

impl Closures {
    pub(crate) fn new() -> Self {
        Self {
            nodes: NodeClosures::new(),
            edges: EdgeClosures::new(),
        }
    }

    pub(crate) fn update_node(&mut self, id: NodeId) {
        self.nodes.update(id, &self.edges);
    }

    pub(crate) fn update_edge<E>(&mut self, edge: &Edge<E>) {
        self.edges.update(edge);

        self.nodes.update(edge.source, &self.edges);
        self.nodes.update(edge.target, &self.edges);
    }

    /// Removes the node and all edges connected to it from the closure tables.
    ///
    /// Be aware that this does not remove the node from the graph itself.
    ///
    /// Only call this once you have removed the node (and all edges connected to it) from the
    /// graph.
    pub(crate) fn remove_node(&mut self, id: NodeId) {
        let edges = self.edges.remove_node(id);
        self.nodes.remove(id, &edges);
    }

    pub(crate) fn remove_edge<E>(&mut self, edge: &Edge<E>) {
        self.edges.remove(edge);

        self.nodes.update(edge.source, &self.edges);
        self.nodes.update(edge.target, &self.edges);
    }

    pub(crate) fn refresh<N, E>(
        &mut self,
        nodes: &Slab<NodeId, Node<N>>,
        edges: &Slab<EdgeId, Edge<E>>,
    ) {
        self.edges.refresh(edges);
        self.nodes.refresh(nodes, &self.edges);
    }

    pub(crate) fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }
}

#[cfg(test)]
mod tests {
    use core::iter::once;

    use hashbrown::{HashMap, HashSet};
    use petgraph_core::{attributes::Attributes, edge::marker::Directed};

    use crate::{
        closure::{EdgeClosures, NodeClosure},
        DinoGraph, EdgeId, NodeId,
    };

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

    #[test]
    fn single_node() {
        let mut graph = DinoGraph::<u8, u8, Directed>::new();

        let node = graph.insert_node(Attributes::new(1)).unwrap();
        let id = *node.id();

        let closures = &graph.storage().closures;

        assert_eq!(closures.nodes.nodes.len(), 1);
        assert_eq!(closures.nodes.externals, once(id).collect());

        assert_eq!(
            closures.nodes.nodes[&id],
            NodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: HashSet::new(),
                neighbours: HashSet::new(),
                outgoing_edges: HashSet::new(),
                incoming_edges: HashSet::new(),
                edges: HashSet::new(),
            }
        );

        assert_eq!(
            closures.edges,
            EdgeClosures {
                source_to_targets: HashMap::new(),
                target_to_sources: HashMap::new(),
                source_to_edges: HashMap::new(),
                targets_to_edges: HashMap::new(),
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

        assert_eq!(closures.nodes.nodes.len(), 2);
        assert_eq!(closures.nodes.externals, [a, b].into_iter().collect());

        assert_eq!(
            closures.nodes.nodes[&a],
            NodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: HashSet::new(),
                neighbours: HashSet::new(),
                outgoing_edges: HashSet::new(),
                incoming_edges: HashSet::new(),
                edges: HashSet::new(),
            }
        );

        assert_eq!(
            closures.nodes.nodes[&b],
            NodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: HashSet::new(),
                neighbours: HashSet::new(),
                outgoing_edges: HashSet::new(),
                incoming_edges: HashSet::new(),
                edges: HashSet::new(),
            }
        );

        assert_eq!(
            closures.edges,
            EdgeClosures {
                source_to_targets: HashMap::new(),
                target_to_sources: HashMap::new(),
                source_to_edges: HashMap::new(),
                targets_to_edges: HashMap::new(),
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

        let edge = graph.insert_edge(1u8, a, b).unwrap();
        let edge = *edge.id();

        let closures = &graph.storage().closures;

        assert_eq!(closures.nodes.nodes.len(), 2);
        assert!(closures.nodes.externals.is_empty());

        assert_eq!(
            closures.nodes.nodes[&a],
            NodeClosure {
                outgoing_neighbours: once(b).collect(),
                incoming_neighbours: HashSet::new(),
                neighbours: once(b).collect(),
                outgoing_edges: once(edge).collect(),
                incoming_edges: HashSet::new(),
                edges: once(edge).collect(),
            }
        );

        assert_eq!(
            closures.nodes.nodes[&b],
            NodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: once(a).collect(),
                neighbours: once(a).collect(),
                outgoing_edges: HashSet::new(),
                incoming_edges: once(edge).collect(),
                edges: once(edge).collect(),
            }
        );

        assert_eq!(
            closures.edges,
            EdgeClosures {
                source_to_targets: map! {
                    a => once(b).collect(),
                },
                target_to_sources: map! {
                    b => once(a).collect(),
                },
                source_to_edges: map! {
                    a => once(edge).collect(),
                },
                targets_to_edges: map! {
                    b => once(edge).collect(),
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

        let edge = graph.insert_edge(1u8, a, a).unwrap();
        let edge = *edge.id();

        let closures = &graph.storage().closures;

        assert_eq!(closures.nodes.nodes.len(), 1);
        assert!(closures.nodes.externals.is_empty());

        assert_eq!(
            closures.nodes.nodes[&a],
            NodeClosure {
                outgoing_neighbours: once(a).collect(),
                incoming_neighbours: once(a).collect(),
                neighbours: once(a).collect(),
                outgoing_edges: once(edge).collect(),
                incoming_edges: once(edge).collect(),
                edges: once(edge).collect(),
            }
        );

        assert_eq!(
            closures.edges,
            EdgeClosures {
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

            let ab = graph.insert_edge(1u8, a, b).unwrap();
            let ab = *ab.id();

            let bc = graph.insert_edge(1u8, b, c).unwrap();
            let bc = *bc.id();

            let ca = graph.insert_edge(1u8, c, a).unwrap();
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

            assert_eq!(closures.nodes.nodes.len(), 3);
            assert!(closures.nodes.externals.is_empty());

            assert_eq!(
                closures.nodes.nodes[&a],
                NodeClosure {
                    outgoing_neighbours: once(b).collect(),
                    incoming_neighbours: once(c).collect(),
                    neighbours: [b, c].into_iter().collect(),
                    outgoing_edges: once(ab).collect(),
                    incoming_edges: once(ca).collect(),
                    edges: [ab, ca].into_iter().collect(),
                }
            );

            assert_eq!(
                closures.nodes.nodes[&b],
                NodeClosure {
                    outgoing_neighbours: once(c).collect(),
                    incoming_neighbours: once(a).collect(),
                    neighbours: [c, a].into_iter().collect(),
                    outgoing_edges: once(bc).collect(),
                    incoming_edges: once(ab).collect(),
                    edges: [bc, ab].into_iter().collect(),
                }
            );

            assert_eq!(
                closures.nodes.nodes[&c],
                NodeClosure {
                    outgoing_neighbours: once(a).collect(),
                    incoming_neighbours: once(b).collect(),
                    neighbours: [a, b].into_iter().collect(),
                    outgoing_edges: once(ca).collect(),
                    incoming_edges: once(bc).collect(),
                    edges: [ca, bc].into_iter().collect(),
                }
            );

            assert_eq!(
                closures.edges,
                EdgeClosures {
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

        let ab1 = graph.insert_edge(1u8, a, b).unwrap();
        let ab1 = *ab1.id();

        let ab2 = graph.insert_edge(1u8, a, b).unwrap();
        let ab2 = *ab2.id();

        let closures = &graph.storage().closures;

        assert_eq!(closures.nodes.nodes.len(), 2);
        assert!(closures.nodes.externals.is_empty());

        assert_eq!(
            closures.nodes.nodes[&a],
            NodeClosure {
                outgoing_neighbours: once(b).collect(),
                incoming_neighbours: HashSet::new(),
                neighbours: once(b).collect(),
                outgoing_edges: [ab1, ab2].into_iter().collect(),
                incoming_edges: HashSet::new(),
                edges: [ab1, ab2].into_iter().collect(),
            }
        );

        assert_eq!(
            closures.nodes.nodes[&b],
            NodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: once(a).collect(),
                neighbours: once(a).collect(),
                outgoing_edges: HashSet::new(),
                incoming_edges: [ab1, ab2].into_iter().collect(),
                edges: [ab1, ab2].into_iter().collect(),
            }
        );

        assert_eq!(
            closures.edges,
            EdgeClosures {
                source_to_targets: map! {
                    a => once(b).collect(),
                },
                target_to_sources: map! {
                    b => once(a).collect(),
                },
                source_to_edges: map! {
                    a => [ab1, ab2].into_iter().collect(),
                },
                targets_to_edges: map! {
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

        assert_eq!(closures.nodes.nodes.len(), 2);
        assert!(closures.nodes.externals.is_empty());

        assert_eq!(
            closures.nodes.nodes[&a],
            NodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: once(c).collect(),
                neighbours: once(c).collect(),
                outgoing_edges: HashSet::new(),
                incoming_edges: once(ca).collect(),
                edges: once(ca).collect(),
            }
        );

        assert_eq!(
            closures.nodes.nodes[&c],
            NodeClosure {
                outgoing_neighbours: once(a).collect(),
                incoming_neighbours: HashSet::new(),
                neighbours: once(a).collect(),
                outgoing_edges: once(ca).collect(),
                incoming_edges: HashSet::new(),
                edges: once(ca).collect(),
            }
        );

        assert_eq!(
            closures.edges,
            EdgeClosures {
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

        assert_eq!(closures.nodes.nodes.len(), 3);
        assert!(closures.nodes.externals.is_empty());

        assert_eq!(
            closures.nodes.nodes[&a],
            NodeClosure {
                outgoing_neighbours: once(b).collect(),
                incoming_neighbours: once(c).collect(),
                neighbours: [b, c].into_iter().collect(),
                outgoing_edges: once(ab).collect(),
                incoming_edges: once(ca).collect(),
                edges: [ab, ca].into_iter().collect(),
            }
        );

        assert_eq!(
            closures.nodes.nodes[&b],
            NodeClosure {
                outgoing_neighbours: HashSet::new(),
                incoming_neighbours: once(a).collect(),
                neighbours: once(a).collect(),
                outgoing_edges: HashSet::new(),
                incoming_edges: once(ab).collect(),
                edges: once(ab).collect(),
            }
        );

        assert_eq!(
            closures.nodes.nodes[&c],
            NodeClosure {
                outgoing_neighbours: once(a).collect(),
                incoming_neighbours: HashSet::new(),
                neighbours: once(a).collect(),
                outgoing_edges: once(ca).collect(),
                incoming_edges: HashSet::new(),
                edges: once(ca).collect(),
            }
        );

        assert_eq!(
            closures.edges,
            EdgeClosures {
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
                    (b, c) => HashSet::new(),
                    (c, a) => once(ca).collect(),
                },
            }
        );
    }
}
