use petgraph_core::{base::MaybeOwned, Edge, GraphStorage};

pub struct Cost<T>(pub(in crate::shortest_paths) T);

impl<T> Cost<T> {
    pub fn value(&self) -> &T {
        &self.0
    }

    pub fn into_value(self) -> T {
        self.0
    }
}

pub struct DefaultCost;

pub trait GraphCost<S>
where
    S: GraphStorage,
{
    type Value;

    fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, Self::Value>;
}

impl<S, F, T> GraphCost<S> for F
where
    S: GraphStorage,
    F: Fn(Edge<S>) -> MaybeOwned<T>,
{
    type Value = T;

    fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, T> {
        self(edge)
    }
}

impl<S> GraphCost<S> for DefaultCost
where
    S: GraphStorage,
{
    type Value = S::EdgeWeight;

    fn cost<'a>(&self, edge: Edge<'a, S>) -> MaybeOwned<'a, S::EdgeWeight> {
        MaybeOwned::Borrowed(edge.weight())
    }
}

#[cfg(test)]
mod tests {
    use petgraph_core::{base::MaybeOwned, edge::marker::Directed, Edge, GraphStorage};
    use petgraph_dino::DinoStorage;

    use crate::shortest_paths::common::cost::{DefaultCost, GraphCost};

    fn needs_cost_fn<S, F, T>(_: F)
    where
        S: GraphStorage,
        F: GraphCost<S, Value = T>,
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
        needs_cost_fn::<UsizeStorage, _, usize>(DefaultCost);
    }
}
