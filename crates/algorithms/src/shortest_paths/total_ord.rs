use core::cmp::Ordering;

// We cannot blanket impl here because of the trait system, f32/f64 could in theory
// implement `Ord`, in that case the blanket impl would conflict with the impls.
pub trait TotalOrd {
    fn total_cmp(&self, other: &Self) -> Ordering;
}

macro_rules! impl_total_ord {
    ($($t:ty),*) => {
        $(
            impl TotalOrd for $t {
                fn total_cmp(&self, other: &Self) -> Ordering {
                    self.cmp(other)
                }
            }
        )*
    };
}

#[rustfmt::skip]
impl_total_ord!(
    (),
    u8, u16, u32, u64, u128, usize,
    i8, i16, i32, i64, i128, isize
);

impl TotalOrd for f32 {
    fn total_cmp(&self, other: &Self) -> Ordering {
        self.total_cmp(other)
    }
}

impl TotalOrd for f64 {
    fn total_cmp(&self, other: &Self) -> Ordering {
        self.total_cmp(other)
    }
}
