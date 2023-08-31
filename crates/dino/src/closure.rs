use alloc::vec::Vec;

// The closure tables have quite a bit of allocations (due to the nested nature of the data
// structure). Question is can we avoid them?
use hashbrown::{HashMap, HashSet};

use crate::{edge::Edge, node::Node, DinosaurStorage, EdgeId, NodeId};

pub(crate) struct NodeClosure {
    outgoing_neighbours: HashSet<NodeId>,
    incoming_neighbours: HashSet<NodeId>,

    neighbours: HashSet<NodeId>,

    outgoing_edges: Vec<EdgeId>,
    incoming_edges: Vec<EdgeId>,

    edges: Vec<EdgeId>,
}

impl NodeClosure {
    fn new() -> Self {
        Self {
            outgoing_neighbours: HashSet::new(),
            incoming_neighbours: HashSet::new(),

            neighbours: HashSet::new(),

            outgoing_edges: Vec::new(),
            incoming_edges: Vec::new(),

            edges: Vec::new(),
        }
    }

    pub(crate) fn outgoing_neighbours(&self) -> &HashSet<NodeId> {
        &self.outgoing_neighbours
    }

    pub(crate) fn incoming_neighbours(&self) -> &HashSet<NodeId> {
        &self.incoming_neighbours
    }

    pub(crate) fn neighbours(&self) -> &HashSet<NodeId> {
        &self.neighbours
    }

    pub(crate) fn outgoing_edges(&self) -> &[EdgeId] {
        &self.outgoing_edges
    }

    pub(crate) fn incoming_edges(&self) -> &[EdgeId] {
        &self.incoming_edges
    }

    pub(crate) fn edges(&self) -> &[EdgeId] {
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

pub(crate) struct EdgeClosures {
    source_to_targets: HashMap<NodeId, HashSet<NodeId>>,
    target_to_sources: HashMap<NodeId, HashSet<NodeId>>,

    source_to_edges: HashMap<NodeId, HashSet<EdgeId>>,
    targets_to_edges: HashMap<NodeId, HashSet<EdgeId>>,
}

impl EdgeClosures {
    fn new() -> Self {
        Self {
            source_to_targets: HashMap::new(),
            target_to_sources: HashMap::new(),

            source_to_edges: HashMap::new(),
            targets_to_edges: HashMap::new(),
        }
    }

    pub(crate) fn source_to_targets(&self) -> &HashMap<NodeId, HashSet<NodeId>> {
        &self.source_to_targets
    }

    pub(crate) fn target_to_sources(&self) -> &HashMap<NodeId, HashSet<NodeId>> {
        &self.target_to_sources
    }

    pub(crate) fn source_to_edges(&self) -> &HashMap<NodeId, HashSet<EdgeId>> {
        &self.source_to_edges
    }

    pub(crate) fn targets_to_edges(&self) -> &HashMap<NodeId, HashSet<EdgeId>> {
        &self.targets_to_edges
    }

    fn update<E>(&mut self, edge: &Edge<E>) {
        self.source_to_targets
            .entry(edge.source)
            .or_insert_with(HashSet::new)
            .insert(edge.target);

        self.target_to_sources
            .entry(edge.target)
            .or_insert_with(HashSet::new)
            .insert(edge.source);

        self.source_to_edges
            .entry(edge.source)
            .or_insert_with(HashSet::new)
            .insert(edge.id);

        self.targets_to_edges
            .entry(edge.target)
            .or_insert_with(HashSet::new)
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
    }

    fn clear(&mut self) {
        self.source_to_targets.clear();
        self.target_to_sources.clear();

        self.source_to_edges.clear();
        self.targets_to_edges.clear();
    }

    fn refresh<E>(&mut self, edges: &HashMap<EdgeId, Edge<E>>) {
        self.clear();

        for edge in edges.values() {
            self.update(edge);
        }
    }
}

pub(crate) struct NodeClosures {
    nodes: HashMap<NodeId, NodeClosure>,
}

impl NodeClosures {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub(crate) fn get(&self, id: NodeId) -> Option<&NodeClosure> {
        self.nodes.get(&id)
    }

    fn get_or_insert(&mut self, id: NodeId) -> &mut NodeClosure {
        self.nodes.entry(id).or_insert_with(NodeClosure::new)
    }

    fn update(&mut self, id: NodeId, closure: &EdgeClosures) {
        self.get_or_insert(id).refresh(id, closure);
    }

    fn remove(&mut self, id: NodeId) {
        self.nodes.remove(&id);
    }

    fn clear(&mut self) {
        self.nodes.clear();
    }

    fn refresh<N>(&mut self, nodes: &HashMap<NodeId, Node<N>>, closure: &EdgeClosures) {
        for id in nodes.keys() {
            self.update(*id, closure);
        }

        self.gc(nodes);
    }

    fn gc<N>(&mut self, nodes: &HashMap<NodeId, Node<N>>) {
        let existing_nodes: HashSet<NodeId> = nodes.keys().copied().collect();

        self.nodes.retain(|id, _| existing_nodes.contains(id));
    }
}

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

    pub(crate) fn remove_node(&mut self, id: NodeId) {
        self.nodes.remove(id);
    }

    pub(crate) fn remove_edge<E>(&mut self, edge: &Edge<E>) {
        self.edges.remove(edge);

        self.nodes.update(edge.source, &self.edges);
        self.nodes.update(edge.target, &self.edges);
    }

    pub(crate) fn refresh<N, E>(
        &mut self,
        nodes: &HashMap<NodeId, Node<N>>,
        edges: &HashMap<EdgeId, Edge<E>>,
    ) {
        self.edges.refresh(edges);
        self.nodes.refresh(nodes, &self.edges);
    }

    pub(crate) fn clear(&mut self) {
        self.nodes.clear();
        self.edges.clear();
    }
}
