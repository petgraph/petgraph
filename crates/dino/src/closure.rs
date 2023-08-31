use alloc::vec::Vec;

use hashbrown::{HashMap, HashSet};

use crate::{DinosaurStorage, EdgeId, NodeId};

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

    fn refresh<N, E>(&mut self, id: NodeId, closure: &EdgeClosures) {
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

        if let Some(target_to_edges) = closure.target_to_edges.get(&id) {
            self.incoming_edges.extend(target_to_edges.iter().copied());
            self.edges.extend(target_to_edges.iter().copied());
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

    fn update<N, E>(&mut self, id: EdgeId, storage: &DinosaurStorage<N, E>) {
        let Some(edge) = storage.get_edge(id) else {
            return;
        };

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
            .insert(id);

        self.targets_to_edges
            .entry(edge.target)
            .or_insert_with(HashSet::new)
            .insert(id);
    }

    fn remove<N, E>(&mut self, id: EdgeId, storage: &DinosaurStorage<N, E>) {
        let Some(edge) = storage.get_edge(id) else {
            return;
        };

        // lookup in the current closure if there are multiple edges with the same source and target
        let multi = 'multi: {
            let matching_source = self.source_to_edges.get(&edge.source);
            let matching_target = self.targets_to_edges.get(&edge.target);

            let (source, target) = match (matching_source, matching_target) {
                (Some(source), Some(target)) => (source, target),
                _ => break 'multi false,
            };

            // This avoids an additional allocation
            let same_source_and_target = source
                .iter()
                .any(|&edge_id| id != edge_id && target.contains(&edge_id));

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
            edges.remove(&id);
        }

        if let Some(edges) = self.targets_to_edges.get_mut(&edge.target) {
            edges.remove(&id);
        }
    }

    fn clear(&mut self) {
        self.source_to_targets.clear();
        self.target_to_sources.clear();

        self.source_to_edges.clear();
        self.targets_to_edges.clear();
    }

    fn refresh<N, E>(&mut self, storage: &DinosaurStorage<N, E>) {
        self.clear();

        for &id in storage.edges.keys() {
            self.update(id, storage);
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

    fn get_or_insert(&mut self, id: NodeId) -> &mut NodeClosure {
        self.nodes.entry(id).or_insert_with(NodeClosure::new)
    }

    pub(crate) fn update<N, E>(&mut self, id: NodeId, closure: &EdgeClosures) {
        self.get_or_insert(id).refresh(id, closure);
    }

    pub(crate) fn remove(&mut self, id: NodeId) {
        self.nodes.remove(&id);
    }

    fn refresh<N, E>(&mut self, storage: &DinosaurStorage<N, E>, closure: &EdgeClosures) {
        for id in storage.nodes.keys() {
            self.update(*id, closure);
        }

        self.gc(storage);
    }

    fn gc<N, E>(&mut self, storage: &DinosaurStorage<N, E>) {
        let existing_nodes: HashSet<NodeId> = storage.nodes.keys().copied().collect();

        self.nodes.retain(|id, _| existing_nodes.contains(id));
    }
}

pub(crate) struct Closures {
    nodes: NodeClosures,
    edges: EdgeClosures,
}

impl Closures {
    pub(crate) fn new() -> Self {
        Self {
            nodes: NodeClosures::new(),
            edges: EdgeClosures::new(),
        }
    }

    pub(crate) fn update_node<N, E>(&mut self, id: NodeId) {
        self.nodes.update(id, &self.edges);
    }

    pub(crate) fn update_edge<N, E>(&mut self, id: EdgeId, storage: &DinosaurStorage<N, E>) {
        self.edges.update(id, storage);

        let Some(edge) = storage.get_edge(id) else {
            return;
        };

        self.nodes.update(edge.source, &self.edges);
        self.nodes.update(edge.target, &self.edges);
    }

    pub(crate) fn remove_node(&mut self, id: NodeId) {
        self.nodes.remove(id);
    }

    pub(crate) fn remove_edge<N, E>(&mut self, id: EdgeId, storage: &DinosaurStorage<N, E>) {
        self.edges.remove(id, storage);

        let Some(edge) = storage.get_edge(id) else {
            return;
        };

        self.nodes.update(edge.source, &self.edges);
        self.nodes.update(edge.target, &self.edges);
    }

    pub(crate) fn refresh<N, E>(&mut self, storage: &DinosaurStorage<N, E>) {
        self.edges.refresh(storage);
        self.nodes.refresh(storage, &self.edges);
    }
}
