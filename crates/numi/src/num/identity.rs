//! Identity elements for numbers.

use core::num::{Saturating, Wrapping};

use crate::macros::all_the_numbers;

/// Additive identity.
///
/// # Laws
///
/// ```text
/// ∀ a ∈ Self: a + 0 = a
/// ∀ a ∈ Self: 0 + a = a
/// ```
///
/// # Note
///
/// This trait is different from [`num_traits::Zero`], as it doesn't have any supertraits and does
/// not require [`Sized`].
pub trait Zero {
    /// Returns the additive identity of `Self`, `0`.
    ///
    /// # Purity
    ///
    /// This function is pure.
    fn zero() -> Self;

    /// Returns `true` if `self` is equal to the additive identity.
    ///
    /// # Purity
    ///
    /// This function is pure.
    fn is_zero(&self) -> bool;
}

macro_rules! impl_zero {
    (@int $ty:ty) => {
        impl Zero for $ty {
            #[inline]
            fn zero() -> Self {
                0
            }

            #[inline]
            fn is_zero(&self) -> bool {
                *self == 0
            }
        }
    };

    (@float $ty:ty) => {
        impl Zero for $ty {
            #[inline]
            fn zero() -> Self {
                0.0
            }

            #[inline]
            fn is_zero(&self) -> bool {
                *self == 0.0
            }
        }
    };
}

all_the_numbers!(@typed impl_zero);

impl<T> Zero for Wrapping<T>
where
    T: Zero,
{
    #[inline]
    fn zero() -> Self {
        Self(T::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

impl<T> Zero for Saturating<T>
where
    T: Zero,
{
    #[inline]
    fn zero() -> Self {
        Self(T::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

#[cfg(feature = "ordered-float-impl")]
impl<T> Zero for ordered_float::OrderedFloat<T>
where
    T: Zero,
{
    #[inline]
    fn zero() -> Self {
        Self(T::zero())
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.0.is_zero()
    }
}

#[cfg(feature = "ordered-float-impl")]
impl<T> Zero for ordered_float::NotNan<T>
where
    T: Copy + Zero,
{
    #[inline]
    fn zero() -> Self {
        // SAFETY: `T::zero()` is always a valid `NotNan<T>`.
        unsafe { Self::new_unchecked(T::zero()) }
    }

    #[inline]
    fn is_zero(&self) -> bool {
        self.into_inner().is_zero()
    }
}

/// Multiplicative identity.
///
/// # Laws
///
/// ```text
/// ∀ a ∈ Self: a * 1 = a
/// ∀ a ∈ Self: 1 * a = a
/// ```
///
/// # Note
///
/// This trait is different from [`num_traits::One`], as it doesn't have any supertraits and does
/// not require [`Sized`].
pub trait One {
    /// Returns the multiplicative identity of `Self`, `1`.
    ///
    /// # Purity
    ///
    /// This function is pure.
    fn one() -> Self;

    /// Returns `true` if `self` is equal to the multiplicative identity.
    ///
    /// # Purity
    ///
    /// This function is pure.
    fn is_one(&self) -> bool;
}

macro_rules! impl_one {
    (@int $ty:ty) => {
        impl One for $ty {
            #[inline]
            fn one() -> Self {
                1
            }

            #[inline]
            fn is_one(&self) -> bool {
                *self == 1
            }
        }
    };

    (@float $ty:ty) => {
        impl One for $ty {
            #[inline]
            fn one() -> Self {
                1.0
            }

            #[inline]
            fn is_one(&self) -> bool {
                let difference = *self - 1.0;

                // abs isn't available in no_std
                difference < Self::EPSILON && difference > -Self::EPSILON
            }
        }
    };
}

all_the_numbers!(@typed impl_one);

impl<T> One for Wrapping<T>
where
    T: One,
{
    #[inline]
    fn one() -> Self {
        Self(T::one())
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

impl<T> One for Saturating<T>
where
    T: One,
{
    #[inline]
    fn one() -> Self {
        Self(T::one())
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

#[cfg(feature = "ordered-float-impl")]
impl<T> One for ordered_float::OrderedFloat<T>
where
    T: One,
{
    #[inline]
    fn one() -> Self {
        Self(T::one())
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.0.is_one()
    }
}

#[cfg(feature = "ordered-float-impl")]
impl<T> One for ordered_float::NotNan<T>
where
    T: Copy + One,
{
    #[inline]
    fn one() -> Self {
        // SAFETY: `T::one()` is always a valid `NotNan<T>`.
        unsafe { Self::new_unchecked(T::one()) }
    }

    #[inline]
    fn is_one(&self) -> bool {
        self.into_inner().is_one()
    }
}
