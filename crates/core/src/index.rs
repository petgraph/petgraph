use crate::storage::GraphStorage;

pub trait GraphIndex {
    type Value;
}

pub struct Arbitrary<T>(pub T);

impl<T> GraphIndex for Arbitrary<T> {
    type Value = T;
}

pub struct Managed<T>(T);

impl<T> Managed<T>
where
    T: ManagedGraphIndex,
{
    pub fn value(&self) -> &T {
        &self.0
    }

    pub fn into_value(self) -> T {
        self.0
    }

    pub fn unchecked_from_value(value: T) -> Self {
        Self(value)
    }

    pub fn next(storage: &T::Storage) -> Self {
        Self(T::next(storage))
    }
}

pub trait ManagedGraphIndex {
    type Storage: GraphStorage;

    fn next(storage: &Self::Storage) -> Self;
}

impl<T> GraphIndex for Managed<T>
where
    T: ManagedGraphIndex,
{
    type Value = T;
}
