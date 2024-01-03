use core::hash::{BuildHasher, Hash, Hasher};

pub(crate) struct ValueHash<T> {
    value: u64,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> ValueHash<T> {
    pub(crate) fn new<S>(hash_builder: &S, value: &T) -> Self
    where
        T: Hash,
        S: BuildHasher,
    {
        let hash = {
            let mut hasher = hash_builder.build_hasher();
            value.hash(&mut hasher);
            hasher.finish()
        };

        Self {
            value: hash,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<T> PartialEq for ValueHash<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T> Eq for ValueHash<T> {}

impl<T> Hash for ValueHash<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl<T> Clone for ValueHash<T> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T> Copy for ValueHash<T> {}
