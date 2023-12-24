use core::ops::Add;

// proxy trait to not need to carry around where clause.
/// Automatically implemented trait for adding a reference to a reference.
///
/// This trait is implemented for all types that implement `Add<&T, Output = T>` on `&T`.
///
/// This is a helper trait that shouldn't need to be implemented directly.
/// The reason for it's existence is due to limitations in the Rust trait system.
/// To have a similar bound on a trait like [`BellmanFordMeasure`] one would need to use
/// `where for<'a> &'a Self: Add<&'a Self, Output = Self>`, but that means that this where clause
/// would need to be repeated every time the trait is used.
///
/// This limitation is known as [`implied bounds`](https://github.com/rust-lang/rust/issues/44491).
pub trait AddRef<Rhs = Self> {
    type Output;

    fn add_ref(&self, rhs: &Rhs) -> Self::Output;
}

impl<T> AddRef for T
where
    for<'a> &'a T: Add<&'a T, Output = T>,
{
    type Output = T;

    fn add_ref(&self, rhs: &T) -> Self::Output {
        self + rhs
    }
}
