#[cfg(target_pointer_width = "128")]
use funty::AtMost128 as AtMostUsize;
#[cfg(target_pointer_width = "16")]
use funty::AtMost16 as AtMostUsize;
#[cfg(target_pointer_width = "32")]
use funty::AtMost32 as AtMostUsize;
#[cfg(target_pointer_width = "64")]
use funty::AtMost64 as AtMostUsize;
use funty::Unsigned;

/// The default integer type for graph indices.
/// `u32` is the default to reduce the size of the graph's data and improve
/// performance in the common case.
///
/// Used for node and edge indices in `Graph` and `StableGraph`, used
/// for node indices in `Csr`.
#[cfg(any(
    target_pointer_width = "32",
    target_pointer_width = "64",
    target_pointer_width = "128"
))]
pub type DefaultIx = u32;

/// The default integer type for graph indices.
/// `u16` is the default on 16-bit platforms to reduce the size of the graph's data and improve
/// performance.
#[cfg(target_pointer_width = "16")]
pub type DefaultIx = u16;

/// Trait for the unsigned integer type used for node and edge indices.
///
/// ## Safety
///
/// There is a contractual obligation that [`from_usize`] is only called with values of the correct
/// size. Should this contract be violated, the implementation must panic.
///
/// [`from_usize`] must be the inverse of [`to_usize`].
///
/// The conversion from and to `usize` must be lossless with no implicit wrapping.
pub unsafe trait IndexType: Unsigned + AtMostUsize {
    #[deprecated(since = "0.1.0", note = "Use `Self::from_usize(x)` instead")]
    fn new(x: usize) -> Self;

    #[must_use]
    fn from_usize(value: usize) -> Self {
        Self::new(value)
    }

    #[deprecated(since = "0.1.0", note = "Use `Fundamental::to_usize` instead")]
    fn index(&self) -> usize;

    #[deprecated(since = "0.1.0", note = "Use `Integral::MAX` instead")]
    fn max() -> Self;
}

unsafe impl<T> IndexType for T
where
    T: Unsigned + AtMostUsize,
{
    fn new(x: usize) -> Self {
        Self::from_usize(x)
    }

    fn from_usize(value: usize) -> Self {
        // This will runtime panic if the contract has been violated.
        value.try_into().map_or_else(
            |_| {
                panic!("index out of range");
            },
            |value| value,
        )
    }

    fn index(&self) -> usize {
        self.as_usize()
    }

    fn max() -> Self {
        Self::MAX
    }
}
