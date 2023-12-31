use petgraph_core::{edge::EdgeId, Edge, GraphDirectionality};

use crate::DinoStorage;

pub(crate) struct NodeDirectedConnectionsIter<'storage, N, E, D, I>
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
        let iter = self.iter.as_mut()?;

        loop {
            let Some(id) = iter.next() else {
                self.iter = None;
                return None;
            };

            let Some(edge) = self.storage.edges.get_unchecked(id) else {
                continue;
            };

            return Some(Edge::new(
                self.storage,
                edge.id,
                &edge.weight,
                edge.source,
                edge.target,
            ));
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.as_ref().map_or((0, Some(0)), Iterator::size_hint)
    }
}
