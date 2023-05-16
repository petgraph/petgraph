use core::cmp::Ordering;

pub trait TotalOrd {
    fn total_cmp(&self, other: &Self) -> Ordering;
}

impl<T> TotalOrd for T
where
    T: Ord,
{
    fn total_cmp(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }
}

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
