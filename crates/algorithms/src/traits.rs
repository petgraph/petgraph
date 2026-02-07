use std::{fmt::Debug, ops::Add};

/// Associated data that can be used for measures (such as length).
pub trait Measure: Debug + PartialOrd + Add<Self, Output = Self> + Default + Clone + Copy {}

impl<M> Measure for M where M: Debug + PartialOrd + Add<M, Output = M> + Default + Clone + Copy {}

/// A type that has a zero value.
pub trait Zero {
    fn zero() -> Self;
}

macro_rules! impl_zero(
    ( $( $t:ident ),* )=> {
        $(
            impl Zero for $t {
                fn zero() -> Self {
                    0 as $t
                }
            }
        )*
    }
);

impl_zero!(u8, u16, u32, u64, u128, usize, f32, f64);

/// A type that has minimum and maximum values.
pub trait Bounded {
    fn min() -> Self;
    fn max() -> Self;
}

macro_rules! impl_bounded(
    ( $( $t:ident ),* )=> {
        $(
            impl Bounded for $t {
                fn min() -> Self {
                    std::$t::MIN
                }

                fn max() -> Self {
                    std::$t::MAX
                }
            }
        )*
    }
);

impl_bounded!(u8, u16, u32, u64, u128, usize, f32, f64);
