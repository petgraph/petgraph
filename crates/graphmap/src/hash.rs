use core::hash::{BuildHasher, Hash, Hasher};

pub struct ValueHash<T> {
    value: u64,
    _phantom: core::marker::PhantomData<T>,
}

impl<T> ValueHash<T> {
    pub fn new<S>(hash_builder: &S, value: &T) -> Self
    where
        T: Hash + ?Sized,
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
        Self {
            value: self.value,
            _phantom: core::marker::PhantomData,
        }
    }
}

impl<T> Copy for ValueHash<T> {}
