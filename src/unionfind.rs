//! **UnionFind** is a disjoint-set data structure.


/// **UnionFind** is a disjoint-set data structure.
///
/// http://en.wikipedia.org/wiki/Disjoint-set_data_structure
///
/// Too awesome not to quote:
///
/// “The amortized time per operation is **O(α(n))** where **α(n)** is the
/// inverse of **f(x) = A(x, x)** with **A** being the extremely fast-growing Ackermann function.”
#[derive(Show, Clone)]
pub struct UnionFind {
    parent: Vec<uint>,
    // It is a balancing tree structure,
    // so the ranks are logarithmic in the size of the container -- a byte is more than enough.
    //
    // Rank is separated out both to save space and to save cache in when searching in the parent
    // vector.
    rank: Vec<u8>,
}

impl UnionFind
{
    /// Create a new **UnionFind** of **n** disjoint sets.
    pub fn new(n: uint) -> Self
    {
        let mut parent = Vec::with_capacity(n);
        let mut rank = Vec::with_capacity(n);
        for index in range(0, n) {
            parent.push(index);
            rank.push(0);
        }
        UnionFind{parent: parent, rank: rank}
    }

    /// Return the representative for **x**.
    ///
    /// **Panics** if **x** is out of bounds.
    pub fn find(&self, x: uint) -> uint
    {
        assert!(x < self.parent.len());
        unsafe {
            let mut x = x;
            loop {
                // Use unchecked indexing because we can trust the internal set ids.
                debug_assert!(x < self.parent.len());
                let xparent = *self.parent.get_unchecked(x);
                if xparent == x {
                    break
                }
                x = xparent;
            }
            x
        }
    }

    /// Return the representative for **x**.
    ///
    /// Write back the found representative, flattening the internal
    /// datastructure in the process and quicken future lookups.
    ///
    /// **Panics** if **x** is out of bounds.
    pub fn find_mut(&mut self, x: uint) -> uint
    {
        assert!(x < self.parent.len());
        unsafe {
            self.find_mut_recursive(x)
        }
    }

    unsafe fn find_mut_recursive(&mut self, x: uint) -> uint
    {
        debug_assert!(x < self.parent.len());
        let xparent = *self.parent.get_unchecked(x);
        if xparent != x {
            let xrep = self.find_mut_recursive(xparent);
            let xparent = self.parent.get_unchecked_mut(x);
            *xparent = xrep;
            *xparent
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

        let xrep = self.find_mut(x);
        let yrep = self.find_mut(y);

        if xrep == yrep {
            return false
        }

        let xrank = self.rank[xrep];
        let yrank = self.rank[yrep];

        // The rank corresponds roughly to the depth of the treeset, so put the 
        // smaller set below the larger
        if xrank < yrank {
            self.parent[xrep] = yrep;
        } else if xrank > yrank {
            self.parent[yrep] = xrep;
        } else {
            // put y below x when equal.
            self.parent[yrep] = xrep;
            self.rank[xrep] += 1;
        }
        true
    }
}
