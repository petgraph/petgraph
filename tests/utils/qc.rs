use core::ops::Deref;
use petgraph::{graph::DiGraph, graphmap::NodeTrait};
use quickcheck::{Arbitrary, Gen};

use crate::gen_range;

#[derive(Copy, Clone, Debug)]
/// quickcheck Arbitrary adaptor - half the size of `T` on average
pub struct Small<T>(pub T);

impl<T> Deref for Small<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T> Arbitrary for Small<T>
where
    T: Arbitrary,
{
    fn arbitrary(g: &mut Gen) -> Self {
        let sz = g.size() / 2;
        Small(T::arbitrary(&mut Gen::new(sz)))
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        Box::new((**self).shrink().map(Small))
    }
}

#[cfg(feature = "stable_graph")]
/// A directed graph where each pair of nodes has exactly one edge between them, and no loops.
#[derive(Clone, Debug)]
pub struct Tournament<N, E>(pub DiGraph<N, E>);

/// `Arbitrary` for `Tournament` creates a graph with arbitrary node count, and exactly one edge of
/// arbitrary direction between each pair of nodes, and no loops. The average node count is reduced,
/// to mitigate the high edge count.
impl<N, E> Arbitrary for Tournament<N, E>
where
    N: NodeTrait + Arbitrary,
    E: Arbitrary,
{
    fn arbitrary(g: &mut Gen) -> Self {
        let g_size_sqrt = (g.size() as f64).sqrt() as usize;
        let nodes = gen_range(g, 0..g_size_sqrt);
        if nodes == 0 {
            return Tournament(DiGraph::with_capacity(0, 0));
        }

        let mut gr = DiGraph::new();
        for _ in 0..nodes {
            gr.add_node(N::arbitrary(g));
        }
        for i in gr.node_indices() {
            for j in gr.node_indices() {
                if i >= j {
                    continue;
                }
                let (source, target) = if bool::arbitrary(g) { (i, j) } else { (j, i) };
                gr.add_edge(source, target, E::arbitrary(g));
            }
        }
        Tournament(gr)
    }

    fn shrink(&self) -> Box<dyn Iterator<Item = Self>> {
        let Tournament(gr) = self;
        Box::new(gr.shrink().map(Tournament))
    }
}
