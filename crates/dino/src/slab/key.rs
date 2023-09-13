use core::{fmt::Debug, hash::Hash};

use crate::slab::id::EntryId;

pub(crate) trait Key:
    Debug + Copy + Clone + PartialEq + Eq + PartialOrd + Ord + Hash
{
    fn from_id(id: EntryId) -> Self;
    fn into_id(self) -> EntryId;
}
