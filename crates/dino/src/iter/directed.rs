use petgraph_core::{edge::Direction, Edge, GraphDirectionality, GraphStorage};

use crate::{iter::closure::NodeIdClosureIter, node::Node, slab::Key, DinoStorage, EdgeId, NodeId};

pub struct NodeDirectedConnectionsIter<'storage, N, E, D, I>
where
    D: GraphDirectionality,
{
    pub(crate) storage: &'storage DinoStorage<N, E, D>,
    pub(crate) iter: Option<I>,
}

impl<'storage, N, E, D, I> Iterator for NodeDirectedConnectionsIter<'storage, N, E, D, I>
where
    D: GraphDirectionality,
    I: Iterator<Item = EdgeId>,
{
    type Item = Edge<'storage, DinoStorage<N, E, D>>;

    fn next(&mut self) -> Option<Self::Item> {
        let id = self.iter.as_mut()?.next()?;

        let edge = self.storage.edges.get_unchecked(id)?;

        Some(Edge::new(
            self.storage,
            &edge.id,
            &edge.weight,
            &edge.source,
            &edge.target,
        ))
    }
}
