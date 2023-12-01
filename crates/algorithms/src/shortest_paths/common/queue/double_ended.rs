use alloc::collections::VecDeque;
use core::{
    hash::Hash,
    iter::Sum,
    ops::{Add, Div},
};

use fxhash::FxBuildHasher;
use hashbrown::HashSet;
use num_traits::{CheckedDiv, Zero};
use petgraph_core::{GraphStorage, Node};

pub(in crate::shortest_paths) struct DoubleEndedQueueItem<'graph, S, T>
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
    active: HashSet<&'graph S::NodeId, FxBuildHasher>,
}

impl<'graph, S, T> DoubleEndedQueue<'graph, S, T>
where
    S: GraphStorage,
    S::NodeId: Eq + Hash,
{
    pub(in crate::shortest_paths) fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            active: HashSet::with_hasher(FxBuildHasher::default()),
        }
    }

    pub(in crate::shortest_paths) fn with_capacity(capacity: usize) -> Self {
        Self {
            queue: VecDeque::with_capacity(capacity),
            active: HashSet::with_capacity_and_hasher(capacity, FxBuildHasher::default()),
        }
    }

    pub(in crate::shortest_paths) fn push_front(
        &mut self,
        node: Node<'graph, S>,
        priority: T,
    ) -> bool {
        if !self.active.insert(node.id()) {
            return false;
        }

        self.queue
            .push_front(DoubleEndedQueueItem { node, priority });
        true
    }

    pub(in crate::shortest_paths) fn push_back(
        &mut self,
        node: Node<'graph, S>,
        priority: T,
    ) -> bool {
        if !self.active.insert(node.id()) {
            return false;
        }

        self.queue
            .push_back(DoubleEndedQueueItem { node, priority });
        true
    }

    pub(in crate::shortest_paths) fn pop_front(
        &mut self,
    ) -> Option<DoubleEndedQueueItem<'graph, S, T>> {
        self.queue.pop_front()
    }

    pub(in crate::shortest_paths) fn pop_back(
        &mut self,
    ) -> Option<DoubleEndedQueueItem<'graph, S, T>> {
        self.queue.pop_back()
    }

    pub(in crate::shortest_paths) fn peek_front(
        &self,
    ) -> Option<&DoubleEndedQueueItem<'graph, S, T>> {
        self.queue.front()
    }

    pub(in crate::shortest_paths) fn peek_back(
        &self,
    ) -> Option<&DoubleEndedQueueItem<'graph, S, T>> {
        self.queue.back()
    }

    pub(in crate::shortest_paths) fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }

    pub(in crate::shortest_paths) fn len(&self) -> usize {
        self.queue.len()
    }

    pub(in crate::shortest_paths) fn contains_node(&self, node: &S::NodeId) -> bool {
        // let (front, back) = self.queue.as_slices();
        //
        // front.iter().any(|item| item.id() == node.id())
        //     || back.iter().any(|item| item.id() == node.id())

        self.active.contains(node)
    }
}

impl<'graph, S, T> DoubleEndedQueue<'graph, S, T>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) fn average_priority(&self) -> Option<T>
    where
        T: Zero + Div<Output = T> + Add<Output = T> + for<'a> Sum<&'a T> + TryFrom<usize>,
    {
        let (front, back) = self.queue.as_slices();

        let front_sum: T = front.iter().map(|item| &item.priority).sum();
        let back_sum: T = back.iter().map(|item| &item.priority).sum();

        let total_sum: T = front_sum + back_sum;

        if self.queue.is_empty() {
            return None;
        }

        let length: T = self.queue.len().try_into().ok()?;

        if length.is_zero() {
            return None;
        }

        Some(total_sum / length)
    }
}
