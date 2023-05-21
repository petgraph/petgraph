//! `UnionFind<K>` is a disjoint-set data structure.

use alloc::{vec, vec::Vec};
use core::cmp::Ordering;

use petgraph_core::index::IndexType;

/// `UnionFind<K>` is a disjoint-set data structure. It tracks set membership of *n* elements
/// indexed from *0* to *n - 1*. The scalar type is `K` which must be an unsigned integer type.
///
/// <http://en.wikipedia.org/wiki/Disjoint-set_data_structure>
///
/// Too awesome not to quote:
///
/// “The amortized time per operation is **O(α(n))** where **α(n)** is the
/// inverse of **f(x) = A(x, x)** with **A** being the extremely fast-growing Ackermann function.”
#[derive(Debug, Clone)]
pub(crate) struct UnionFind<K> {
    // For element at index *i*, store the index of its parent; the representative itself
    // stores its own index. This forms equivalence classes which are the disjoint sets, each
    // with a unique representative.
    parent: Vec<K>,
    // It is a balancing tree structure,
    // so the ranks are logarithmic in the size of the container -- a byte is more than enough.
    //
    // Rank is separated out both to save space and to save cache in when searching in the parent
    // vector.
    rank: Vec<u8>,
}

#[inline]
unsafe fn get_unchecked<K>(xs: &[K], index: usize) -> &K {
    debug_assert!(index < xs.len());
    xs.get_unchecked(index)
}

#[inline]
unsafe fn get_unchecked_mut<K>(xs: &mut [K], index: usize) -> &mut K {
    debug_assert!(index < xs.len());
    xs.get_unchecked_mut(index)
}

impl<K> UnionFind<K>
where
    K: IndexType,
{
    /// Create a new `UnionFind` of `n` disjoint sets.
    pub fn new(n: usize) -> Self {
        let rank = vec![0; n];
        let parent = (0..n).map(K::from_usize).collect::<Vec<K>>();

        UnionFind { parent, rank }
    }

    /// Return the representative for `x`.
    ///
    /// **Panics** if `x` is out of bounds.
    pub fn find(&self, x: K) -> K {
        assert!(x.index() < self.parent.len());
        unsafe {
            let mut x = x;
            loop {
                // Use unchecked indexing because we can trust the internal set ids.
                let xparent = *get_unchecked(&self.parent, x.index());
                if xparent == x {
                    break;
                }
                x = xparent;
            }
            x
        }
    }

    /// Return the representative for `x`.
    ///
    /// Write back the found representative, flattening the internal
    /// datastructure in the process and quicken future lookups.
    ///
    /// **Panics** if `x` is out of bounds.
    pub fn find_mut(&mut self, x: K) -> K {
        assert!(x.index() < self.parent.len());
        unsafe { self.find_mut_recursive(x) }
    }

    unsafe fn find_mut_recursive(&mut self, mut x: K) -> K {
        let mut parent = *get_unchecked(&self.parent, x.index());
        while parent != x {
            let grandparent = *get_unchecked(&self.parent, parent.index());
            *get_unchecked_mut(&mut self.parent, x.index()) = grandparent;
            x = parent;
            parent = grandparent;
        }
        x
    }

    /// Returns `true` if the given elements belong to the same set, and returns
    /// `false` otherwise.
    pub fn equiv(&self, x: K, y: K) -> bool {
        self.find(x) == self.find(y)
    }

    /// Unify the two sets containing `x` and `y`.
    ///
    /// Return `false` if the sets were already the same, `true` if they were unified.
    ///
    /// **Panics** if `x` or `y` is out of bounds.
    pub fn union(&mut self, x: K, y: K) -> bool {
        if x == y {
            return false;
        }
        let xrep = self.find_mut(x);
        let yrep = self.find_mut(y);

        if xrep == yrep {
            return false;
        }

        let xrepu = xrep.index();
        let yrepu = yrep.index();
        let xrank = self.rank[xrepu];
        let yrank = self.rank[yrepu];

        // The rank corresponds roughly to the depth of the treeset, so put the
        // smaller set below the larger
        match xrank.cmp(&yrank) {
            Ordering::Less => self.parent[xrepu] = yrep,
            Ordering::Greater => self.parent[yrepu] = xrep,
            Ordering::Equal => {
                self.parent[yrepu] = xrep;
                self.rank[xrepu] += 1;
            }
        }
        true
    }

    /// Return a vector mapping each element to its representative.
    pub fn into_labeling(mut self) -> Vec<K> {
        // write in the labeling of each element
        unsafe {
            for ix in 0..self.parent.len() {
                let k = *get_unchecked(&self.parent, ix);
                let xrep = self.find_mut_recursive(k);
                *self.parent.get_unchecked_mut(ix) = xrep;
            }
        }
        self.parent
    }
}

