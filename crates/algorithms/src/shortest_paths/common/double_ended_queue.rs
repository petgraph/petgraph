// Newtype for VecDeque<T> to avoid exposing the VecDeque type as we may decide to reimplement this.
pub(in crate::shortest_paths) struct DoubleEndedQueue<T>(VecDequeue<T>);

impl DoubleEndedQueue<T> {
    pub(in crate::shortest_paths) fn new() -> Self {
        Self(VecDequeue::new())
    }

    pub(in crate::shortest_paths) fn with_capacity(capacity: usize) -> Self {
        Self(VecDequeue::with_capacity(capacity))
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

    pub(in crate::shortest_paths) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}
