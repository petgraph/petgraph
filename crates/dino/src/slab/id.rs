use core::{
    fmt::{Debug, Display, Formatter},
    num::{NonZeroU8, NonZeroUsize},
};

use crate::slab::{generation::Generation, key::Key};

/// A `EntryId` is a single `usize`, encoding generation information in the top
/// 1/4 of the bits, and index information in the remaining bits. This table
/// shows the breakdown for supported target platforms:
///
/// | `target_pointer_width` | generation bits | index bits | unused bits |
/// |------------------------|-----------------|------------|-------------|
/// | 16                     | 4               | 12         | 0           |
/// | 32                     | 8               | 24         | 0           |
/// | 64                     | 8               | 24         | 32          |
///
/// Each time a lot is allocated, its generation is incremented. When retrieving
/// values using a `EntryId`, the generation is validated as a safe guard against
/// returning a value. Because the generation isn't significantly wide, the
/// generation can wrap and is not a perfect protection against stale data,
/// although the likelihood of improper access is greatly reduced.
///
/// These values are used as indices into a roaring bitmap, which has a limit of [`u32::MAX`]
/// entries. This means that the `EntryId` is limited to 32 bits as well.
///
/// `EntryId` has the following layout: `<unused> <generation> <index>`
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct EntryId(NonZeroUsize);

impl Debug for EntryId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("EntryId")
            .field("index", &self.index())
            .field("generation", &self.generation())
            .finish()
    }
}

impl Display for EntryId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.write_fmt(format_args!("{}g{}", self.index(), self.generation().get()))
    }
}

#[cfg(target_pointer_width = "16")]
type RawIndex = u16;

#[cfg(any(target_pointer_width = "32", target_pointer_width = "64"))]
type RawIndex = u32;

impl EntryId {
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) const GENERATION_MAX: u8 = (RawIndex::MAX >> Self::INDEX_BITS) as u8;
    pub(crate) const INDEX_BITS: u32 = RawIndex::BITS / 4 * 3;
    pub(crate) const INDEX_MASK: usize = 2_usize.pow(Self::INDEX_BITS) - 1;

    #[inline]
    #[cfg_attr(target_pointer_width = "64", allow(clippy::absurd_extreme_comparisons))]
    pub(crate) fn new(generation: Generation, index: usize) -> Option<Self> {
        let is_valid = generation.get() <= Self::GENERATION_MAX && index <= Self::INDEX_MASK;

        is_valid.then(|| {
            Self(
                NonZeroUsize::new((generation.get() as usize) << Self::INDEX_BITS | index)
                    .expect("generation is non-zero"),
            )
        })
    }

    pub(crate) fn new_unchecked(raw: u32) -> Self {
        Self(NonZeroUsize::new(raw as usize).expect("raw is zero"))
    }

    #[inline]
    #[must_use]
    pub(crate) const fn index(self) -> usize {
        self.0.get() & Self::INDEX_MASK
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) const fn raw(self) -> u32 {
        // we know that the index will never be larger than 32 bits
        self.0.get() as u32
    }

    #[inline]
    #[must_use]
    #[allow(clippy::cast_possible_truncation)]
    pub(crate) fn generation(self) -> Generation {
        Generation(
            NonZeroU8::new((self.0.get() >> Self::INDEX_BITS) as u8).expect("invalid generation"),
        )
    }
}

impl Key for EntryId {
    fn from_id(id: EntryId) -> Self {
        id
    }

    fn into_id(self) -> EntryId {
        self
    }
}
