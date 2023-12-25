//! Checked arithmetic operations.
// TODO: more traits?
use crate::macros::all_the_numbers;

/// Addition that checks for overflow.
///
/// # Note
///
/// This trait is not implemented for [`Wrapping<T>`] and [`Saturating<T>`], this is on purpose, as
/// `checked_add` is defined as whenever an overflow happens, which is intentional for these types.
pub trait CheckedAdd: Sized {
    /// Adds two numbers, checking for overflow. If overflow happens, `None` is returned.
    ///
    /// # Purity
    ///
    /// This function is pure.
    fn checked_add(&self, rhs: &Self) -> Option<Self>;
}

macro_rules! impl_checked_add {
    (@int $ty:ty) => {
        impl CheckedAdd for $ty {
            #[inline]
            fn checked_add(&self, rhs: &Self) -> Option<Self> {
                <$ty>::checked_add(*self, *rhs)
            }
        }
    };

    (@float $ty:ty) => {};
}

all_the_numbers!(@typed impl_checked_add);

// Wrapping<T> and Saturating<T> are not supported, checked operations aren't a concept for them.

#[cfg(feature = "ordered-float-impl")]
impl<T> CheckedAdd for ordered_float::NotNan<T>
where
    T: ordered_float::FloatCore,
{
    fn checked_add(&self, rhs: &Self) -> Option<Self> {
        let lhs = self.into_inner();
        let rhs = rhs.into_inner();

        Self::new(lhs + rhs).ok()
    }
}
