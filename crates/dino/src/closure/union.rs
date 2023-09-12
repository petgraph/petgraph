use roaring::RoaringBitmap;

pub(crate) struct UnionIterator<'a> {
    left: roaring::bitmap::Iter<'a>,
    left_next: Option<u32>,

    right: roaring::bitmap::Iter<'a>,
    right_next: Option<u32>,

    last: Option<u32>,
}

impl<'a> UnionIterator<'a> {
    pub(crate) fn new(left: &'a RoaringBitmap, right: &'a RoaringBitmap) -> Self {
        let left = left.iter();
        let right = right.iter();

        Self {
            left,
            left_next: None,
            right,
            right_next: None,
            last: None,
        }
    }
}

impl Iterator for UnionIterator<'_> {
    type Item = u32;

    fn next(&mut self) -> Option<Self::Item> {
        // a and b originate from `RoaringBitmap::iter`, which is guaranteed to be sorted, this
        // simplifies the logic needed here a lot.
        // We only want to return all unique elements from both iterators.

        // The algorithm is pretty simple.
        // 1) get the last element from each iterator, but only if it is larger than the last
        //    element we returned
        // 2) return the smaller of the two elements
        // 3) set the last element to the element we just returned
        const fn is_greater_than_or_equal(left: Option<u32>, right: Option<u32>) -> bool {
            match (left, right) {
                (Some(last), Some(next)) => last >= next,
                // `None` can occur if the last iteration chose the value of the right side,
                // therefore we continue.
                // `None` on the left side means, meaning we
                // can stop and take the value.
                (_, None) => true,
                (None, _) => false,
            }
        }

        let last = self.last.take();

        let mut left_next = self.left_next.take();
        let mut right_next = self.right_next.take();

        // Find a value that is larger than the last value we returned.
        // `last >= left_next`
        while is_greater_than_or_equal(last, left_next) {
            let Some(next) = self.left.next() else {
                left_next = None;
                break;
            };

            left_next = Some(next);
        }

        // Find a value that is larger than the last value we returned.
        // `last >= right_next`
        while is_greater_than_or_equal(last, right_next) {
            let Some(next) = self.right.next() else {
                right_next = None;
                break;
            };

            right_next = Some(next);
        }

        let next = match (left_next, right_next) {
            (Some(a), Some(b)) => {
                if a < b {
                    self.right_next = Some(b);
                    Some(a)
                } else {
                    self.left_next = Some(a);
                    Some(b)
                }
            }
            (Some(a), None) => Some(a),
            (None, Some(b)) => Some(b),
            (None, None) => None,
        };

        self.last = next;
        next
    }
}
