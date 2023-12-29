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
        let Err(index) = self.0.binary_search(&node) else {
            return;
        };

        self.0.insert(index, node);
    }

    pub(crate) fn clear(&mut self) {
        self.0.clear();
    }

    pub(crate) fn iter(&self) -> slice::Iter<'_, T> {
        self.0.iter()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn contains(&self, node: &T) -> bool {
        self.0.binary_search(node).is_ok()
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

#[cfg(test)]
mod test {
    use alloc::vec;

    #[test]
    fn insert_sorted() {
        let mut vec = super::UniqueVec::new();

        vec.insert(1);
        vec.insert(2);
        vec.insert(3);
        vec.insert(4);
        vec.insert(5);

        assert_eq!(vec.0, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn insert_unsorted() {
        let mut vec = super::UniqueVec::new();

        vec.insert(5);
        vec.insert(4);
        vec.insert(3);
        vec.insert(2);
        vec.insert(1);

        assert_eq!(vec.0, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn insert_duplicates() {
        let mut vec = super::UniqueVec::new();

        vec.insert(1);
        vec.insert(1);
        vec.insert(1);
        vec.insert(1);
        vec.insert(1);

        assert_eq!(vec.0, vec![1]);
    }
}
