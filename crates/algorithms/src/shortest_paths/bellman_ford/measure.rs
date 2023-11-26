use core::{iter::Sum, ops::Add};

use num_traits::{CheckedDiv, Zero};

pub trait BellmanFordMeasure:
    PartialOrd
    + Clone
    + Add<Self, Output = Self>
    + for<'a> Add<&'a Self, Output = Self>
    + CheckedDiv
    + for<'a> Sum<&'a Self>
    + TryFrom<usize>
    + Zero
{
}

// TODO: &Self Add without where clause
impl<T> BellmanFordMeasure for T where
    T: PartialOrd
        + Clone
        + Add<Self, Output = Self>
        + for<'a> Add<&'a Self, Output = Self>
        + CheckedDiv
        + for<'a> Sum<&'a Self>
        + TryFrom<usize>
        + Zero
{
}
