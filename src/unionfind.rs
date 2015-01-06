
#[derive(Show, Copy, Clone)]
struct Elt {
    rank: uint,
    set: uint,
}

/// A disjoint-set data structure or “Union & Find” datastructure.
///
/// http://en.wikipedia.org/wiki/Disjoint-set_data_structure
///
/// Too awesome not to quote:
///
/// “The amortized time per operation is **O(α(n))** where **α(n)** is the
/// inverse of **f(x) = A(x, x)** with **A** being the extremely fast-growing Ackermann function.”
#[derive(Show, Clone)]
pub struct UnionFind {
    v: Vec<Elt>,
}

impl UnionFind
{
    /// Create a new **UnionFind** of **n** disjoint sets.
    pub fn new(n: uint) -> Self
    {
        let mut v = Vec::with_capacity(n);
        for index in range(0, n) {
            v.push(Elt{ rank: 0, set: index});
        }
        UnionFind{v: v}
    }

    /// Return the representative for **x**.
    ///
    /// **Panics** if **x** is out of bounds.
    pub fn find(&self, x: uint) -> uint
    {
        let xparent = self.v[x].set;
        if xparent == x {
            x
        } else {
            unsafe {
                self.find_rep(xparent)
            }
        }
    }

    /// Return the reprsentative for **x**.
    ///
    /// Use unchecked indexing because we can trust the internal set ids.
    #[inline]
    unsafe fn find_rep(&self, x: uint) -> uint
    {
        let mut x = x;
        loop {
            debug_assert!(x < self.v.len());
            let xparent = self.v.get_unchecked(x);
            if xparent.set == x {
                break
            }
            x = xparent.set;
        }
        x
    }

    /// Return the representative for **x**.
    ///
    /// Write back the found representative, flattening the internal
    /// datastructure in the process and quicken future lookups.
    ///
    /// **Panics** if **x** is out of bounds.
    pub fn find_mut(&mut self, x: uint) -> uint
    {
        self.find_compress(x).set
    }

    /// Return the representative for **x**.
    ///
    /// **Panics** if **x** is out of bounds.
    fn find_compress(&mut self, x: uint) -> Elt
    {
        let xparent = self.v[x];
        if xparent.set != x {
            // path compression: update set id to point directly to the representative
            unsafe {
                let i = self.find_rep(xparent.set);
                let xparent = self.v.get_unchecked_mut(x);
                xparent.set = i;
                *xparent
            }
        } else {
            xparent
        }
    }

    /// Unify the two sets containing **x** and **y**.
    ///
    /// Return **false** if the sets were already the same, **true** if they were unified.
    /// 
    /// **Panics** if **x** or **y** is out of bounds.
    pub fn union(&mut self, x: uint, y: uint) -> bool
    {
        if x == y {
            return false
        }

        let xrep = self.find_compress(x);
        let yrep = self.find_compress(y);

        if xrep.set == yrep.set {
            return false
        }

        // The rank corresponds roughly to the depth of the treeset, so put the 
        // smaller set below the larger
        if xrep.rank < yrep.rank {
            self.v[xrep.set].set = yrep.set;
        } else if xrep.rank > yrep.rank {
            self.v[yrep.set].set = xrep.set;
        } else {
            // put y below x when equal.
            self.v[yrep.set].set = xrep.set;
            self.v[xrep.set].rank += 1;
        }
        true
    }
}
