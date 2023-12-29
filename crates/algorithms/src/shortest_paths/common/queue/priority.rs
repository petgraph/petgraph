use alloc::collections::BinaryHeap;
use core::{
    cell::Cell,
    cmp::{Ordering, Reverse},
};

use petgraph_core::{GraphStorage, Node};

struct PriorityQueueItem<'a, S, T>
where
    S: GraphStorage,
{
    node: Node<'a, S>,

    priority: T,

    skip: Cell<bool>,
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
    T: Ord,
{
    heap: BinaryHeap<Reverse<PriorityQueueItem<'a, S, T>>>,
}

impl<'a, S, T> PriorityQueue<'a, S, T>
where
    S: GraphStorage,
    T: Ord,
{
    pub(in crate::shortest_paths) fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub(in crate::shortest_paths) fn push(&mut self, node: Node<'a, S>, priority: T) {
        self.heap.push(Reverse(PriorityQueueItem {
            node,
            priority,

            skip: Cell::new(false),
        }));
    }

    pub(in crate::shortest_paths) fn decrease_priority(&mut self, node: Node<'a, S>, priority: T) {
        for Reverse(item) in &self.heap {
            if item.node.id() == node.id() {
                item.skip.set(true);
                break;
            }
        }

        self.heap.push(Reverse(PriorityQueueItem {
            node,
            priority,

            skip: Cell::new(false),
        }));
    }

    pub(in crate::shortest_paths) fn pop_min(&mut self) -> Option<Node<'a, S>> {
        while let Some(Reverse(item)) = self.heap.pop() {
            if !item.skip.get() {
                return Some(item.node);
            }
        }

        None
    }
}
