
#[derive(Show, Copy, Clone)]
struct Elt {
    rank: uint,
    set: uint,
}

/// A disjoint-set data structure or “Union & Find” datastructure.
///
/// http://en.wikipedia.org/wiki/Disjoint-set_data_structure
#[derive(Show, Clone)]
pub struct UnionFind {
    v: Vec<Elt>,
}

impl UnionFind
{
    /// Create a new **UnionFind**.
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
        let set = self.v[x].set;
        if set == x {
            x
        } else {
            self.find(set)
        }
    }

    /// Return the representative for **x**.
    ///
    /// Find and write back the found representative, flattening the internal
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
        // path compression: update set id to point directly to the representative
        let elt = self.v[x];
        if elt.set != x {
            let parent = self.find_compress(elt.set);
            self.v[x].set = parent.set;
            parent
        } else {
            elt
        }
    }

    /// Unify the two sets containing **x** and **y**.
    ///
    /// Return false if the sets were already the same, true if they were unified.
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
