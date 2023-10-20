use core::num::NonZeroU8;

use crate::slab::EntryId;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub(crate) struct Generation(pub(super) NonZeroU8);

impl Generation {
    pub(crate) const fn first() -> Self {
        Self(match NonZeroU8::new(1) {
            Some(n) => n,
            None => unreachable!(),
        })
    }

    #[cfg_attr(target_pointer_width = "64", allow(clippy::absurd_extreme_comparisons))]
    pub(crate) const fn next(self) -> Self {
        match self.0.checked_add(1) {
            Some(next) if next.get() <= EntryId::GENERATION_MAX => Self(next),
            _ => Self::first(),
        }
    }

    pub(crate) const fn get(self) -> u8 {
        self.0.get()
    }
}
