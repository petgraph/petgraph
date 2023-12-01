use core::{
    iter::Sum,
    ops::{Add, Div},
};

use num_traits::{CheckedDiv, Zero};

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

/// Automatically implemented trait for types that can be used as a measure in the Bellman-Ford
/// algorithm.
///
/// This trait is implemented for all types that implement the supertraits mentioned in the trait
/// definition.
/// These traits either originate from [`core`] or [`num_traits`].
/// Special attention must be paid to the [`AddRef`] trait, which is a proxy trait which is
/// implemented for types that implement: `&Self: Add<&Self, Output = Self>`.
///
/// ```rust
/// use core::num::Wrapping;
/// use core::num::Saturating;
/// use petgraph_algorithms::shortest_paths::bellman_ford::BellmanFordMeasure;
/// use static_assertions::assert_impl_all;
///
/// // Some examples of types that implement BellmanFordMeasure
/// assert_impl_all!(u8: BellmanFordMeasure);
/// assert_impl_all!(u16: BellmanFordMeasure);
/// assert_impl_all!(u32: BellmanFordMeasure);
/// assert_impl_all!(u64: BellmanFordMeasure);
/// assert_impl_all!(u128: BellmanFordMeasure);
/// assert_impl_all!(usize: BellmanFordMeasure);
///
/// assert_impl_all!(i8: BellmanFordMeasure);
/// assert_impl_all!(i16: BellmanFordMeasure);
/// assert_impl_all!(i32: BellmanFordMeasure);
/// assert_impl_all!(i64: BellmanFordMeasure);
/// assert_impl_all!(i128: BellmanFordMeasure);
/// assert_impl_all!(isize: BellmanFordMeasure);
///
/// assert_impl_all!(f32: BellmanFordMeasure);
/// assert_impl_all!(f64: BellmanFordMeasure);
///
/// assert_impl_all!(Wrapping<u8>: BellmanFordMeasure);
/// assert_impl_all!(Wrapping<u16>: BellmanFordMeasure);
/// assert_impl_all!(Wrapping<u32>: BellmanFordMeasure);
/// assert_impl_all!(Wrapping<u64>: BellmanFordMeasure);
/// assert_impl_all!(Wrapping<u128>: BellmanFordMeasure);
/// assert_impl_all!(Wrapping<usize>: BellmanFordMeasure);
///
/// assert_impl_all!(Wrapping<i8>: BellmanFordMeasure);
/// assert_impl_all!(Wrapping<i16>: BellmanFordMeasure);
/// assert_impl_all!(Wrapping<i32>: BellmanFordMeasure);
/// assert_impl_all!(Wrapping<i64>: BellmanFordMeasure);
/// assert_impl_all!(Wrapping<i128>: BellmanFordMeasure);
///
/// assert_impl_all!(Saturating<u8>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<u16>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<u32>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<u64>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<u128>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<usize>: BellmanFordMeasure);
///
/// assert_impl_all!(Saturating<i8>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<i16>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<i32>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<i64>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<i128>: BellmanFordMeasure);
/// assert_impl_all!(Saturating<isize>: BellmanFordMeasure);
/// ```
pub trait BellmanFordMeasure:
    PartialOrd
    + Clone
    + Add<Self, Output = Self>
    + AddRef<Self, Output = Self>
    + Div<Self, Output = Self>
    + for<'a> Sum<&'a Self>
    + TryFrom<usize>
    + Zero
{
}

// TODO: &Self Add without where clause
impl<T> BellmanFordMeasure for T where
    T: PartialOrd
        + Clone
        + Add<Self, Output = Self>
        + AddRef<Self, Output = Self>
        + Div<Self, Output = Self>
        + for<'a> Sum<&'a Self>
        + TryFrom<usize>
        + Zero
{
}
