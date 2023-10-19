use alloc::collections::BinaryHeap;
use core::{
    cell::Cell,
    cmp::{Ordering, Reverse},
};

use petgraph_core::{GraphStorage, Node};

struct QueueItem<'a, S, T>
where
    S: GraphStorage,
{
    node: Node<'a, S>,

    priority: T,

    skip: Cell<bool>,
}

impl<S, T> PartialEq for QueueItem<'_, S, T>
where
    S: GraphStorage,
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.priority.eq(&other.priority)
    }
}

impl<S, T> Eq for QueueItem<'_, S, T>
where
    S: GraphStorage,
    T: Eq,
{
}

impl<S, T> PartialOrd for QueueItem<'_, S, T>
where
    S: GraphStorage,
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.priority.partial_cmp(&other.priority)
    }
}

impl<S, T> Ord for QueueItem<'_, S, T>
where
    S: GraphStorage,
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.priority.cmp(&other.priority)
    }
}

pub(super) struct Queue<'a, S, T>
where
    S: GraphStorage,
    T: Ord,
{
    heap: BinaryHeap<Reverse<QueueItem<'a, S, T>>>,
}

impl<'a, S, T> Queue<'a, S, T>
where
    S: GraphStorage,
    T: Ord,
{
    pub(super) fn new() -> Self {
        Self {
            heap: BinaryHeap::new(),
        }
    }

    pub(super) fn push(&mut self, node: Node<'a, S>, priority: T) {
        self.heap.push(Reverse(QueueItem {
            node,
            priority,

            skip: Cell::new(false),
        }));
    }

    pub(super) fn decrease_priority(&mut self, node: Node<'a, S>, priority: T) {
        for Reverse(item) in &self.heap {
            if item.node.id() == node.id() {
                item.skip.set(true);
                break;
            }
        }

        self.heap.push(Reverse(QueueItem {
            node,
            priority,

            skip: Cell::new(false),
        }));
    }

    pub(super) fn pop_min(&mut self) -> Option<Node<'a, S>> {
        while let Some(Reverse(item)) = self.heap.pop() {
            if !item.skip.get() {
                return Some(item.node);
            }
        }

        None
    }
}
