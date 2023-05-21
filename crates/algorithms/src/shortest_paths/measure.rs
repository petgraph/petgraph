use funty::{Floating, Integral, Numeric};

use crate::shortest_paths::total_ord::TotalOrd;

/// Associated data that can be used for measures (such as length).
pub trait Measure: Numeric + TotalOrd {}

impl<M> Measure for M where M: Numeric + TotalOrd {}

/// A floating-point measure.
pub trait FloatMeasure: Measure + Floating {
    const NEG_ZERO: Self;
    const POS_ZERO: Self;
}

// for blanket implementation `From` needs to be const
impl FloatMeasure for f32 {
    const NEG_ZERO: Self = -0.0;
    const POS_ZERO: Self = 0.0;
}

impl FloatMeasure for f64 {
    const NEG_ZERO: Self = -0.0;
    const POS_ZERO: Self = 0.0;
}

// We cannot blanket impl here because of the trait system, f32/f64 could in theory
// implement `Measure + Integral`, in that case the blanket impl would conflict with the impls.
pub trait BoundedMeasure: Measure {
    const MAX: Self;
    const MIN: Self;

    fn checked_add(self, rhs: Self) -> Option<Self>;
}

macro_rules! impl_bounded_measure {
    ($($t:ty),*) => {
        $(
            impl BoundedMeasure for $t {
                const MAX: Self = <$t>::MAX;
                const MIN: Self = <$t>::MIN;

                fn checked_add(self, rhs: Self) -> Option<Self> {
                    <$t>::checked_add(self, rhs)
                }
            }
        )*
    };
}

impl_bounded_measure!(
    u8, u16, u32, u64, u128, usize, //
    i8, i16, i32, i64, i128, isize
);

impl BoundedMeasure for f32 {
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;

    fn checked_add(self, rhs: Self) -> Option<Self> {
        let value = self + rhs;

        value.is_finite().then_some(value)
    }
}

impl BoundedMeasure for f64 {
    const MAX: Self = Self::MAX;
    const MIN: Self = Self::MIN;

    fn checked_add(self, rhs: Self) -> Option<Self> {
        let value = self + rhs;

        value.is_finite().then_some(value)
    }
}
