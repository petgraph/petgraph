//! Collection of utility types and traits for working with values that may be owned or borrowed.
use core::{
    cmp::Ordering,
    hash::{Hash, Hasher},
};

/// A type that may be owned or borrowed.
///
/// This is analogous to [`Cow`], but without the `ToOwned` trait requirement (and therefore without
/// the `alloc` requirement).
///
/// [`Cow`]: alloc::borrow::Cow
#[derive(Debug, Copy, Clone)]
pub enum MaybeOwned<'a, T> {
    /// A borrowed value.
    Borrowed(&'a T),
    /// An owned value.
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
    /// Returns a mutable reference to the owned value, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::base::MaybeOwned;
    ///
    /// let mut value = MaybeOwned::Owned(42);
    /// assert_eq!(value.as_mut(), Some(&mut 42));
    ///
    /// let mut value = MaybeOwned::Borrowed(&42);
    /// assert_eq!(value.as_mut(), None);
    /// ```
    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(value) => Some(value),
        }
    }

    /// Converts the `MaybeOwned` into an owned value.
    ///
    /// This is a no-op if the value is already owned, and requires cloning if it is borrowed.
    ///
    /// # Example
    ///
    /// ```
    /// use petgraph_core::base::MaybeOwned;
    ///
    /// let value = MaybeOwned::Owned(42);
    /// assert_eq!(value.into_owned(), 42);
    ///
    /// let value = MaybeOwned::Borrowed(&42);
    /// assert_eq!(value.into_owned(), 42);
    /// ```
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
        self.as_ref().hash(state);
    }
}
