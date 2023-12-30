use alloc::collections::BinaryHeap;
use core::cmp::Ordering;

use petgraph_core::{
    id::{BooleanMapper, FlaggableGraphId},
    GraphStorage, Node,
};

pub(in crate::shortest_paths) struct PriorityQueueItem<'a, S, T>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) node: Node<'a, S>,

    pub(in crate::shortest_paths) priority: T,
}

impl<S, T> PartialEq for PriorityQueueItem<'_, S, T>
where
    S: GraphStorage,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        other.priority.eq(&self.priority)
    }
}

impl<S, T> Eq for PriorityQueueItem<'_, S, T>
where
    S: GraphStorage,
    T: Eq,
{
}

impl<S, T> PartialOrd for PriorityQueueItem<'_, S, T>
where
    S: GraphStorage,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.priority.partial_cmp(&self.priority)
    }
}

impl<S, T> Ord for PriorityQueueItem<'_, S, T>
where
    S: GraphStorage,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        other.priority.cmp(&self.priority)
    }
}

pub(in crate::shortest_paths) struct PriorityQueue<'a, S, T>
where
    S: GraphStorage,
    S::NodeId: FlaggableGraphId<S>,
    T: Ord,
{
    heap: BinaryHeap<PriorityQueueItem<'a, S, T>>,

    flags: <S::NodeId as FlaggableGraphId<S>>::Store<'a>,
}

impl<'a, S, T> PriorityQueue<'a, S, T>
where
    S: GraphStorage,
    S::NodeId: FlaggableGraphId<S>,
    T: Ord,
{
    #[inline]
    pub(in crate::shortest_paths) fn new(storage: &'a S) -> Self {
        Self {
            heap: BinaryHeap::new(),
            flags: <S::NodeId as FlaggableGraphId<S>>::flag_store(storage),
        }
    }

    pub(in crate::shortest_paths) fn push(&mut self, node: Node<'a, S>, priority: T) {
        self.heap.push(PriorityQueueItem { node, priority });
    }

    pub(in crate::shortest_paths) fn visit(&mut self, id: &'a S::NodeId) {
        self.flags.set(id, true);
    }

    #[inline]
    pub(in crate::shortest_paths) fn has_been_visited(&self, id: &'a S::NodeId) -> bool {
        self.flags.index(id)
    }

    #[inline]
    pub(in crate::shortest_paths) fn decrease_priority(&mut self, node: Node<'a, S>, priority: T) {
        if self.has_been_visited(node.id()) {
            return;
        }

        self.heap.push(PriorityQueueItem { node, priority });
    }

    #[inline]
    pub(in crate::shortest_paths) fn pop_min(&mut self) -> Option<PriorityQueueItem<'a, S, T>> {
        loop {
            let item = self.heap.pop()?;

            if self.has_been_visited(item.node.id()) {
                continue;
            }

            self.visit(item.node.id());
            return Some(item);
        }
    }
}
