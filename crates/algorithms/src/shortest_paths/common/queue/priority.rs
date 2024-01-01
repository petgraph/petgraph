use alloc::collections::BinaryHeap;
use core::cmp::Ordering;

use petgraph_core::{
    node::NodeId,
    storage::{
        auxiliary::{BooleanGraphStorage, FrequencyHint, Hints, OccupancyHint, PerformanceHint},
        AuxiliaryGraphStorage,
    },
    GraphStorage,
};

pub(in crate::shortest_paths) struct PriorityQueueItem<T> {
    pub(in crate::shortest_paths) node: NodeId,

    pub(in crate::shortest_paths) priority: T,
}

impl<T> PartialEq for PriorityQueueItem<T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        other.priority.eq(&self.priority)
    }
}

impl<T> Eq for PriorityQueueItem<T> where T: Eq {}

impl<T> PartialOrd for PriorityQueueItem<T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.priority.partial_cmp(&self.priority)
    }
}

impl<T> Ord for PriorityQueueItem<T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

pub(in crate::shortest_paths) struct PriorityQueue<'graph, S, T>
where
    S: AuxiliaryGraphStorage + 'graph,
    T: Ord,
{
    heap: BinaryHeap<PriorityQueueItem<T>>,
    pub(crate) check_admissibility: bool,

    flags: S::BooleanNodeStorage<'graph>,
}

impl<'graph, S, T> PriorityQueue<'graph, S, T>
where
    S: AuxiliaryGraphStorage + 'graph,
    T: Ord,
{
    #[inline]
    pub(in crate::shortest_paths) fn new(storage: &'graph S) -> Self {
        Self {
            heap: BinaryHeap::new(),
            check_admissibility: true,
            flags: storage.boolean_node_storage(Hints {
                performance: PerformanceHint {
                    read: FrequencyHint::Frequent,
                    write: FrequencyHint::Infrequent,
                },
                occupancy: OccupancyHint::Dense,
            }),
        }
    }

    pub(in crate::shortest_paths) fn push(&mut self, node: NodeId, priority: T) {
        self.heap.push(PriorityQueueItem { node, priority });
    }

    pub(in crate::shortest_paths) fn visit(&mut self, id: NodeId) {
        self.flags.set(id, true);
    }

    #[inline]
    pub(in crate::shortest_paths) fn has_been_visited(&self, id: NodeId) -> bool {
        self.flags.get(id).unwrap_or(false)
    }

    #[inline]
    pub(in crate::shortest_paths) fn decrease_priority(&mut self, node: NodeId, priority: T) {
        if self.check_admissibility && self.has_been_visited(node) {
            return;
        }

        self.heap.push(PriorityQueueItem { node, priority });
    }

    #[inline]
    pub(in crate::shortest_paths) fn pop_min(&mut self) -> Option<PriorityQueueItem<T>> {
        loop {
            let item = self.heap.pop()?;

            if self.check_admissibility && self.has_been_visited(item.node) {
                continue;
            }

            self.visit(item.node);
            return Some(item);
        }
    }
}
