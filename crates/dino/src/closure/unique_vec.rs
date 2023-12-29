use alloc::vec::Vec;
use core::slice;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct UniqueVec<T>(Vec<T>);

impl<T> UniqueVec<T>
where
    T: Ord,
{
    pub(crate) const fn new() -> Self {
        Self(Vec::new())
    }

    pub(crate) fn insert(&mut self, node: T) {
        self.0.push(node);
        self.0.sort();
        self.0.dedup();
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    pub(crate) fn iter(&self) -> slice::Iter<'_, T> {
        self.0.iter()
    }

    pub(crate) fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> IntoIterator for UniqueVec<T> {
    type IntoIter = alloc::vec::IntoIter<T>;
    type Item = T;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}
