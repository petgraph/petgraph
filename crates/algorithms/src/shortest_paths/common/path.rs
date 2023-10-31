use core::fmt::{Debug, Formatter};
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

impl<'a, S> PartialEq for Path<'a, S>
where
    S: GraphStorage,
    Node<'a, S>: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        (&self.source, &self.target, &self.transit)
            == (&other.source, &other.target, &other.transit)
    }
}

impl<'a, S> Eq for Path<'a, S>
where
    S: GraphStorage,
    Node<'a, S>: Eq,
{
}

impl<S> Clone for Path<'_, S>
where
    S: GraphStorage,
{
    fn clone(&self) -> Self {
        Self {
            source: self.source,
            target: self.target,
            transit: self.transit.clone(),
        }
    }
}

impl<'a, S> Debug for Path<'a, S>
where
    S: GraphStorage,
    Node<'a, S>: Debug,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Path")
            .field("source", &self.source)
            .field("target", &self.target)
            .field("transit", &self.transit)
            .finish()
    }
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
