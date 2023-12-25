use core::{
    iter::Sum,
    ops::{Add, Div},
};

use num_traits::{CheckedDiv, Zero};

use crate::shortest_paths::common::AddRef;

/// Automatically implemented trait for types that can be used as a measure in the Bellman-Ford
/// algorithm.
///
/// This trait is implemented for all types that implement the supertraits mentioned in the trait
/// definition.
/// These traits either originate from [`core`] or [`num_traits`].
/// Special attention must be paid to the [`AddRef`] trait, which is a proxy trait which is
/// implemented for types that implement: `&Self: Add<&Self, Output = Self>`.
///
/// # Example
///
/// ```rust
/// use core::num::{Wrapping, Saturating};
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
    Clone
    + PartialOrd
    + Add<Self, Output = Self>
    + AddRef<Self, Output = Self>
    + Div<Self, Output = Self>
    + for<'a> Sum<&'a Self>
    + TryFrom<usize>
    + Zero
{
}

impl<T> BellmanFordMeasure for T where
    T: Clone
        + PartialOrd
        + Add<Self, Output = Self>
        + AddRef<Self, Output = Self>
        + Div<Self, Output = Self>
        + for<'a> Sum<&'a Self>
        + TryFrom<usize>
        + Zero
{
}
