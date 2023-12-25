use core::{
    cmp::Ordering,
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
};

/// A borrowed or owned value.
///
/// This is analogous to [`Cow`], but without the additional [`ToOwned`] trait requirement which
/// makes it unsuitable for `no_std` environments.
///
/// # Name
///
/// The name `Moo` is a play on `Cow`, and is also a reference to the fact that cows moo.
///
/// [`Cow`]: alloc::borrow::Cow
pub enum Moo<'a, T> {
    Borrowed(&'a T),
    Owned(T),
}

impl<T> Display for Moo<'_, T>
where
    T: Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        Display::fmt(self.as_ref(), f)
    }
}

impl<'a, T> From<T> for Moo<'a, T> {
    fn from(value: T) -> Self {
        Self::Owned(value)
    }
}

impl<'a, T> From<&'a T> for Moo<'a, T> {
    fn from(value: &'a T) -> Self {
        Self::Borrowed(value)
    }
}

impl<'a, T> Moo<'a, T> {
    /// Returns a mutable reference to the owned value, if any.
    ///
    /// # Example
    ///
    /// ```
    /// use numi::borrow::Moo;
    ///
    /// let mut value = Moo::Owned(42);
    /// assert_eq!(value.as_mut(), Some(&mut 42));
    ///
    /// let mut value = Moo::Borrowed(&42);
    /// assert_eq!(value.as_mut(), None);
    /// ```
    pub fn as_mut(&mut self) -> Option<&mut T> {
        match self {
            Self::Borrowed(_) => None,
            Self::Owned(value) => Some(value),
        }
    }

    /// Converts the `Moo` into an owned value.
    ///
    /// This is a no-op if the value is already owned, and requires cloning if it is borrowed.
    ///
    /// # Example
    ///
    /// ```
    /// use numi::borrow::Moo;
    ///
    /// let value = Moo::Owned(42);
    /// assert_eq!(value.into_owned(), 42);
    ///
    /// let value = Moo::Borrowed(&42);
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

impl<T> AsRef<T> for Moo<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            Self::Borrowed(value) => value,
            Self::Owned(value) => value,
        }
    }
}

impl<T> PartialEq<T> for Moo<'_, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &T) -> bool {
        self.as_ref().eq(other)
    }
}

impl<T> PartialEq for Moo<'_, T>
where
    T: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.as_ref().eq(other.as_ref())
    }
}

impl<T> Eq for Moo<'_, T> where T: Eq {}

impl<T> PartialOrd<T> for Moo<'_, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &T) -> Option<Ordering> {
        self.as_ref().partial_cmp(other)
    }
}

impl<T> PartialOrd for Moo<'_, T>
where
    T: PartialOrd,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.as_ref().partial_cmp(other.as_ref())
    }
}

impl<T> Ord for Moo<'_, T>
where
    T: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_ref().cmp(other.as_ref())
    }
}

impl<T> Hash for Moo<'_, T>
where
    T: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.as_ref().hash(state);
    }
}
