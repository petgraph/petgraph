use petgraph_core::{base::MaybeOwned, Edge, GraphStorage};

pub struct EdgeWeight(());

impl EdgeWeight {
    pub(super) fn new() -> Self {
        Self(())
    }
}

pub trait CostFn<S>
where
    S: GraphStorage,
{
    type Cost;

    fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, Self::Cost>;
}

impl<S, F, T> CostFn<S> for F
where
    S: GraphStorage,
    F: Fn(Edge<S>) -> MaybeOwned<T>,
{
    type Cost = T;

    fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, T> {
        self(edge)
    }
}

impl<S> CostFn<S> for EdgeWeight
where
    S: GraphStorage,
{
    type Cost = S::EdgeWeight;

    fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, S::EdgeWeight> {
        MaybeOwned::Borrowed(edge.weight())
    }
}

#[cfg(test)]
mod tests {
    use petgraph_core::{base::MaybeOwned, edge::marker::Directed, Edge, GraphStorage};
    use petgraph_dino::{DiDinoGraph, DinoStorage};

    use crate::shortest_paths::common::cost::{CostFn, EdgeWeight};

    fn needs_cost_fn<S, F, T>(_: F)
    where
        S: GraphStorage,
        F: CostFn<S, Cost = T>,
    {
    }

    fn maybe_edge_cost<S>(edge: Edge<S>) -> MaybeOwned<'_, usize>
    where
        S: GraphStorage,
        S::EdgeWeight: AsRef<[u8]>,
    {
        edge.weight().as_ref().len().into()
    }

    #[test]
    fn trait_impl() {
        type StrStorage = DinoStorage<&'static str, &'static str, Directed>;
        type UsizeStorage = DinoStorage<usize, usize, Directed>;

        needs_cost_fn::<StrStorage, _, usize>(maybe_edge_cost);
        needs_cost_fn::<UsizeStorage, _, usize>(EdgeWeight::new());
    }
}
