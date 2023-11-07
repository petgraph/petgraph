use petgraph_core::{edge::Direction, DirectedGraphStorage, Edge, GraphStorage, Node};

pub(in crate::shortest_paths) trait Connections<'a, S>
where
    S: GraphStorage,
{
    fn connections(&self, node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a;
}

impl<'a, S, I> Connections<'a, S> for fn(&Node<'a, S>) -> I
where
    S: GraphStorage,
    I: Iterator<Item = Edge<'a, S>> + 'a,
{
    fn connections(&self, node: &Node<'a, S>) -> impl Iterator<Item = Edge<'a, S>> + 'a {
        (*self)(node)
    }
}

pub(in crate::shortest_paths) fn outgoing_connections<'a, S>(
    node: &Node<'a, S>,
) -> impl Iterator<Item = Edge<'a, S>> + 'a
where
    S: DirectedGraphStorage,
{
    node.directed_connections(Direction::Outgoing)
}
