use core::{borrow::Borrow, fmt::Display};

use numi::borrow::Moo;
use petgraph_core::{Edge, Graph};

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Cost<T>(T);

impl<T> Cost<T> {
    pub const fn new(value: T) -> Self {
        Self(value)
    }

    pub const fn value(&self) -> &T {
        &self.0
    }

    pub fn into_value(self) -> T {
        self.0
    }
}

// ensure that all traits have been implemented
// see: https://rust-lang.github.io/api-guidelines/interoperability.html
#[cfg(test)]
static_assertions::assert_impl_all!(Cost<&'static str>: core::fmt::Debug, Display, Clone, PartialEq, Eq, PartialOrd, Ord, core::hash::Hash, Send, Sync, From<&'static str>, AsRef<&'static str>, Borrow<&'static str>);

impl<T> Display for Cost<T>
where
    T: Display,
{
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<T> From<T> for Cost<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> AsRef<T> for Cost<T> {
    fn as_ref(&self) -> &T {
        &self.0
    }
}

impl<T> Borrow<T> for Cost<T> {
    fn borrow(&self) -> &T {
        &self.0
    }
}

pub struct DefaultCost;

pub trait GraphCost<S>
where
    S: Graph,
{
    type Value;

    fn cost<'graph>(&self, edge: Edge<'graph, S>) -> Moo<'graph, Self::Value>;
}

impl<S, F, T> GraphCost<S> for F
where
    S: Graph,
    F: Fn(Edge<S>) -> Moo<T>,
{
    type Value = T;

    fn cost<'graph>(&self, edge: Edge<'graph, S>) -> Moo<'graph, T> {
        self(edge)
    }
}

impl<S> GraphCost<S> for DefaultCost
where
    S: Graph,
{
    type Value = S::EdgeWeight;

    fn cost<'graph>(&self, edge: Edge<'graph, S>) -> Moo<'graph, S::EdgeWeight> {
        Moo::Borrowed(edge.weight())
    }
}

#[cfg(test)]
mod tests {
    use numi::borrow::Moo;
    use petgraph_core::{Edge, Graph, edge::marker::Directed};
    use petgraph_dino::DinoStorage;

    use crate::shortest_paths::common::cost::{DefaultCost, GraphCost};

    fn needs_cost_fn<S, F, T>(_: F)
    where
        S: Graph,
        F: GraphCost<S, Value = T>,
    {
    }

    fn maybe_edge_cost<S>(edge: Edge<S>) -> Moo<'_, usize>
    where
        S: Graph,
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
