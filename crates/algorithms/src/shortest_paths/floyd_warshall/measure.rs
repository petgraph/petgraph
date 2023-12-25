use numi::num::{checked::CheckedAdd, identity::Zero};

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
/// // see `CheckedAdd` why `Wrapping<T>` and `Saturating<T>` are not implemented
/// ```
pub trait FloydWarshallMeasure: Clone + PartialOrd + CheckedAdd + Zero {}

impl<T> FloydWarshallMeasure for T where T: Clone + PartialOrd + CheckedAdd + Zero {}
