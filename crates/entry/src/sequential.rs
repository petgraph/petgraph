use core::hash::Hash;

use petgraph_core::{storage::SequentialGraphStorage, GraphDirectionality};

use crate::{Backend, EntryStorage};

impl<NK, NV, EK, EV, D> SequentialGraphStorage for EntryStorage<NK, NV, EK, EV, D>
where
    D: GraphDirectionality,
    NK: Hash,
    EK: Hash,
{
    type EdgeIdBijection<'a> = <Backend<NK, NV, EK, EV, D> as SequentialGraphStorage>::EdgeIdBijection<'a> where Self: 'a;
    type NodeIdBijection<'a> = <Backend<NK, NV, EK, EV, D> as SequentialGraphStorage>::NodeIdBijection<'a> where Self: 'a;

    fn node_id_bijection(&self) -> Self::NodeIdBijection<'_> {
        self.inner.node_id_bijection()
    }

    fn edge_id_bijection(&self) -> Self::EdgeIdBijection<'_> {
        self.inner.edge_id_bijection()
    }
}
