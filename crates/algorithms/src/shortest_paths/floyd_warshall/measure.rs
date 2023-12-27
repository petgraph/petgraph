use numi::num::{checked::CheckedAdd, identity::Zero};

/// A trait for types that can be used as edge weights in the Floyd-Warshall algorithm.
///
/// This trait is automatically implemented for all types that implement the supertraits mentioned
/// in the trait definition.
///
/// These traits either originate from [`core`] or [`numi`].
///
/// # Note on floating point types
///
/// This trait is not implemented for both [`f32`] and [`f64`], because they do not implement
/// [`CheckedAdd`] which is required for the Floyd-Warshall algorithm.
/// You can instead use [`ordered_float::NotNan`], which is a wrapper type that implements
/// [`CheckedAdd`].
///
/// The reason that [`f32`] and [`f64`] do not implement [`CheckedAdd`] is because they do not
/// have the concept of an overflow, which is required for [`CheckedAdd`].
/// If a value gets to large it will instead become [`f32::INFINITY`] or [`f64::INFINITY`].
///
/// # Example
///
/// ```rust
/// use core::num::Wrapping;
/// use core::num::Saturating;
/// use ordered_float::NotNan;
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
/// // f32 and f64 are not implemented because they do not implement CheckedAdd
/// // use `ordered_float::NotNan` instead.
/// assert_impl_all!(NotNan<f32>: FloydWarshallMeasure);
/// assert_impl_all!(NotNan<f64>: FloydWarshallMeasure);
///
/// // see `CheckedAdd` why `Wrapping<T>` and `Saturating<T>` are not implemented
/// ```
pub trait FloydWarshallMeasure: Clone + PartialOrd + CheckedAdd + Zero {}

impl<T> FloydWarshallMeasure for T where T: Clone + PartialOrd + CheckedAdd + Zero {}
