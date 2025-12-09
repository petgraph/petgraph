use core::{
    fmt::{self, Debug, Display},
    hash::Hash,
};

pub trait Id: Copy + PartialEq + Debug + Display {}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum IdError {
    OutOfRange { value: u64, range: (u64, u64) },
}

impl Display for IdError {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::OutOfRange {
                value,
                range: (min, max),
            } => {
                write!(fmt, "Value {value} is out of range [{min}..={max}]!")
            }
        }
    }
}

impl core::error::Error for IdError {}

pub trait LinearId:
    Id
    + PartialEq
    + Eq
    + PartialOrd
    + Ord
    + Hash
    + TryFrom<u16, Error = IdError>
    + TryFrom<u32, Error = IdError>
    + TryFrom<u64, Error = IdError>
    + TryFrom<usize, Error = IdError>
{
    const MIN: Self;
    const MAX: Self;

    #[inline]
    #[must_use]
    fn from_u16(index: u16) -> Self {
        Self::try_from(index).expect("Cannot create ID: value outside valid range")
    }

    #[inline]
    #[must_use]
    fn from_u32(index: u32) -> Self {
        Self::try_from(index).expect("Cannot create ID: value outside valid range")
    }

    #[inline]
    #[must_use]
    fn from_u64(index: u64) -> Self {
        Self::try_from(index).expect("Cannot create ID: value outside valid range")
    }

    #[inline]
    #[must_use]
    fn from_usize(index: usize) -> Self {
        Self::try_from(index).expect("Cannot create ID: value outside valid range")
    }

    fn as_u16(self) -> u16;
    fn as_u32(self) -> u32;
    fn as_u64(self) -> u64;
    fn as_usize(self) -> usize;

    #[inline]
    #[must_use = "Use `increment_by` to modify the id in place"]
    fn plus(self, amount: usize) -> Self {
        Self::from_usize(self.as_usize() + amount)
    }

    #[inline]
    #[must_use = "Use `decrement_by` to modify the id in place"]
    fn minus(self, amount: usize) -> Self {
        Self::from_usize(self.as_usize() - amount)
    }

    #[inline]
    fn increment_by(&mut self, amount: usize) {
        *self = self.plus(amount);
    }

    #[inline]
    fn decrement_by(&mut self, amount: usize) {
        *self = self.minus(amount);
    }
}