#[cfg(test)]
mod tests {
    use indexmap::IndexSet;
    use proptest::{collection::vec, prelude::*};

    use crate::utilities::union_find::UnionFind;

    #[test]
    fn union() {
        let n = 8;
        let mut u = UnionFind::new(n);

        // [{0}, {1}, {2}, {3}, {4}, {5}, {6}, {7}]
        for i in 0..n {
            assert_eq!(u.find(i), i);
            assert_eq!(u.find_mut(i), i);
            assert!(!u.union(i, i));
        }

        // [{0, 1}, {2}, {3}, {4}, {5}, {6}, {7}]
        u.union(0, 1);
        assert_eq!(u.find(0), u.find(1));

        // [{0, 1, 3}, {2}, {4}, {5}, {6}, {7}]
        u.union(1, 3);
        assert_eq!(u.find(0), u.find(3));
        assert_eq!(u.find(1), u.find(3));

        // [{0, 1, 3, 4}, {2}, {5}, {6}, {7}]
        u.union(1, 4);
        // [{0, 1, 3, 4, 7}, {2}, {5}, {6}]
        u.union(4, 7);
        assert_ne!(u.find(0), u.find(2));
        assert_eq!(u.find(7), u.find(0));

        // [{0, 1, 3, 4, 7}, {2}, {5, 6}]
        u.union(5, 6);
        assert_eq!(u.find(6), u.find(5));
        assert_ne!(u.find(6), u.find(7));

        // check that there are now 3 disjoint sets
        let set = (0..n).map(|i| u.find(i)).collect::<IndexSet<_>>();
        assert_eq!(set.len(), 3);
    }

    #[test]
    fn equivalence() {
        let n = 8;
        let mut u = UnionFind::new(n);

        // [{0}, {1}, {2}, {3}, {4}, {5}, {6}, {7}]
        for i in 0..n {
            assert_eq!(u.find(i), i);
            assert_eq!(u.find_mut(i), i);
            assert!(u.equiv(i, i));
        }

        // [{0, 1}, {2}, {3}, {4}, {5}, {6}, {7}]
        u.union(0, 1);
        assert!(u.equiv(0, 1));

        // [{0, 1, 3}, {2}, {4}, {5}, {6}, {7}]
        u.union(1, 3);
        assert!(u.equiv(1, 3));

        // [{0, 1, 3, 4}, {2}, {5}, {6}, {7}]
        u.union(1, 4);
        // [{0, 1, 3, 4, 7}, {2}, {5}, {6}]
        u.union(4, 7);
        assert!(u.equiv(0, 7));
        assert!(u.equiv(7, 0));

        assert!(!u.equiv(0, 2));

        // [{0, 1, 3, 4, 7}, {2}, {5, 6}]
        u.union(5, 6);
        assert!(u.equiv(6, 5));
        assert!(!u.equiv(6, 7));

        // check that there are now 3 disjoint sets
        let set = (0..n).map(|i| u.find(i)).collect::<IndexSet<_>>();
        assert_eq!(set.len(), 3);
    }

    const N_U16: usize = u16::MAX as usize;
    const N_U8: usize = u8::MAX as usize;

    // This code is not ideal, but it's the best I can do for now. This is mostly 1:1 ported from
    // petgraph 0.6.3.
    #[cfg(not(miri))]
    proptest! {
        #[test]
        fn integration(elements in vec((0..N_U16, 0..N_U16), 1..128)) {
            let mut u = UnionFind::new(N_U16);

            for (a, b) in elements {
                let ar = u.find(a);
                let br = u.find(b);

                assert_eq!(ar != br, u.union(a, b));
            }
        }

        #[test]
        fn integration_u8(elements in vec((0..u8::MAX, 0..u8::MAX), 1..(N_U8*8))) {
            let mut u = UnionFind::<u8>::new(N_U8);

            for (a, b) in elements {
                let ar = u.find(a);
                let br = u.find(b);

                assert_eq!(ar != br, u.union(a, b));
            }
        }
    }

    #[test]
    fn labeling() {
        // [{0}, {1}, ..., {47}]
        let mut u = UnionFind::<u32>::new(48);

        // [{0, ..., 24}, {25}, ..., {47}]
        for i in 0..24 {
            u.union(i + 1, i);
        }

        // [{0, ..., 24}, {25, ..., 47}]
        for i in 25..47 {
            u.union(i, i + 1);
        }

        // [{0, ..., 24, 25, ..., 47}]
        assert!(u.union(23, 25));
        // we already joined them
        assert!(!u.union(24, 23));

        let v = u.into_labeling();
        assert!(v.iter().all(|x| *x == v[0]));
    }
}
