use std::iter;

pub fn enumerate<I>(iterable: I) -> iter::Enumerate<I::IntoIter>
where
    I: IntoIterator,
{
    iterable.into_iter().enumerate()
}

pub fn zip<I, J>(i: I, j: J) -> iter::Zip<I::IntoIter, J::IntoIter>
where
    I: IntoIterator,
    J: IntoIterator,
{
    i.into_iter().zip(j)
}

/// Mutably index a `Vec` without invalidating extant references under stacked borrows.
#[inline]
pub fn index_mut_no_sb_invalidation<T>(vec: &mut Vec<T>, index: usize) -> &mut T {
    #[inline(never)]
    #[cold]
    fn index_len_fail(index: usize, len: usize) -> ! {
        panic!("index {} is out of range for Vec length {}", index, len);
    }
    // Note, `Vec::len` isn't explicitly guaranteed to preserve validity of existing pointers but I
    // don't see any particular reason why it would and there is at least a test in the standard
    // library that would notice if this changes.
    let len = vec.len();
    if index < len {
        let ptr = vec.as_mut_ptr();
        // SAFETY: This is in bounds.
        unsafe { &mut *ptr.add(index) }
    } else {
        index_len_fail(index, len)
    }
}
