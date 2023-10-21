use petgraph_core::{base::MaybeOwned, Edge, GraphStorage};

trait CostReturn {
    type Owned;

    fn into_maybe_owned(self) -> MaybeOwned<Self::Owned>;
}

impl<T> CostReturn for T {
    type Owned = T;

    fn into_maybe_owned(self) -> MaybeOwned<Self::Owned> {
        MaybeOwned::Owned(self)
    }
}

impl<T> CostReturn for &T {
    type Owned = T;

    fn into_maybe_owned(self) -> MaybeOwned<Self::Owned> {
        MaybeOwned::Owned(self)
    }
}

// impl<'a, T> CostReturn<T> for &'a T {
//     fn into_maybe_owned(self) -> MaybeOwned<T> {
//         MaybeOwned::Borrowed(self)
//     }
// }

trait CostArgument<S>
where
    S: GraphStorage,
{
    fn from_edge(edge: &Edge<S>) -> Self;
}

impl<S> CostArgument<S> for Edge<'_, S>
where
    S: GraphStorage,
{
    fn from_edge(edge: &Edge<S>) -> Self {
        *edge
    }
}

impl<S> CostArgument<S> for &Edge<'_, S>
where
    S: GraphStorage,
{
    fn from_edge(edge: &Edge<S>) -> Self {
        edge
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
    F: Fn(Edge<S>) -> T,
    T: CostReturn,
{
    type Cost = T::Owned;

    fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, Self::Cost> {
        let cost = self(edge);

        cost.into_maybe_owned()
    }
}

impl<S> CostFn<S> for ()
where
    S: GraphStorage,
{
    type Cost = S::EdgeWeight;

    fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, Self::Cost> {
        MaybeOwned::Borrowed(edge.weight())
    }
}

// impl<S, F, T> CostFn<S> for F
// where
//     S: GraphStorage,
//     F: Fn(&Edge<S>) -> T,
// {
//     type Cost = T;
//
//     fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, Self::Cost> {
//         MaybeOwned::from(self(&edge))
//     }
// }

// impl<S, F, T> CostFn<S> for F
// where
//     S: GraphStorage,
//     F: Fn(Edge<S>) -> MaybeOwned<T>,
// {
//     type Cost = T;
//
//     fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, Self::Cost> {
//         self(edge)
//     }
// }

#[cfg(test)]
mod tests {
    use petgraph_core::{base::MaybeOwned, edge::marker::Directed, Edge, GraphStorage};
    use petgraph_dino::{DiDinoGraph, DinoStorage};

    use crate::shortest_paths::common::cost::CostFn;

    fn needs_cost_fn<S, F, T>(_: F)
    where
        S: GraphStorage,
        F: CostFn<S, Cost = T>,
    {
    }

    fn edge_cost<S>(edge: Edge<S>) -> usize
    where
        S: GraphStorage,
        S::EdgeWeight: AsRef<[u8]>,
    {
        edge.weight().as_ref().len()
    }

    fn ref_edge_cost<S>(edge: Edge<S>) -> &usize
    where
        S: GraphStorage<EdgeWeight = usize>,
    {
        edge.weight()
    }

    fn maybe_edge_cost<S>(edge: Edge<S>) -> MaybeOwned<'_, usize>
    where
        S: GraphStorage,
        S::EdgeWeight: AsRef<[u8]>,
    {
        MaybeOwned::Owned(edge.weight().as_ref().len())
    }

    #[test]
    fn trait_impl() {
        type StrStorage = DinoStorage<&'static str, &'static str, Directed>;
        type UsizeStorage = DinoStorage<usize, usize, Directed>;

        needs_cost_fn::<StrStorage, _, usize>(edge_cost);
        needs_cost_fn::<UsizeStorage, _, usize>(ref_edge_cost);
        // needs_cost_fn::<StrStorage, _, usize>(maybe_edge_cost);
        // needs_cost_fn::<Storage, _, <Storage as GraphStorage>::EdgeWeight>(Edge::into_weight);
    }
}
