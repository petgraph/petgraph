use core::{
    fmt::Debug,
    num::FpCategory,
    ops::{Add, Sub},
};

use crate::utilities::min_scored::TotalOrd;

/// Associated data that can be used for measures (such as length).
pub trait Measure: Debug + TotalOrd + Add<Self, Output = Self> + Default + Clone {}

impl<M> Measure for M where M: Debug + TotalOrd + Add<M, Output = M> + Default + Clone {}

/// A floating-point measure.
pub trait FloatMeasure: Measure + Copy {
    fn classify(self) -> FpCategory;
}

impl FloatMeasure for f32 {
    fn classify(self) -> FpCategory {
        self.classify()
    }
}

impl FloatMeasure for f64 {
    fn classify(self) -> FpCategory {
        self.classify()
    }
}

pub trait BoundedMeasure: Measure + Sub<Self, Output = Self> {
    fn min() -> Self;
    fn max() -> Self;

    fn overflowing_add(self, rhs: Self) -> (Self, bool);
}

macro_rules! impl_bounded_measure_integer(
    ( $( $t:ident ),* ) => {
        $(
            impl BoundedMeasure for $t {
                fn min() -> Self {
                    $t::MIN
                }

                fn max() -> Self {
                    $t::MAX
                }

                fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                    self.overflowing_add(rhs)
                }
            }
        )*
    };
);

impl_bounded_measure_integer!(
    u8, u16, u32, u64, u128, usize, i8, i16, i32, i64, i128, isize
);

macro_rules! impl_bounded_measure_float(
    ( $( $t:ident ),* ) => {
        $(
            impl BoundedMeasure for $t {
                fn min() -> Self {
                    $t::MIN
                }

                fn max() -> Self {
                    $t::MAX
                }

                fn overflowing_add(self, rhs: Self) -> (Self, bool) {
                    let value = self + rhs;
                    (value, !value.is_finite())
                }
            }
        )*
    };
);

impl_bounded_measure_float!(f32, f64);
