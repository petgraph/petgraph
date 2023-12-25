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
pub trait Zero {
    fn zero() -> Self;

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

/// Multiplicative identity.
///
/// # Laws
///
/// ```text
/// ∀ a ∈ Self: a * 1 = a
/// ∀ a ∈ Self: 1 * a = a
/// ```
pub trait One {
    fn one() -> Self;

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
