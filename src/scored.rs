
/// Hold a score and a scored object in a pair for use with a BinaryHeap.
///
/// MinScored compares in reverse order compared to the score, so that we can
/// use BinaryHeap as a "min-heap" to extract the score, value pair with the
/// lowest score.
///
/// NOTE: MinScored implements a total order (Eq + Ord), so that it is possible
/// to use float types as scores.
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
        // Order NaN first, (NaN is equal to itself and largest)
        let selfnan = self.0 != self.0;
        let othernan = other.0 != other.0;
        if selfnan && othernan {
            Equal
        } else if selfnan {
            Less
        } else if othernan {
            Greater
        // Then order in reverse order
        } else if self.0 < other.0 {
            Greater
        } else if self.0 > other.0 {
            Less
        } else {
            Equal
        }
    }
}

