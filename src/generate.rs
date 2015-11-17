
use fb::FixedBitSet;
use std::default::Default;
use {Graph, Directed};
use graph::NodeIndex;

/*
pub struct DAG {
    size: usize,
    bits: FixedBitSet,
}

// A DAG has the property that the adjacency matrix is lower triangular,
// diagonal zero.
//
// This means we only allow edges i → j where i < j.
//
// The set of all DAG of a particular size is simply the power set of all
// possible edges.
//
// For a graph of n=3 nodes we have (n - 1) * n / 2 = 3 possible edges.
//
// Gray code
//
// gray(x) { x ^ (x >> 1) }
//
// See fxtbook on gray codes
//
// Use a gray code sequence to efficiently step through the whole set of edges

impl DAG {
    pub fn new(size: usize) -> Self {
        DAG {
            size: size,
            bits: FixedBitSet::with_capacity(size),
        }
    }

    fn state_to_graph(&self) -> Graph<(), (), Directed> {
        let popcount = self.bits.as_slice().iter()
                                .fold(0, |acc, x| acc + x.count_ones() as usize);
        Graph::with_capacity(self.size, popcount)
    }
}
*/

/// Generate all possible Directed acyclic graphs (DAGs) of a particular size.
///
/// For a graph of size *k* there are *e = (k - 1) k / 2* possible edges and
/// *2<sup>e</sup>* DAGs.
pub struct DAG {
    size: usize,
    bits: u64,
    g: Graph<(), (), Directed>,
}

impl DAG {
    pub fn new(size: usize) -> Self {
        assert!(size != 0);
        let nedges = (size - 1) * size / 2;
        assert!(nedges <= 64);
        DAG {
            size: size,
            bits: !0,
            g: Graph::with_capacity(size, nedges),
        }
    }

    fn state_to_graph(&mut self) -> &Graph<(), (), Directed> {
        let popcount = self.bits.count_ones() as usize;
        self.g.clear();
        //let mut g = Graph::with_capacity(self.size, popcount);
        for _ in 0..self.size {
            self.g.add_node(());
        }
        // interpret the bits in order, it's a lower triangular matrix:
        //   a b c d
        // a x x x x
        // b 0 x x x
        // c 1 2 x x
        // d 3 4 5 x
        let mut bit = 0;
        for i in 0..self.size {
            for j in i+1..self.size {
                if self.bits & (1u64 << bit) != 0 {
                    self.g.add_edge(NodeIndex::new(i), NodeIndex::new(j), ());
                }

                bit += 1;
            }
        }
        &self.g
    }

    pub fn next_ref(&mut self) -> Option<&Graph<(), (), Directed>> {
        if self.bits == !0 {
            self.bits = 0;
        } else {
            self.bits += 1;
            let nedges = (self.size - 1) * self.size / 2;
            if self.bits >= 1u64 << nedges {
                return None;
            }
        }
        Some(self.state_to_graph())
    }
}

impl Iterator for DAG {
    type Item = Graph<(), (), Directed>;

    fn next(&mut self) -> Option<Self::Item> {
        self.next_ref().cloned()
    }
}
