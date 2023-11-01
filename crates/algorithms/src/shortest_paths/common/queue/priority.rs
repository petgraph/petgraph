use alloc::collections::BinaryHeap;
use core::{
    cell::Cell,
    cmp::{Ordering, Reverse},
};

use petgraph_core::{
    id::{FlagStorage, FlaggableGraphId},
    GraphStorage, Node,
};

struct PriorityQueueItem<'a, S, T>
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
        self.priority.eq(&other.priority)
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
        self.priority.partial_cmp(&other.priority)
    }
}

impl<S, T> Ord for PriorityQueueItem<'_, S, T>
where
    S: GraphStorage,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

pub(in crate::shortest_paths) struct PriorityQueue<'a, S, T>
where
    S: GraphStorage,
    S::NodeId: FlaggableGraphId<S>,
    T: Ord,
{
    heap: BinaryHeap<Reverse<PriorityQueueItem<'a, S, T>>>,

    flags: <S::NodeId as FlaggableGraphId<S>>::Store<'a>,
}

impl<'a, S, T> PriorityQueue<'a, S, T>
where
    S: GraphStorage,
    S::NodeId: FlaggableGraphId<S>,
    T: Ord,
{
    pub(in crate::shortest_paths) fn new(storage: &'a S) -> Self {
        Self {
            heap: BinaryHeap::new(),
            flags: <S::NodeId as FlaggableGraphId<S>>::flag_store(storage),
        }
    }

    pub(in crate::shortest_paths) fn push(&mut self, node: Node<'a, S>, priority: T) {
        self.heap.push(Reverse(PriorityQueueItem { node, priority }));
    }

    pub(in crate::shortest_paths) fn has_been_visited(&self, id: &'a S::NodeId) -> bool {
        self.flags.index(id)
    }

    pub(in crate::shortest_paths) fn decrease_priority(&mut self, node: Node<'a, S>, priority: T) {
        self.heap.push(Reverse(PriorityQueueItem { node, priority }));
    }

    pub(in crate::shortest_paths) fn pop_min(&mut self) -> Option<QueueItem<'a, S, T>> {
        while let Some(Reverse(item)) = self.heap.pop() {
            let visited = self.flags.index(item.node.id());

            if !visited {
                self.flags.set(item.node.id(), true);

                return Some(item);
            }
        }

        None
    }
}
