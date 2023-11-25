use alloc::collections::VecDeque;
use std::{
    iter::Sum,
    ops::{Add, Div},
};

use num_traits::{CheckedDiv, Zero};
use petgraph_core::{GraphStorage, Node};

struct DoubleEndedQueueItem<'graph, S, T>
where
    S: GraphStorage,
{
    node: Node<'graph, S>,

    priority: T,
}

impl<'graph, S, T> DoubleEndedQueueItem<'graph, S, T>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) fn node(&self) -> Node<'graph, S> {
        self.node
    }

    pub(in crate::shortest_paths) fn priority(&self) -> &T {
        &self.priority
    }

    pub(in crate::shortest_paths) fn into_parts(self) -> (Node<'graph, S>, T) {
        (self.node, self.priority)
    }
}

// Newtype for VecDeque<T> to avoid exposing the VecDeque type as we may decide to reimplement this.
pub(in crate::shortest_paths) struct DoubleEndedQueue<'graph, S, T>
where
    S: GraphStorage,
{
    queue: VecDeque<DoubleEndedQueueItem<'graph, S, T>>,
}

impl<'graph, S, T> DoubleEndedQueue<'graph, S, T>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) fn new() -> Self {
        Self {
            queue: VecDeque::new(),
        }
    }

    pub(in crate::shortest_paths) fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
        }
    }

    pub(in crate::shortest_paths) fn push_front(&mut self, node: Node<'graph, S>, priority: T) {
        self.queue
            .push_front(DoubleEndedQueueItem { node, priority });
    }

    pub(in crate::shortest_paths) fn push_back(&mut self, node: Node<'graph, S>, priority: T) {
        self.queue
            .push_back(DoubleEndedQueueItem { node, priority });
    }

    pub(in crate::shortest_paths) fn pop_front(&mut self) -> Option<DoubleEndedQueueItem<S, T>> {
        self.queue.pop_front()
    }

    pub(in crate::shortest_paths) fn pop_back(&mut self) -> Option<DoubleEndedQueueItem<S, T>> {
        self.queue.pop_back()
    }

    pub(in crate::shortest_paths) fn peek_front(&self) -> Option<&DoubleEndedQueueItem<S, T>> {
        self.queue.front()
    }

    pub(in crate::shortest_paths) fn peek_back(&self) -> Option<&DoubleEndedQueueItem<S, T>> {
        self.queue.back()
    }

    pub(in crate::shortest_paths) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub(in crate::shortest_paths) fn len(&self) -> usize {
        self.queue.len()
    }

    pub(in crate::shortest_paths) fn contains_node(&self, node: &S::NodeId) -> bool {
        let (front, back) = self.queue.as_slices();

        front.iter().any(|item| item.id() == node.id())
            || back.iter().any(|item| item.id() == node.id())
    }
}

impl<'graph, S, T> DoubleEndedQueue<'graph, S, T>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) fn average_priority(&self) -> Option<T>
    where
        T: CheckedDiv + for<'a> Sum<&'a T> + From<usize>,
        for<'a> &'a T: Add<Output = T>,
    {
        let (front, back) = self.queue.as_slices();

        let front_sum: T = front.iter().map(|item| &item.priority).sum();
        let back_sum: T = back.iter().map(|item| &item.priority).sum();

        let total_sum: T = &front_sum + &back_sum;

        if self.queue.is_empty() {
            return None;
        }

        let length: T = self.queue.len().into();

        total_sum.checked_div(&length)
    }
}
