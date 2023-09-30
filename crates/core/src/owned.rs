use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

#[derive(Debug, Copy, Clone)]
pub enum MaybeOwned<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

impl<'a, T> From<T> for MaybeOwned<'a, T> {
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'a, T> From<&'a T> for MaybeOwned<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Borrowed(value)
    }
}

impl<'a, T> MaybeOwned<'a, T> {
    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(value) => Some(value),
        }
    }

    pub fn into_owned(self) -> T
    where
        T: Clone,
    {
        match self {
            Self::Borrowed(value) => value.clone(),
            Self::Owned(value) => value,
        }
    }
}

impl<T> AsRef<T> for MaybeOwned<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Borrowed(value) => value,
            Self::Owned(value) => value,
        }
    }
}

impl<T> PartialEq<T> for MaybeOwned<'_, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.as_ref().eq(other)
    }
}

impl<T> PartialEq for MaybeOwned<'_, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<T> Eq for MaybeOwned<'_, T> where T: Eq {}

impl<T> PartialOrd<T> for MaybeOwned<'_, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl<T> PartialOrd for MaybeOwned<'_, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T> Ord for MaybeOwned<'_, T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T> Hash for MaybeOwned<'_, T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state)
    }
}
