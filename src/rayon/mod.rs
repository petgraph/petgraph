use crate::graphmap::{CompactDirection, GraphMap};
use indexmap::map::rayon::ParKeys;
use rayon::{iter::plumbing::UnindexedConsumer, prelude::*};

/// A [ParallelIterator] over this graph's nodes.
pub struct ParNodes<'a, N>
where
    N: Send + Sync,
{
    iter: ParKeys<'a, N, Vec<(N, CompactDirection)>>,
}

impl<'a, N> ParallelIterator for ParNodes<'a, N>
where
    N: Send + Sync,
{
    type Item = &'a N;

    fn drive_unindexed<C>(self, c: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.iter.drive_unindexed(c)
    }
}

/// Adds parallel iterators where possible, i.e. where nodes are [Send] + [Sync].
pub trait NodesParIter<'a, N>
where
    N: Send + Sync,
{
    /// Returns a parallel iterator over this graph's nodes.
    fn par_nodes(&'a self) -> ParNodes<'a, N>;
}

impl<'a, N, E, Ty> NodesParIter<'a, N> for GraphMap<N, E, Ty>
where
    N: Send + Sync,
{
    fn par_nodes(&'a self) -> ParNodes<'a, N> {
        ParNodes {
            iter: self.nodes.par_keys(),
        }
    }
}
