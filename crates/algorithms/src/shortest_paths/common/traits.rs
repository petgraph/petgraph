use petgraph_core::{Edge, GraphStorage, Node};

pub(in crate::shortest_paths) trait ConnectionFn<'a, S>
where
    S: GraphStorage,
{
    fn connections(&self, node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a;
}

impl<'a, S, I> ConnectionFn<'a, S> for fn(&Node<'a, S>) -> I
where
    S: GraphStorage,
    I: Iterator<Item = Edge<'a, S>> + 'a,
{
    fn connections(&self, node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a {
        (*self)(node)
    }
}
