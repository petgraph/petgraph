use core::{fmt, hash::Hash};

/// The default integer type for graph indices.
/// `u32` is the default to reduce the size of the graph's data and improve
/// performance in the common case.
///
/// Used for node and edge indices in `Graph` and `StableGraph`, used
/// for node indices in `Csr`.
pub type DefaultIx = u32;

/// Trait for the unsigned integer type used for node and edge indices.
///
/// Marked `unsafe` because: the trait must faithfully preserve
/// and convert index values.
#[deprecated(note = "Use `funty::Unsigned` instead", since = "0.1.0")]
pub unsafe trait IndexType: Copy + Default + Hash + Ord + fmt::Debug + 'static {
    fn new(x: usize) -> Self;
    fn index(&self) -> usize;
    fn max() -> Self;
}

unsafe impl IndexType for usize {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x
    }

    #[inline(always)]
    fn index(&self) -> Self {
        *self
    }

    #[inline(always)]
    fn max() -> Self {
        ::std::usize::MAX
    }
}

unsafe impl IndexType for u32 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x as u32
    }

    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    fn max() -> Self {
        ::std::u32::MAX
    }
}

unsafe impl IndexType for u16 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x as u16
    }

    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    fn max() -> Self {
        ::std::u16::MAX
    }
}

unsafe impl IndexType for u8 {
    #[inline(always)]
    fn new(x: usize) -> Self {
        x as u8
    }

    #[inline(always)]
    fn index(&self) -> usize {
        *self as usize
    }

    #[inline(always)]
    fn max() -> Self {
        ::std::u8::MAX
    }
}
