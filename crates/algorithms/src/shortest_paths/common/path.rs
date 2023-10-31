use std::iter::once;

use petgraph_core::{GraphStorage, Node};

pub struct Path<'a, S>
where
    S: GraphStorage,
{
    pub(in crate::shortest_paths) source: Node<'a, S>,
    pub(in crate::shortest_paths) target: Node<'a, S>,

    pub(in crate::shortest_paths) transit: Vec<Node<'a, S>>,
}

impl<'a, S> Path<'a, S>
where
    S: GraphStorage,
{
    pub fn source(&self) -> Node<'a, S> {
        self.source
    }

    pub fn target(&self) -> Node<'a, S> {
        self.target
    }

    pub fn transit(&self) -> &[Node<'a, S>] {
        &self.transit
    }

    #[must_use]
    pub fn to_vec(self) -> Vec<Node<'a, S>> {
        let mut vec = Vec::with_capacity(self.transit.len() + 2);

        vec.push(self.source);
        vec.extend(self.transit);
        vec.push(self.target);

        vec
    }

    pub fn iter(&self) -> impl Iterator<Item = Node<'a, S>> + '_ {
        once(self.source)
            .chain(self.transit.iter().copied())
            .chain(once(self.target))
    }

    pub(in crate::shortest_paths) fn reverse(self) -> Self {
        let mut transit = self.transit;

        transit.reverse();

        Self {
            source: self.target,
            target: self.source,
            transit,
        }
    }
}

impl<'a, S> IntoIterator for Path<'a, S>
where
    S: GraphStorage,
{
    type IntoIter = alloc::vec::IntoIter<Node<'a, S>>;
    type Item = Node<'a, S>;

    fn into_iter(self) -> Self::IntoIter {
        self.to_vec().into_iter()
    }
}
