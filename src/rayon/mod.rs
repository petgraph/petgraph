use crate::graphmap::CompactDirection;
use crate::prelude::GraphMap;
use indexmap::map::rayon::ParKeys;
use rayon::iter::plumbing::UnindexedConsumer;
use rayon::prelude::*;

pub struct ParNodes<'a, N>
where
    N: Send + Sync + Clone,
{
    iter: ParKeys<'a, N, Vec<(N, CompactDirection)>>,
}

impl<'a, N> ParallelIterator for ParNodes<'a, N>
where
    N: Send + Sync + Clone,
{
    type Item = &'a N;

    fn drive_unindexed<C>(self, c: C) -> C::Result
    where
        C: UnindexedConsumer<Self::Item>,
    {
        self.iter.drive_unindexed(c)
    }
}

pub trait NodesParIter<'a, N>
where
    N: Send + Sync + Clone,
{
    fn par_nodes(&'a self) -> ParNodes<'a, N>;
}

impl<'a, N, E, Ty> NodesParIter<'a, N> for GraphMap<N, E, Ty>
where
    N: Send + Sync + Clone,
{
    fn par_nodes(&'a self) -> ParNodes<'a, N> {
        ParNodes {
            iter: self.nodes.par_keys(),
        }
    }
}
