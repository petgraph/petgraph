use num_traits::{CheckedAdd, Zero};

/// A trait for types that can be used as edge weights in the Floyd-Warshall algorithm.
///
/// This trait is automatically implemented for all types that implement the supertraits mentioned
/// in the trait definition.
///
/// These traits either originate from [`core`] or [`num_traits`].
///
/// # Example
///
/// ```rust
/// use core::num::Wrapping;
/// use core::num::Saturating;
/// use petgraph_algorithms::shortest_paths::floyd_warshall::FloydWarshallMeasure;
/// use static_assertions::assert_impl_all;
///
/// // Some examples of types that implement DijkstraMeasure
/// assert_impl_all!(u8: FloydWarshallMeasure);
/// assert_impl_all!(u16: FloydWarshallMeasure);
/// assert_impl_all!(u32: FloydWarshallMeasure);
/// assert_impl_all!(u64: FloydWarshallMeasure);
/// assert_impl_all!(u128: FloydWarshallMeasure);
/// assert_impl_all!(usize: FloydWarshallMeasure);
///
/// assert_impl_all!(i8: FloydWarshallMeasure);
/// assert_impl_all!(i16: FloydWarshallMeasure);
/// assert_impl_all!(i32: FloydWarshallMeasure);
/// assert_impl_all!(i64: FloydWarshallMeasure);
/// assert_impl_all!(i128: FloydWarshallMeasure);
/// assert_impl_all!(isize: FloydWarshallMeasure);
///
/// assert_impl_all!(f32: FloydWarshallMeasure);
/// assert_impl_all!(f64: FloydWarshallMeasure);
///
/// assert_impl_all!(Wrapping<u8>: FloydWarshallMeasure);
/// assert_impl_all!(Wrapping<u16>: FloydWarshallMeasure);
/// assert_impl_all!(Wrapping<u32>: FloydWarshallMeasure);
/// assert_impl_all!(Wrapping<u64>: FloydWarshallMeasure);
/// assert_impl_all!(Wrapping<u128>: FloydWarshallMeasure);
/// assert_impl_all!(Wrapping<usize>: FloydWarshallMeasure);
///
/// assert_impl_all!(Wrapping<i8>: FloydWarshallMeasure);
/// assert_impl_all!(Wrapping<i16>: FloydWarshallMeasure);
/// assert_impl_all!(Wrapping<i32>: FloydWarshallMeasure);
/// assert_impl_all!(Wrapping<i64>: FloydWarshallMeasure);
/// assert_impl_all!(Wrapping<i128>: FloydWarshallMeasure);
///
/// assert_impl_all!(Saturating<u8>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<u16>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<u32>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<u64>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<u128>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<usize>: FloydWarshallMeasure);
///
/// assert_impl_all!(Saturating<i8>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<i16>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<i32>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<i64>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<i128>: FloydWarshallMeasure);
/// assert_impl_all!(Saturating<isize>: FloydWarshallMeasure);
/// ```
pub trait FloydWarshallMeasure: Clone + PartialOrd + CheckedAdd + Zero {}

impl<T> FloydWarshallMeasure for T where T: Clone + PartialOrd + CheckedAdd + Zero {}
