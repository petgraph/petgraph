use num_traits::Zero;

use crate::shortest_paths::common::AddRef;

/// Automatically implemented trait for types that can be used as a measure in the Dijkstra
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
/// use core::num::Wrapping;
/// use core::num::Saturating;
/// use petgraph_algorithms::shortest_paths::dijkstra::DijkstraMeasure;
/// use static_assertions::assert_impl_all;
///
/// // Some examples of types that implement DijkstraMeasure
/// assert_impl_all!(u8: DijkstraMeasure);
/// assert_impl_all!(u16: DijkstraMeasure);
/// assert_impl_all!(u32: DijkstraMeasure);
/// assert_impl_all!(u64: DijkstraMeasure);
/// assert_impl_all!(u128: DijkstraMeasure);
/// assert_impl_all!(usize: DijkstraMeasure);
///
/// assert_impl_all!(i8: DijkstraMeasure);
/// assert_impl_all!(i16: DijkstraMeasure);
/// assert_impl_all!(i32: DijkstraMeasure);
/// assert_impl_all!(i64: DijkstraMeasure);
/// assert_impl_all!(i128: DijkstraMeasure);
/// assert_impl_all!(isize: DijkstraMeasure);
///
/// assert_impl_all!(f32: DijkstraMeasure);
/// assert_impl_all!(f64: DijkstraMeasure);
///
/// assert_impl_all!(Wrapping<u8>: DijkstraMeasure);
/// assert_impl_all!(Wrapping<u16>: DijkstraMeasure);
/// assert_impl_all!(Wrapping<u32>: DijkstraMeasure);
/// assert_impl_all!(Wrapping<u64>: DijkstraMeasure);
/// assert_impl_all!(Wrapping<u128>: DijkstraMeasure);
/// assert_impl_all!(Wrapping<usize>: DijkstraMeasure);
///
/// assert_impl_all!(Wrapping<i8>: DijkstraMeasure);
/// assert_impl_all!(Wrapping<i16>: DijkstraMeasure);
/// assert_impl_all!(Wrapping<i32>: DijkstraMeasure);
/// assert_impl_all!(Wrapping<i64>: DijkstraMeasure);
/// assert_impl_all!(Wrapping<i128>: DijkstraMeasure);
///
/// assert_impl_all!(Saturating<u8>: DijkstraMeasure);
/// assert_impl_all!(Saturating<u16>: DijkstraMeasure);
/// assert_impl_all!(Saturating<u32>: DijkstraMeasure);
/// assert_impl_all!(Saturating<u64>: DijkstraMeasure);
/// assert_impl_all!(Saturating<u128>: DijkstraMeasure);
/// assert_impl_all!(Saturating<usize>: DijkstraMeasure);
///
/// assert_impl_all!(Saturating<i8>: DijkstraMeasure);
/// assert_impl_all!(Saturating<i16>: DijkstraMeasure);
/// assert_impl_all!(Saturating<i32>: DijkstraMeasure);
/// assert_impl_all!(Saturating<i64>: DijkstraMeasure);
/// assert_impl_all!(Saturating<i128>: DijkstraMeasure);
/// assert_impl_all!(Saturating<isize>: DijkstraMeasure);
/// ```
pub trait DijkstraMeasure: Clone + PartialOrd + Ord + AddRef<Self, Output = Self> + Zero {}

impl<T> DijkstraMeasure for T where T: Clone + PartialOrd + Ord + AddRef<Self, Output = Self> + Zero {}
