use core::hash::Hash;

use petgraph_core::{
    storage::{auxiliary::Hints, AuxiliaryGraphStorage},
    GraphDirectionality,
};

use crate::{Backend, EntryStorage};

impl<NK, NV, EK, EV, D> AuxiliaryGraphStorage for EntryStorage<NK, NV, EK, EV, D>
where
    D: GraphDirectionality,
    NK: Hash,
    EK: Hash,
{
    type BooleanEdgeStorage<'a> = <Backend<NK, NV, EK, EV, D> as AuxiliaryGraphStorage>::BooleanEdgeStorage<'a> where Self: 'a;
    type BooleanNodeStorage<'a> = <Backend<NK, NV, EK, EV, D> as AuxiliaryGraphStorage>::BooleanNodeStorage<'a> where Self: 'a;
    type SecondaryEdgeStorage<'a, V> = <Backend<NK, NV, EK, EV, D> as AuxiliaryGraphStorage>::SecondaryEdgeStorage<'a, V> where Self: 'a;
    type SecondaryNodeStorage<'a, V> = <Backend<NK, NV, EK, EV, D> as AuxiliaryGraphStorage>::SecondaryNodeStorage<'a, V> where Self: 'a;

    fn secondary_node_storage<V>(&self, hints: Hints) -> Self::SecondaryNodeStorage<'_, V> {
        self.inner.secondary_node_storage(hints)
    }

    fn secondary_edge_storage<V>(&self, hints: Hints) -> Self::SecondaryEdgeStorage<'_, V> {
        self.inner.secondary_edge_storage(hints)
    }

    fn boolean_node_storage(&self, hints: Hints) -> Self::BooleanNodeStorage<'_> {
        self.inner.boolean_node_storage(hints)
    }

    fn boolean_edge_storage(&self, hints: Hints) -> Self::BooleanEdgeStorage<'_> {
        self.inner.boolean_edge_storage(hints)
    }
}
