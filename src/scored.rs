
/// **MinScored\<K, T\>** holds a score **K** and a scored object **T** in
/// a pair for use with a **BinaryHeap**.
///
/// MinScored compares in reverse order by the score, so that we can
/// use BinaryHeap as a min-heap to extract the score-value pair with the
/// least score.
///
/// **Note:** MinScored implements a total order (**Ord**), so that it is possible
/// to use float types as scores.
#[deriving(Copy, Clone, Show)]
pub struct MinScored<K, T>(pub K, pub T);

impl<K: PartialEq, T> PartialEq for MinScored<K, T> {
    #[inline]
    fn eq(&self, other: &MinScored<K, T>) -> bool {
        self.0 == other.0
    }
}

impl<K: PartialEq, T> Eq for MinScored<K, T> {}

impl<K: PartialOrd, T> PartialOrd for MinScored<K, T> {
    #[inline]
    fn partial_cmp(&self, other: &MinScored<K, T>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<K: PartialOrd + PartialEq, T> Ord for MinScored<K, T> {
    #[inline]
    fn cmp(&self, other: &MinScored<K, T>) -> Ordering {
        let a = &self.0;
        let b = &other.0;
        if a == b {
            Equal
        } else if a < b {
            Greater
        } else if a > b {
            Less
        } else {
            // these are the NaN cases
            if a != a && b != b {
                Equal
            } else if a != a {
            // Order NaN less, so that it is last in the MinScore order
                Less
            } else {
                Greater
            }
        }
    }
}

