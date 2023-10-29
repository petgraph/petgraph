use petgraph_core::{base::MaybeOwned, GraphStorage, Node};

pub trait GraphHeuristic<S>
where
    S: GraphStorage,
{
    type Value;

    fn estimate<'a>(&self, source: Node<'a, S>, target: Node<'a, S>)
    -> MaybeOwned<'a, Self::Value>;
}

impl<S, F, T> GraphHeuristic<S> for F
where
    S: GraphStorage,
    F: for<'a> Fn(Node<'a, S>, Node<'a, S>) -> MaybeOwned<'a, T>,
{
    type Value = T;

    fn estimate<'a>(&self, source: Node<'a, S>, target: Node<'a, S>) -> MaybeOwned<'a, T> {
        self(source, target)
    }
}
