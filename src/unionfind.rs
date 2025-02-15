//! `UnionFind<K>` is a disjoint-set data structure.

use super::graph::IndexType;
use std::cmp::Ordering;

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
pub struct UnionFind<K> {
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

impl<K> Default for UnionFind<K> {
    fn default() -> Self {
        Self {
            parent: Vec::new(),
            rank: Vec::new(),
        }
    }
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
        let parent = (0..n).map(K::new).collect::<Vec<K>>();

        UnionFind { parent, rank }
    }

    /// Create a new `UnionFind` with no elements.
    pub const fn new_empty() -> Self {
        Self {
            parent: Vec::new(),
            rank: Vec::new(),
        }
    }

    /// Returns the number of elements in the union-find data-structure.
    pub fn len(&self) -> usize {
        self.parent.len()
    }

    /// Returns true if there are no elements in the union-find data-structure.
    pub fn is_empty(&self) -> bool {
        self.parent.is_empty()
    }

    /// Adds a new disjoint set and returns the index of the new set.
    ///
    /// The new disjoint set is always added to the end, so the returned
    /// index is the same as the number of elements before calling this function.
    ///
    /// **Time Complexity**
    /// Takes amortized O(1) time.
    pub fn new_set(&mut self) -> K {
        let retval = K::new(self.parent.len());
        self.rank.push(0);
        self.parent.push(retval);
        retval
    }

    /// Return the representative for `x`.
    ///
    /// **Panics** if `x` is out of bounds.
    pub fn find(&self, x: K) -> K {
        self.try_find(x).expect("The index is out of bounds")
    }

    /// Return the representative for `x` or `None` if `x` is out of bounds.
    pub fn try_find(&self, mut x: K) -> Option<K> {
        if x.index() >= self.len() {
            return None;
        }

        loop {
            // Use unchecked indexing because we can trust the internal set ids.
            let xparent = unsafe { *get_unchecked(&self.parent, x.index()) };
            if xparent == x {
                break;
            }
            x = xparent;
        }

        Some(x)
    }

    /// Return the representative for `x`.
    ///
    /// Write back the found representative, flattening the internal
    /// datastructure in the process and quicken future lookups.
    ///
    /// **Panics** if `x` is out of bounds.
    pub fn find_mut(&mut self, x: K) -> K {
        assert!(x.index() < self.len());
        unsafe { self.find_mut_recursive(x) }
    }

    /// Return the representative for `x` or `None` if `x` is out of bounds.
    ///
    /// Write back the found representative, flattening the internal
    /// datastructure in the process and quicken future lookups.
    pub fn try_find_mut(&mut self, x: K) -> Option<K> {
        if x.index() >= self.len() {
            return None;
        }
        Some(unsafe { self.find_mut_recursive(x) })
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
    ///
    /// **Panics** if `x` or `y` is out of bounds.
    pub fn equiv(&self, x: K, y: K) -> bool {
        self.find(x) == self.find(y)
    }

    /// Returns `Ok(true)` if the given elements belong to the same set, and returns
    /// `Ok(false)` otherwise.
    ///
    /// If `x` or `y` are out of bounds, it returns `Err` with the first bad index found.
    pub fn try_equiv(&self, x: K, y: K) -> Result<bool, K> {
        let xrep = self.try_find(x).ok_or(x)?;
        let yrep = self.try_find(y).ok_or(y)?;
        Ok(xrep == yrep)
    }

    /// Unify the two sets containing `x` and `y`.
    ///
    /// Return `false` if the sets were already the same, `true` if they were unified.
    ///
    /// **Panics** if `x` or `y` is out of bounds.
    pub fn union(&mut self, x: K, y: K) -> bool {
        self.try_union(x, y).unwrap()
    }

    /// Unify the two sets containing `x` and `y`.
    ///
    /// Return `Ok(false)` if the sets were already the same, `Ok(true)` if they were unified.
    ///
    /// If `x` or `y` are out of bounds, it returns `Err` with first found bad index.
    /// But if `x == y`, the result will be `Ok(false)` even if the indexes go out of bounds.
    pub fn try_union(&mut self, x: K, y: K) -> Result<bool, K> {
        if x == y {
            return Ok(false);
        }
        let xrep = self.try_find_mut(x).ok_or(x)?;
        let yrep = self.try_find_mut(y).ok_or(y)?;

        if xrep == yrep {
            return Ok(false);
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
        Ok(true)
    }

    /// Return a vector mapping each element to its representative.
    pub fn into_labeling(mut self) -> Vec<K> {
        // write in the labeling of each element
        unsafe {
            for ix in 0..self.len() {
                let k = *get_unchecked(&self.parent, ix);
                let xrep = self.find_mut_recursive(k);
                *self.parent.get_unchecked_mut(ix) = xrep;
            }
        }
        self.parent
    }
}
