use core::{fmt::Debug, ops::Add};

/// Associated data that can be used for measures (such as length).
pub trait Measure: Debug + PartialOrd + Add<Self, Output = Self> + Copy {}

impl<M> Measure for M where M: Debug + PartialOrd + Add<M, Output = M> + Copy {}

/// A type that has a zero value.
pub trait Zero {
    fn zero() -> Self;
}

macro_rules! impl_zero(
    ( $( $t:ty ),* )=> {
        $(
            impl Zero for $t {
                fn zero() -> Self {
                    <$t as Default>::default()
                }
            }
        )*
    }
);

impl_zero!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

/// A type that can only have non-negative values.
pub trait NonNegative {}

macro_rules! impl_non_negative(
    ( $( $t:ty ),* )=> {
        $(
            impl NonNegative for $t {
            }
        )*
    }
);

impl_non_negative!(u8, u16, u32, u64, u128, usize);

/// A type that has minimum and maximum values.
pub trait Bounded {
    fn min() -> Self;
    fn max() -> Self;
}

macro_rules! impl_bounded(
    ( $( $t:ty ),* )=> {
        $(
            impl Bounded for $t {
                fn min() -> Self {
                    <$t>::MIN
                }

                fn max() -> Self {
                    <$t>::MAX
                }
            }
        )*
    }
);

impl_bounded!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);

pub trait MinMax {
    fn min_cmp(&self, other: &Self) -> Self;
    fn max_cmp(&self, other: &Self) -> Self;
}

macro_rules! impl_min_max(
    ( $( $t:ty ),* )=> {
        $(
            impl MinMax for $t {
                fn min_cmp(&self, other: &Self) -> Self {
                    if self < other {
                        *self
                    } else {
                        *other
                    }
                }

                fn max_cmp(&self, other: &Self) -> Self {
                    if self > other {
                        *self
                    } else {
                        *other
                    }
                }
            }
        )*
    }
);

impl_min_max!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize, f32, f64
);
