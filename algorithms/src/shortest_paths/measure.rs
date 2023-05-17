use core::{
    fmt::Debug,
    num::FpCategory,
    ops::{Add, Sub},
};

use funty::{Floating, Integral, Numeric};

use crate::shortest_paths::total_ord::TotalOrd;

// TODO: Measure should be Numeric
/// Associated data that can be used for measures (such as length).
pub trait Measure: Numeric + TotalOrd {}

impl<M> Measure for M where M: Numeric + TotalOrd {}

/// A floating-point measure.
pub trait FloatMeasure: Measure + Floating {
    const NEG_ZERO: Self;
    const POS_ZERO: Self;
}

impl FloatMeasure for f32 {
    const NEG_ZERO: Self = -0.0;
    const POS_ZERO: Self = 0.0;
}

impl FloatMeasure for f64 {
    const NEG_ZERO: Self = -0.0;
    const POS_ZERO: Self = 0.0;
}

pub trait BoundedMeasure: Measure {
    const MAX: Self;
    const MIN: Self;

    fn checked_add(self, rhs: Self) -> Option<Self>;
}

impl<T> BoundedMeasure for T
where
    T: Measure + Integral,
{
    const MAX: Self = T::MAX;
    const MIN: Self = T::MIN;

    fn checked_add(self, rhs: Self) -> Option<Self> {
        <T as Integral>::checked_add(self, rhs)
    }
}

// We cannot blanket impl on `Floating` because we already have a blanket impl on `Integral`.
impl BoundedMeasure for f32 {
    const MAX: Self = f32::MAX;
    const MIN: Self = f32::MIN;

    fn checked_add(self, rhs: Self) -> Option<Self> {
        let value = self + rhs;

        value.is_finite().then_some(value)
    }
}

impl BoundedMeasure for f64 {
    const MAX: Self = f64::MAX;
    const MIN: Self = f64::MIN;

    fn checked_add(self, rhs: Self) -> Option<Self> {
        let value = self + rhs;

        value.is_finite().then_some(value)
    }
}
