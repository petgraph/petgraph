use alloc::collections::VecDeque;

// Newtype for VecDeque<T> to avoid exposing the VecDeque type as we may decide to reimplement this.
pub(in crate::shortest_paths) struct DoubleEndedQueue<T>(VecDeque<T>);

impl<T> DoubleEndedQueue<T>
where
    T: PartialEq,
{
    pub(in crate::shortest_paths) fn new() -> Self {
        Self(VecDeque::new())
    }

    pub(in crate::shortest_paths) fn with_capacity(capacity: usize) -> Self {
        Self(VecDeque::with_capacity(capacity))
    }

    pub(in crate::shortest_paths) fn push_front(&mut self, item: T) {
        self.0.push_front(item);
    }

    pub(in crate::shortest_paths) fn push_back(&mut self, item: T) {
        self.0.push_back(item);
    }

    pub(in crate::shortest_paths) fn pop_front(&mut self) -> Option<T> {
        self.0.pop_front()
    }

    pub(in crate::shortest_paths) fn pop_back(&mut self) -> Option<T> {
        self.0.pop_back()
    }

    pub(in crate::shortest_paths) fn front(&self) -> Option<&T> {
        self.0.front()
    }

    pub(in crate::shortest_paths) fn back(&self) -> Option<&T> {
        self.0.back()
    }

    pub(in crate::shortest_paths) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(in crate::shortest_paths) fn contains(&self, item: &T) -> bool {
        self.0.contains(item)
    }
}
