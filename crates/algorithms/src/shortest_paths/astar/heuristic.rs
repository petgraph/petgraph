use numi::borrow::Moo;
use petgraph_core::{Graph, Node};

pub trait GraphHeuristic<S>
where
    S: Graph,
{
    type Value;

    fn estimate<'a>(&self, source: Node<'a, S>, target: Node<'a, S>) -> Moo<'a, Self::Value>;
}

impl<S, F, T> GraphHeuristic<S> for F
where
    S: Graph,
    F: for<'a> Fn(Node<'a, S>, Node<'a, S>) -> Moo<'a, T>,
{
    type Value = T;

    fn estimate<'a>(&self, source: Node<'a, S>, target: Node<'a, S>) -> Moo<'a, T> {
        self(source, target)
    }
}
